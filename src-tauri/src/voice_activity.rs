use dictation_core::defaults;
use webrtc_vad::{SampleRate, Vad, VadMode};

pub struct VoiceActivity {
    detector: Vad,
    frame: [i16; defaults::FRAME_SAMPLES],
    filled: usize,
    last_decision: bool,
}

impl VoiceActivity {
    pub fn new() -> Self {
        Self {
            detector: Vad::new_with_rate_and_mode(SampleRate::Rate16kHz, VadMode::Aggressive),
            frame: [0; defaults::FRAME_SAMPLES],
            filled: 0,
            last_decision: false,
        }
    }

    pub fn accept(&mut self, samples: &[f32]) -> Result<bool, String> {
        let mut voiced = false;
        for sample in samples {
            self.frame[self.filled] = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            self.filled += 1;
            if self.filled == self.frame.len() {
                self.last_decision = self
                    .detector
                    .is_voice_segment(&self.frame)
                    .map_err(|_| "Speech activity detection rejected the audio format.")?;
                self.filled = 0;
                voiced |= self.last_decision;
            }
        }
        Ok(voiced || self.last_decision)
    }
}
