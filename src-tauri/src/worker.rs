use crate::{
    audio::Capture,
    platform::{Focus, Target},
    runtime::{Command, Runtime},
    speech::LocalWhisper,
    store::Store,
    wake::LocalWake,
};
use crossbeam_channel::{bounded, Receiver, Sender};
use dictation_core::{
    defaults,
    session::{Phase, Session},
    settings::Settings,
    speech_activity::SpeechActivity,
};
use std::{
    collections::VecDeque,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

pub struct Worker {
    pub runtime: Runtime,
    pub store: Store,
    pub settings: Settings,
    pub session: Session,
    pub capture: Option<Capture>,
    pub wake: Option<LocalWake>,
    pub stop: Option<LocalWake>,
    pub whisper: Option<LocalWhisper>,
    pub focus: Focus,
    pub target: Option<Target>,
    pub audio: Vec<f32>,
    pub pre_roll: VecDeque<f32>,
    pub activity: SpeechActivity,
    pub wake_started: bool,
    pub wake_samples: usize,
    pub ready: Sender<()>,
    pub test: Option<(bool, Instant)>,
    pub last_meter: Instant,
    pub last_model_use: Instant,
    pub cooldown: Instant,
    pub pending_armed: bool,
    pub wake_pending: Option<usize>,
    pub stop_candidate: Option<dictation_core::ports::PhraseDetection>,
    pub stop_origin: usize,
    pub last_audio: Instant,
    pub vad: crate::voice_activity::VoiceActivity,
}

pub fn run(runtime: Runtime, commands: Receiver<Command>, store: Store, settings: Settings) {
    let focus = match Focus::new() {
        Ok(focus) => focus,
        Err(error) => {
            runtime.update(|s| {
                s.phase = Phase::Error;
                s.message = error;
            });
            return;
        }
    };
    #[cfg(windows)]
    let focus = match focus.watch_changes() {
        Ok(focus) => focus,
        Err(error) => {
            runtime.update(|s| {
                s.phase = Phase::Error;
                s.input_detection_supported = false;
                s.message = error;
            });
            return;
        }
    };
    let (ready, audio_ready) = bounded(1);
    let mut worker = Worker {
        runtime,
        store,
        settings,
        session: Session::default(),
        capture: None,
        wake: None,
        stop: None,
        whisper: None,
        focus,
        target: None,
        audio: Vec::new(),
        pre_roll: VecDeque::new(),
        activity: SpeechActivity::default(),
        wake_started: false,
        wake_samples: 0,
        ready,
        test: None,
        last_meter: Instant::now(),
        last_model_use: Instant::now(),
        cooldown: Instant::now(),
        pending_armed: false,
        wake_pending: None,
        stop_candidate: None,
        stop_origin: 0,
        last_audio: Instant::now(),
        vad: crate::voice_activity::VoiceActivity::new(),
    };
    worker.session.cancel(worker.settings.paused);
    worker.initialize();
    while !worker.runtime.stopping.load(Ordering::Acquire) {
        match worker.runtime.interrupt.swap(0, Ordering::AcqRel) {
            2 => worker.command(Command::Action(crate::runtime::Action::Pause)),
            1 => worker.command(Command::Action(crate::runtime::Action::Cancel)),
            _ => {}
        }
        crossbeam_channel::select! {
            recv(commands) -> command => match command {
                Ok(Command::Quit) | Err(_) => break,
                Ok(command) => worker.command(command),
            },
            recv(audio_ready) -> _ => worker.audio_ready(),
            default(worker.wait_duration()) => worker.maintenance(),
        }
        worker.maintenance();
    }
    worker.capture = None;
    worker.runtime.authorization.store(0, Ordering::Release);
}

impl Worker {
    pub fn publish(&self, message: impl Into<String>) {
        let message = message.into();
        let mic = self.capture.as_ref().map(|c| c.device_name.clone());
        let wake_listening = self.capture.is_some()
            && matches!(self.session.phase, Phase::Idle | Phase::PendingDelivery)
            && self.wake.is_some()
            && self.wake_pending.is_none()
            && self.test.is_none();
        self.runtime.update(|s| {
            s.phase = if self.wake_pending.is_some() {
                Phase::Preparing
            } else {
                self.session.phase
            };
            s.settings = self.settings.clone();
            s.message = message;
            s.microphone_open = self.capture.is_some();
            s.microphone_name = mic;
            s.wake_listening = wake_listening;
            s.whisper_loaded = self.whisper.is_some();
            s.speech_device_request = self.whisper.as_ref().map(|speech| speech.device_request);
            s.pending_text = self.session.recovery.is_some();
            if self.capture.is_none() {
                s.microphone_level = 0.0;
            }
        });
        let mode = if self.settings.paused {
            0
        } else if self.pending_armed {
            4
        } else if self.wake_pending.is_some() {
            3
        } else {
            match self.session.phase {
                Phase::Idle | Phase::Error | Phase::PendingDelivery
                    if self.settings.input_button == Some(true) => 1,
                Phase::Recording => 2,
                Phase::Transcribing | Phase::Inserting => 3,
                _ => 0,
            }
        };
        self.runtime.overlay.set_mode(mode);
    }

    pub fn fail(&mut self, message: String) {
        if matches!(
            self.session.phase,
            Phase::Preparing | Phase::Recording | Phase::Transcribing | Phase::Inserting
        ) {
            self.last_model_use = Instant::now();
        }
        self.capture = None;
        self.audio.clear();
        self.pre_roll.clear();
        self.runtime.authorization.store(0, Ordering::Release);
        self.session.phase = Phase::Error;
        self.publish(message);
    }

    pub fn listen(&mut self) -> Result<(), String> {
        self.capture = None;
        self.pre_roll.clear();
        self.wake_samples = 0;
        if self.settings.paused
            || self.settings.wake_enabled != Some(true)
            || !matches!(self.session.phase, Phase::Idle | Phase::PendingDelivery)
        {
            return Ok(());
        }
        self.settings.recording_ready()?;
        if self.wake.is_none() {
            self.wake = Some(LocalWake::load(
                self.settings
                    .wake_model
                    .as_deref()
                    .ok_or("Wake model not selected.")?,
                self.settings
                    .wake_phrase
                    .as_deref()
                    .ok_or("Wake phrase not set.")?,
            )?);
        }
        if let Some(wake) = &mut self.wake {
            use dictation_core::ports::WakeDetector;
            wake.reset();
        }
        self.capture = Some(Capture::open(
            self.settings
                .microphone
                .as_deref()
                .ok_or("Microphone not selected.")?,
            self.ready.clone(),
        )?);
        self.last_audio = Instant::now();
        Ok(())
    }

    fn wait_duration(&self) -> Duration {
        if self.capture.is_some()
            || self.test.is_some()
            || self.session.recovery.is_some()
            || self.wake_pending.is_some()
        {
            Duration::from_millis(defaults::MAINTENANCE_MILLIS)
        } else {
            self.model_idle_remaining()
                .unwrap_or(Duration::from_secs(86_400))
        }
    }

    fn maintenance(&mut self) {
        if self.capture.is_some()
            && self.last_audio.elapsed() > Duration::from_secs(defaults::AUDIO_STALL_SECONDS)
        {
            self.capture = None;
            if self.session.phase == Phase::Recording && self.activity.has_speech() {
                self.target = None;
                if let Err(error) = self.finish() {
                    self.fail(error);
                }
            } else {
                self.fail("The microphone stopped producing audio. Check the device and resume listening.".into());
            }
        }
        if self.test.is_some_and(|(_, end)| Instant::now() >= end) {
            self.test = None;
            if let Err(error) = self.listen() {
                self.fail(error);
            } else {
                self.publish("Test finished; microphone policy restored.");
            }
        }
        let pending = self.session.recovery.is_some();
        self.session.expire();
        if pending && self.session.recovery.is_none() {
            if let Ok(mut text) = self.runtime.recovery.lock() {
                *text = None;
            }
            self.pending_armed = false;
            let _ = self.listen();
            self.publish("Temporary recovery text expired.");
        }
        self.unload_idle_model();
    }
}
