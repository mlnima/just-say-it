use crate::{speech::LocalWhisper, worker::Worker};
use dictation_core::session::Phase;
use std::time::{Duration, Instant};

impl Worker {
    pub fn ensure_speech_model(&mut self) -> Result<(), String> {
        if self.whisper.is_none() {
            self.whisper = Some(LocalWhisper::load(
                self.settings
                    .speech_model
                    .as_deref()
                    .ok_or("Choose a speech model.")?,
                self.settings.speech_device.unwrap_or(dictation_core::defaults::SPEECH_DEVICE),
            )?);
            let device_request = self.whisper.as_ref().map(|speech| speech.device_request);
            self.runtime.update(|s| {
                s.whisper_loaded = true;
                s.speech_device_request = device_request;
            });
        }
        self.whisper.as_mut().ok_or("Whisper was not loaded.")?.prepare()?;
        self.last_model_use = Instant::now();
        Ok(())
    }

    pub fn preload_speech_model(&mut self) -> Result<(), String> {
        if self.settings.model_idle_seconds().is_none() && self.settings.speech_model.is_some() {
            self.ensure_speech_model()?;
        }
        Ok(())
    }

    pub fn prepare_next_dictation(&mut self) -> Result<(), String> {
        if self.settings.model_idle_seconds() != Some(0) {
            if let Some(speech) = self.whisper.as_mut() {
                speech.prepare()?;
            }
        }
        Ok(())
    }

    pub fn model_idle_remaining(&self) -> Option<Duration> {
        self.whisper.as_ref()?;
        self.settings.model_idle_seconds().map(|seconds| {
            Duration::from_secs(seconds).saturating_sub(self.last_model_use.elapsed())
        })
    }

    pub fn unload_idle_model(&mut self) {
        if matches!(
            self.session.phase,
            Phase::Preparing | Phase::Recording | Phase::Transcribing | Phase::Inserting
        ) || self.wake_pending.is_some()
        {
            return;
        }
        if self.model_idle_remaining().is_some_and(|time| time.is_zero()) {
            self.whisper = None;
            self.runtime.update(|s| {
                s.whisper_loaded = false;
                s.speech_device_request = None;
            });
        }
    }
}
