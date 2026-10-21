use crate::{worker::Worker, worker_control::Activation};
use dictation_core::{defaults, ports::WakeDetector, session::Phase};
use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

fn quiet(samples: &[f32]) -> bool {
    samples.is_empty()
        || samples.iter().map(|n| n * n).sum::<f32>() / (samples.len() as f32)
            < defaults::SPEECH_RMS.powi(2)
}

impl Worker {
    pub fn audio_ready(&mut self) {
        let samples = match self.capture.as_mut().map(|capture| capture.drain()) {
            Some(Ok(samples)) => samples,
            Some(Err(error)) => {
                if self.session.phase == Phase::Recording && self.activity.has_speech() {
                    self.capture = None;
                    self.target = None;
                    self.publish(error);
                    if let Err(error) = self.finish() {
                        self.fail(error);
                    }
                    return;
                }
                self.fail(error);
                return;
            }
            None => return,
        };
        if !samples.is_empty() {
            self.last_audio = Instant::now();
        }
        if self.test.is_some()
            && self.last_meter.elapsed() >= Duration::from_millis(defaults::METER_MILLIS)
        {
            let level =
                (samples.iter().map(|n| n * n).sum::<f32>() / samples.len().max(1) as f32).sqrt();
            self.runtime.update(|s| s.microphone_level = level);
            self.last_meter = Instant::now();
        }
        for frame in samples.chunks(defaults::FRAME_SAMPLES) {
            if self.runtime.authorization.load(Ordering::Acquire) != self.session.id
                && self.session.phase == Phase::Recording
            {
                return;
            }
            if self.test.is_some_and(|(phrase, _)| phrase) {
                if self
                    .wake
                    .as_mut()
                    .and_then(|wake| wake.accept(frame))
                    .is_some()
                {
                    self.runtime.update(|s| {
                        s.phrase_detections += 1;
                        s.message =
                            "Saved phrase detected. The test does not start dictation.".into();
                    });
                    if let Some(wake) = &mut self.wake {
                        wake.reset();
                    }
                }
                continue;
            }
            if self.test.is_some() {
                continue;
            }
            if self.session.phase == Phase::Recording {
                self.audio.extend_from_slice(frame);
                let voiced = match self.vad.accept(frame) {
                    Ok(voiced) => voiced,
                    Err(error) => {
                        self.fail(error);
                        return;
                    }
                };
                self.activity.accept(frame, voiced);
                if self.stop_boundary(frame) {
                    return;
                }
                if let Some(reason) = self
                    .activity
                    .finish_reason(&self.settings, self.wake_started)
                {
                    self.publish(reason);
                    if let Err(error) = self.finish() {
                        self.fail(error);
                    }
                    return;
                }
            } else if matches!(self.session.phase, Phase::Idle | Phase::PendingDelivery)
                && Instant::now() >= self.cooldown
            {
                self.pre_roll.extend(frame);
                while self.pre_roll.len()
                    > defaults::PRE_ROLL_SECONDS * defaults::SAMPLE_RATE as usize
                {
                    self.pre_roll.pop_front();
                }
                self.wake_samples += frame.len();
                if let Some(quiet_samples) = &mut self.wake_pending {
                    *quiet_samples = if quiet(frame) {
                        *quiet_samples + frame.len()
                    } else {
                        0
                    };
                    if *quiet_samples as f32 / defaults::SAMPLE_RATE as f32
                        >= defaults::COMMAND_BOUNDARY_SECONDS
                    {
                        if let Err(error) = self.start(Activation::Wake) {
                            self.fail(error);
                        }
                    }
                    if self.wake_pending.is_some()
                        && self.wake_samples as f32 / defaults::SAMPLE_RATE as f32
                            >= defaults::NO_SPEECH_SECONDS
                    {
                        self.wake_pending = None;
                        if let Some(wake) = &mut self.wake {
                            wake.reset();
                        }
                        self.publish("No clear pause followed the wake phrase. Say it again, pause briefly, then dictate.");
                    }
                } else if self
                    .wake
                    .as_mut()
                    .and_then(|wake| wake.accept(frame))
                    .is_some()
                {
                    self.wake_pending = Some(0);
                    self.wake_samples = 0;
                    self.runtime.update(|s| s.phrase_detections += 1);
                    self.publish(
                        "Wake phrase heard. Pause briefly, then speak when the button says Finish.",
                    );
                }
            }
        }
    }

    fn stop_boundary(&mut self, frame: &[f32]) -> bool {
        if let Some(mut detected) = self.stop.as_mut().and_then(|stop| stop.accept(frame)) {
            let offset = self.stop_origin as f32 / defaults::SAMPLE_RATE as f32;
            detected.start_seconds += offset;
            detected.end_seconds += offset;
            if self.stop_candidate.is_none() {
                self.stop_candidate = Some(detected);
            }
            if let Some(stop) = &mut self.stop {
                stop.reset();
            }
            self.stop_origin = self.audio.len();
        }
        let Some(candidate) = &self.stop_candidate else {
            return false;
        };
        let start = (candidate.start_seconds * defaults::SAMPLE_RATE as f32) as usize;
        let end = ((candidate.end_seconds + defaults::COMMAND_TOKEN_TAIL_SECONDS)
            * defaults::SAMPLE_RATE as f32) as usize;
        let gap = (defaults::COMMAND_BOUNDARY_SECONDS * defaults::SAMPLE_RATE as f32) as usize;
        if self.audio.len() < end + gap {
            return false;
        }
        let start = start.min(self.audio.len());
        let isolated = quiet(&self.audio[start.saturating_sub(gap)..start])
            && quiet(&self.audio[end..end + gap]);
        self.stop_candidate = None;
        if !isolated {
            return false;
        }
        self.capture = None;
        self.audio.truncate(start);
        self.publish("Isolated stop phrase detected.");
        if let Err(error) = self.finish() {
            self.fail(error);
        }
        true
    }
}
