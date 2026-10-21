use crate::{audio::Capture, shortcuts, speech::LocalWhisper, wake::LocalWake, worker::Worker};
use dictation_core::{defaults, session::Phase, settings::{Settings, SpeechDevice}};
use std::time::Instant;
use tauri_plugin_autostart::ManagerExt;

impl Worker {
    pub fn initialize(&mut self) {
        let gpu = crate::speech_device::detect_gpu();
        self.runtime.update(|s| s.gpu = gpu);
        let shortcut_result = shortcuts::apply(&self.runtime.app, &self.settings);
        let startup_result = self.apply_startup(&self.settings);
        let model_result = self.preload_speech_model();
        self.runtime
            .update(|s| s.shortcuts_registered = shortcut_result.is_ok());
        if let Err(error) = shortcut_result
            .and(startup_result)
            .and(model_result)
            .and_then(|_| self.listen())
        {
            self.fail(error);
        } else {
            let message = self
                .runtime
                .snapshot()
                .map(|s| s.message)
                .unwrap_or_default();
            self.publish(message);
        }
    }

    fn apply_startup(&self, settings: &Settings) -> Result<(), String> {
        let manager = self.runtime.app.autolaunch();
        if let Some(enabled) = settings.launch_at_login {
            if enabled {
                manager.enable()
            } else {
                manager.disable()
            }
            .map_err(|_| "Login startup registration failed.")?;
        }
        let actual = manager.is_enabled().ok();
        self.runtime.update(|s| s.startup_registered = actual);
        Ok(())
    }

    pub fn save(&mut self, settings: Settings) -> Result<(), String> {
        if !matches!(
            self.session.phase,
            Phase::Idle | Phase::Paused | Phase::Error
        ) || self.test.is_some()
        {
            return Err("Finish the active session or test, or discard pending text before changing settings.".into());
        }
        settings.validate()?;
        if settings.speech_device == Some(SpeechDevice::Gpu) {
            let gpu = crate::speech_device::detect_gpu();
            let unavailable = gpu.reason.clone();
            self.runtime.update(|s| s.gpu = gpu);
            if let Some(reason) = unavailable {
                return Err(reason);
            }
        }
        if settings.language.as_ref().is_some_and(|language| {
            language != "auto" && whisper_rs::get_lang_id(language).is_none()
        }) {
            return Err("The selected dictation language is unsupported.".into());
        }
        if settings.wake_enabled == Some(true) {
            settings.recording_ready()?;
        }
        let previous = self.settings.clone();
        let previous_startup = self
            .runtime
            .app
            .autolaunch()
            .is_enabled()
            .map_err(|_| "Cannot inspect the existing login startup registration.")?;
        self.capture = None;
        let prepared = (|| {
            let speech = if settings.speech_model != previous.speech_model
                || settings.speech_device != previous.speech_device
                || (settings.model_idle_seconds().is_none() && self.whisper.is_none())
            {
                settings
                    .speech_model
                    .as_ref()
                    .map(|path| LocalWhisper::load(
                        path, settings.speech_device.unwrap_or(defaults::SPEECH_DEVICE),
                    ))
                    .transpose()?
            } else {
                None
            };
            let wake = if settings.wake_enabled == Some(true) && !settings.paused {
                Some(LocalWake::load(
                    settings
                        .wake_model
                        .as_deref()
                        .ok_or("Choose a wake model.")?,
                    settings
                        .wake_phrase
                        .as_deref()
                        .ok_or("Enter a wake phrase.")?,
                )?)
            } else {
                None
            };
            if settings.stop_enabled == Some(true) {
                crate::wake::keyword_line(
                    std::path::Path::new(
                        settings
                            .wake_model
                            .as_deref()
                            .ok_or("Choose a wake model.")?,
                    ),
                    settings
                        .stop_phrase
                        .as_deref()
                        .ok_or("Enter a stop phrase.")?,
                )?;
            }
            let capture = if wake.is_some() {
                Some(Capture::open(
                    settings
                        .microphone
                        .as_deref()
                        .ok_or("Choose a microphone.")?,
                    self.ready.clone(),
                )?)
            } else {
                None
            };
            shortcuts::apply(&self.runtime.app, &settings)?;
            self.apply_startup(&settings)?;
            self.store.save(&settings)?;
            Ok::<_, String>((speech, wake, capture))
        })();
        let (speech, wake, capture) = match prepared {
            Ok(prepared) => prepared,
            Err(error) => {
                let mut restored = previous.clone();
                restored.launch_at_login = Some(previous_startup);
                let shortcut_result = shortcuts::apply(&self.runtime.app, &previous);
                self.runtime
                    .update(|s| s.shortcuts_registered = shortcut_result.is_ok());
                let rollback = shortcut_result
                    .and_then(|_| self.apply_startup(&restored))
                    .and_then(|_| self.listen());
                return Err(match rollback {
                    Ok(()) => error,
                    Err(rollback) => format!("{error} Restore failed: {rollback}"),
                });
            }
        };
        if settings.speech_model != previous.speech_model
            || settings.speech_device != previous.speech_device || speech.is_some()
        {
            self.whisper = speech;
            self.last_model_use = Instant::now();
        }
        self.settings = settings;
        self.wake = wake;
        self.capture = capture;
        self.last_audio = Instant::now();
        self.stop = None;
        self.wake_samples = 0;
        self.session.cancel(self.settings.paused);
        self.runtime.update(|s| s.shortcuts_registered = true);
        self.publish("Settings saved and applied.");
        Ok(())
    }
}
