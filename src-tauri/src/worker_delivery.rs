use crate::worker::Worker;
use dictation_core::{
    defaults,
    ports::{PassThrough, SpeechProvider, SpeechRequest, TextProcessor},
    session::{safe_inline_text, Delivery, Phase},
};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

impl Worker {
    pub fn finish(&mut self) -> Result<(), String> {
        if let Some(mut capture) = self.capture.take() {
            match capture.drain() {
                Ok(tail) => {
                    let voiced = self.vad.accept(&tail)?;
                    self.activity.accept(&tail, voiced);
                    self.audio.extend(tail);
                }
                Err(error) => {
                    self.publish(error);
                    // Interrupted recordings are transcribed for recovery, never automatically delivered.
                    self.target = None;
                }
            }
        }
        self.stop = None;
        self.wake_pending = None;
        self.last_model_use = Instant::now();
        let id = self.session.id;
        if !self.activity.has_speech() || self.audio.is_empty() {
            self.audio.clear();
            self.session.cancel(false);
            self.runtime.authorization.store(0, Ordering::Release);
            self.listen()?;
            self.publish("No speech detected. Nothing was inserted.");
            return Ok(());
        }
        self.session.phase = Phase::Transcribing;
        self.publish("Microphone closed. Transcribing locally…");
        self.ensure_speech_model()?;
        let audio = std::mem::take(&mut self.audio);
        self.runtime.update(|s| s.inference_runs += 1);
        let request = SpeechRequest {
            session_id: id,
            samples: &audio,
            language: self
                .settings
                .language
                .as_deref()
                .ok_or("Choose a dictation language.")?,
            authorization: self.runtime.authorization.clone(),
        };
        let text = self
            .whisper
            .as_mut()
            .ok_or("Whisper was not loaded.")?
            .transcribe(request);
        drop(audio);
        self.last_model_use = Instant::now();
        if self.runtime.authorization.load(Ordering::Acquire) != id {
            self.session.cancel(self.settings.paused);
            self.publish("Session cancelled; late transcription discarded.");
            return Ok(());
        }
        let text = PassThrough.process(text?)?;
        if text.is_empty() {
            self.session.cancel(false);
            self.runtime.authorization.store(0, Ordering::Release);
            self.listen()?;
            self.publish("No usable speech was recognized. Nothing was inserted.");
            return Ok(());
        }
        if !self.session.authorize_delivery(id) {
            return Err("Outdated or duplicate delivery was prevented.".into());
        }
        let outcome = if safe_inline_text(&text) {
            self.target.as_ref().map_or(Delivery::Rejected, |target| {
                self.focus
                    .insert(target, &text, &self.runtime.authorization, id)
            })
        } else {
            Delivery::Rejected
        };
        self.complete(text, outcome)
    }

    pub fn insert_pending(&mut self) -> Result<(), String> {
        self.session.expire();
        let text = self
            .session
            .recovery
            .as_ref()
            .ok_or("Temporary text has expired.")?
            .text
            .clone();
        let target = self.focus.inspect();
        if !target.known() {
            return Err("Focus a supported editable input, then click its Insert button.".into());
        }
        if !safe_inline_text(&text) {
            return Err("This text contains line breaks or control characters. Select and copy it manually from Recovery.".into());
        }
        self.pending_armed = false;
        self.session.cancel(false);
        let id = self.session.start()?;
        self.runtime.authorization.store(id, Ordering::Release);
        self.session.phase = Phase::Transcribing;
        if !self.session.authorize_delivery(id) {
            return Err("Duplicate delivery prevented.".into());
        }
        let outcome = self
            .focus
            .insert(&target, &text, &self.runtime.authorization, id);
        self.complete(text, outcome)
    }

    fn complete(&mut self, text: String, outcome: Delivery) -> Result<(), String> {
        let message = match outcome {
            Delivery::Submitted => "Sent to app. The destination has not confirmed insertion; the latest text is temporarily available in Recovery.",
            Delivery::Rejected => "Text was not sent: the target changed, is unsupported, a modifier is held, or the text needs review. Open Recovery, or start another dictation to replace this result.",
            Delivery::Partial => "Only part of the input operation was accepted. Check the destination before using Recovery; nothing will retry automatically. Another dictation replaces this result.",
            Delivery::Unknown => "Delivery is uncertain because focus changed. Check the destination before using Recovery. Another dictation replaces this result.",
        };
        if let Ok(mut recovery) = self.runtime.recovery.lock() {
            *recovery = Some(text.clone());
        }
        self.session.delivered(text, outcome);
        self.runtime.authorization.store(0, Ordering::Release);
        self.target = None;
        self.pending_armed = false;
        self.cooldown = Instant::now() + Duration::from_millis(defaults::COOLDOWN_MILLIS);
        self.publish(message);
        // Delivery has already completed. Prepare empty buffers for the next
        // dictation before reopening capture, without retaining past audio.
        self.prepare_next_dictation()?;
        self.listen()?;
        self.publish(message);
        Ok(())
    }
}
