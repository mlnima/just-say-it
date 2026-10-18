use crate::{defaults, settings::Settings};

#[derive(Default)]
pub struct SpeechActivity {
    pub total_samples: usize,
    pub voiced_samples: usize,
    pub last_voice: usize,
}

impl SpeechActivity {
    pub fn accept(&mut self, samples: &[f32], voiced: bool) {
        self.total_samples += samples.len();
        if voiced {
            self.voiced_samples += samples.len();
            self.last_voice = self.total_samples;
        }
    }

    pub fn has_speech(&self) -> bool {
        self.voiced_samples as u64 * 1000 / defaults::SAMPLE_RATE as u64
            >= defaults::MIN_SPEECH_MILLIS
    }

    pub fn finish_reason(&self, settings: &Settings, wake_started: bool) -> Option<&'static str> {
        let seconds = self.total_samples as f32 / defaults::SAMPLE_RATE as f32;
        let quiet = (self.total_samples - self.last_voice) as f32 / defaults::SAMPLE_RATE as f32;
        let silence = settings
            .silence_seconds
            .or(wake_started.then_some(defaults::WAKE_SILENCE_SECONDS));
        if seconds
            >= settings
                .recording_limit_seconds
                .unwrap_or(defaults::MAX_RECORDING_SECONDS) as f32
        {
            return Some("Recording limit reached");
        }
        if !self.has_speech()
            && seconds
                >= settings
                    .no_speech_seconds
                    .unwrap_or(defaults::NO_SPEECH_SECONDS)
        {
            return Some("No speech detected");
        }
        (self.has_speech() && silence.is_some_and(|limit| quiet >= limit))
            .then_some("Silence timeout")
    }
}
