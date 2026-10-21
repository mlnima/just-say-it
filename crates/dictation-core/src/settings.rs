use crate::defaults;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ModelRetention {
    IdleSeconds(u64),
    Never,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeechDevice {
    Cpu,
    Gpu,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub schema_version: u32,
    pub microphone: Option<String>,
    pub speech_model: Option<String>,
    pub speech_device: Option<SpeechDevice>,
    pub model_retention: ModelRetention,
    pub language: Option<String>,
    pub wake_enabled: Option<bool>,
    pub wake_phrase: Option<String>,
    pub wake_model: Option<String>,
    pub stop_enabled: Option<bool>,
    pub stop_phrase: Option<String>,
    pub input_button: Option<bool>,
    pub screen_overlay: Option<bool>,
    pub launch_at_login: Option<bool>,
    pub toggle_shortcut: Option<String>,
    pub silence_seconds: Option<f32>,
    pub no_speech_seconds: Option<f32>,
    pub recording_limit_seconds: Option<u32>,
    pub paused: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: defaults::SCHEMA_VERSION,
            microphone: None,
            speech_model: None,
            speech_device: None,
            model_retention: defaults::MODEL_RETENTION,
            language: None,
            wake_enabled: None,
            wake_phrase: None,
            wake_model: None,
            stop_enabled: None,
            stop_phrase: None,
            input_button: None,
            screen_overlay: None,
            launch_at_login: None,
            toggle_shortcut: None,
            silence_seconds: None,
            no_speech_seconds: None,
            recording_limit_seconds: None,
            paused: false,
        }
    }
}

pub fn validate_phrase(phrase: &str) -> Result<String, String> {
    let normalized = phrase.split_whitespace().collect::<Vec<_>>().join(" ");
    if !(2..=80).contains(&normalized.len())
        || !normalized
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c == ' ' || c == '\'')
    {
        return Err("The installed wake model supports English phrases of 2–80 letters, spaces and apostrophes.".into());
    }
    Ok(normalized.to_uppercase())
}

impl Settings {
    pub fn model_idle_seconds(&self) -> Option<u64> {
        match self.model_retention {
            ModelRetention::Never => None,
            ModelRetention::IdleSeconds(seconds) => Some(seconds),
        }
    }

    pub fn toggle_shortcut(&self) -> &str {
        self.toggle_shortcut
            .as_deref()
            .unwrap_or(defaults::TOGGLE_SHORTCUT)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self
            .model_idle_seconds()
            .is_some_and(|seconds| !defaults::MODEL_IDLE_OPTIONS_SECONDS.contains(&seconds))
        {
            return Err("Choose a supported speech model idle timeout.".into());
        }
        if self.schema_version != defaults::SCHEMA_VERSION {
            return Err(
                "Unsupported settings version; the original file has been preserved.".into(),
            );
        }
        for text in [
            &self.microphone,
            &self.speech_model,
            &self.language,
            &self.wake_model,
            &self.toggle_shortcut,
        ]
        .into_iter()
        .flatten()
        {
            if text.trim().is_empty() || text.len() > 4096 {
                return Err("Clear an unset field instead of saving an empty value.".into());
            }
            if text.contains('\0') {
                return Err("Settings cannot contain a null character.".into());
            }
        }
        for phrase in [&self.wake_phrase, &self.stop_phrase].into_iter().flatten() {
            validate_phrase(phrase)?;
        }
        if let (Some(wake), Some(stop)) = (&self.wake_phrase, &self.stop_phrase) {
            let (wake, stop) = (validate_phrase(wake)?, validate_phrase(stop)?);
            if wake.contains(&stop) || stop.contains(&wake) {
                return Err(
                    "Wake and stop phrases must be distinct and must not contain one another."
                        .into(),
                );
            }
        }
        for (value, minimum, maximum) in [
            (self.silence_seconds, 0.5, 30.0),
            (self.no_speech_seconds, 1.0, 30.0),
        ] {
            if value.is_some_and(|n| !n.is_finite() || !(minimum..=maximum).contains(&n)) {
                return Err(
                    "Silence must be 0.5–30 seconds; no-speech timeout must be 1–30 seconds."
                        .into(),
                );
            }
        }
        if self
            .recording_limit_seconds
            .is_some_and(|n| !(5..=120).contains(&n))
        {
            return Err("Recording limit must be between 5 and 120 seconds.".into());
        }
        if self.wake_enabled == Some(true)
            && (self.wake_phrase.is_none() || self.wake_model.is_none())
        {
            return Err(
                "Choose a wake model and a phrase before enabling voice activation.".into(),
            );
        }
        if self.stop_enabled == Some(true)
            && (self.stop_phrase.is_none() || self.wake_model.is_none())
        {
            return Err("Voice stopping needs a stop phrase and a wake model.".into());
        }
        Ok(())
    }

    pub fn recording_ready(&self) -> Result<(), String> {
        self.validate()?;
        if self.microphone.is_none() || self.speech_model.is_none() || self.language.is_none() {
            return Err(
                "Choose and save a microphone, a Whisper model and a dictation language.".into(),
            );
        }
        Ok(())
    }
}
