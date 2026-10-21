use crate::{
    audio::Capture,
    runtime::{Action, Command},
    wake::LocalWake,
    worker::Worker,
};
use dictation_core::{
    defaults, ports::WakeDetector, session::Phase, speech_activity::SpeechActivity,
};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

pub enum Activation {
    InputButton,
    Wake,
    Shortcut(Option<crate::platform::ShortcutTarget>),
}

impl Worker {
    pub fn command(&mut self, command: Command) {
        let result = match command {
            Command::Interrupt => Ok(()),
            Command::Action(action) => self.action(action),
            Command::Shortcut(target) => self.toggle(Activation::Shortcut(target)),
            Command::Save(settings, reply) => {
                let result = self.save(settings);
                let _ = reply.try_send(result.clone());
                result
            }
            Command::MicrophoneTest => self.test(false),
            Command::PhraseTest => self.test(true),
            Command::Quit => Ok(()),
        };
        if let Err(error) = result {
            if self.session.phase == Phase::Transcribing {
                self.fail(error);
            } else {
                self.publish(error);
            }
        }
    }

    fn action(&mut self, action: Action) -> Result<(), String> {
        match action {
            Action::Quit | Action::Restart => Ok(()),
            Action::Toggle => self.toggle(Activation::InputButton),
            Action::Finish if self.session.phase == Phase::Recording => self.finish(),
            Action::Finish => Ok(()),
            Action::Pause | Action::Cancel => {
                let paused = matches!(action, Action::Pause) || self.settings.paused;
                self.capture = None;
                self.audio.clear();
                self.pre_roll.clear();
                self.wake_pending = None;
                self.pending_armed = false;
                self.test = None;
                if matches!(
                    self.session.phase,
                    Phase::Preparing | Phase::Recording | Phase::Transcribing | Phase::Inserting
                ) {
                    self.last_model_use = Instant::now();
                }
                self.session.cancel(paused);
                if let Ok(mut text) = self.runtime.recovery.lock() {
                    *text = None;
                }
                if paused {
                    self.settings.paused = true;
                    self.wake = None;
                    self.stop = None;
                    self.store.save(&self.settings)?;
                }
                self.cooldown = Instant::now() + Duration::from_millis(defaults::COOLDOWN_MILLIS);
                self.listen()?;
                self.publish(if paused {
                    "All listening paused. The microphone is closed; pause will survive restart."
                } else {
                    "Dictation cancelled. No pending text retained."
                });
                Ok(())
            }
            Action::Resume => {
                let mut resumed = self.settings.clone();
                resumed.paused = false;
                self.save(resumed)
            }
            Action::Discard => {
                self.session.cancel(self.settings.paused);
                self.pending_armed = false;
                if let Ok(mut text) = self.runtime.recovery.lock() {
                    *text = None;
                }
                self.listen()?;
                self.publish("Temporary text discarded.");
                Ok(())
            }
            Action::PreparePending => {
                if self.session.recovery.is_none() {
                    return Err("There is no pending text to insert.".into());
                }
                if self.settings.input_button != Some(true) {
                    return Err("Enable the input button before choosing a new destination.".into());
                }
                self.capture = None;
                self.pending_armed = true;
                self.publish("Focus the intended input, then click its Insert button. Check for existing text before inserting again.");
                Ok(())
            }
        }
    }

    fn toggle(&mut self, activation: Activation) -> Result<(), String> {
        match self.session.phase {
            _ if self.pending_armed => self.insert_pending(),
            Phase::Recording => self.finish(),
            _ => self.start(activation),
        }
    }

    pub fn start(&mut self, activation: Activation) -> Result<(), String> {
        let wake_started = matches!(activation, Activation::Wake);
        if self.settings.paused {
            return Err("Listening is paused. Resume explicitly from the tray or settings.".into());
        }
        if self.test.is_some() {
            return Err("Finish the microphone or phrase test before dictating.".into());
        }
        if wake_started && Instant::now() < self.cooldown {
            return Ok(());
        }
        self.settings.recording_ready()?;
        // Starting another dictation deliberately replaces the previous result.
        // A failed insertion must not disable the user's activation shortcut.
        if self.session.phase == Phase::PendingDelivery {
            self.session.cancel(false);
        }
        let id = self.session.start()?;
        self.runtime.authorization.store(id, Ordering::Release);
        self.target = Some(match activation {
            Activation::Shortcut(target) => self.focus.shortcut_target(target),
            _ => self.focus.inspect(),
        });
        self.wake_started = wake_started;
        self.activity = SpeechActivity::default();
        self.vad = crate::voice_activity::VoiceActivity::new();
        self.stop_candidate = None;
        self.stop_origin = 0;
        if let Ok(mut text) = self.runtime.recovery.lock() {
            *text = None;
        }
        self.audio = Vec::with_capacity(
            self.settings
                .recording_limit_seconds
                .unwrap_or(defaults::MAX_RECORDING_SECONDS) as usize
                * defaults::SAMPLE_RATE as usize,
        );
        self.pre_roll.clear();
        self.wake_pending = None;
        self.publish("Preparing the local speech model. Speak when recording begins.");
        let result = (|| {
            self.ensure_speech_model()?;
            self.stop = if self.settings.stop_enabled == Some(true) {
                Some(LocalWake::load(
                    self.settings
                        .wake_model
                        .as_deref()
                        .ok_or("Choose a wake model for voice stopping.")?,
                    self.settings
                        .stop_phrase
                        .as_deref()
                        .ok_or("Set a stop phrase.")?,
                )?)
            } else {
                None
            };
            if self.capture.is_none() {
                self.capture = Some(Capture::open(
                    self.settings
                        .microphone
                        .as_deref()
                        .ok_or("Choose a microphone.")?,
                    self.ready.clone(),
                )?);
            }
            Ok::<_, String>(())
        })();
        if let Err(error) = result {
            self.fail(error.clone());
            return Err(error);
        }
        if let Some(stop) = &mut self.stop {
            stop.reset();
        }
        self.session.phase = Phase::Recording;
        self.last_audio = Instant::now();
        self.publish(if self.target.as_ref().is_some_and(|target| target.window_only()) {
            "Recording for the window selected by your shortcut. Its text field cannot be inspected; keep the intended input open and use the shortcut again to finish."
        } else {
            "Recording. Use Finish in the tray, your toggle shortcut, or an enabled stopping rule."
        });
        Ok(())
    }

    fn test(&mut self, phrase: bool) -> Result<(), String> {
        if self.settings.paused {
            return Err("Resume listening before starting a microphone test.".into());
        }
        if !matches!(self.session.phase, Phase::Idle | Phase::Error) || self.test.is_some() {
            return Err("Finish the active session or test first.".into());
        }
        self.capture = None;
        if phrase {
            self.wake = Some(LocalWake::load(
                self.settings
                    .wake_model
                    .as_deref()
                    .ok_or("Save a wake model first.")?,
                self.settings
                    .wake_phrase
                    .as_deref()
                    .ok_or("Save a wake phrase first.")?,
            )?);
        }
        self.capture = Some(Capture::open(
            self.settings
                .microphone
                .as_deref()
                .ok_or("Save a microphone first.")?,
            self.ready.clone(),
        )?);
        self.test = Some((
            phrase,
            Instant::now() + Duration::from_secs(defaults::TEST_SECONDS),
        ));
        self.last_audio = Instant::now();
        self.publish(if phrase { "Phrase test: say the saved phrase. Each detection is counted; dictation will not start." } else { "Microphone level test active for 15 seconds. No speech recognition is running." });
        Ok(())
    }
}
