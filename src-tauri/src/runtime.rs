use crate::{platform::OverlayHandle, store::Store};
use crossbeam_channel::{bounded, Sender};
use dictation_core::{defaults, session::Phase, settings::Settings};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
        Arc, Mutex,
    },
};
use tauri::{Emitter, Manager};

#[derive(Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Toggle,
    Finish,
    Cancel,
    Pause,
    Resume,
    Discard,
    PreparePending,
    Restart,
    Quit,
}

pub enum Command {
    Interrupt,
    Action(Action),
    Save(Settings, Sender<Result<(), String>>),
    MicrophoneTest,
    PhraseTest,
    Quit,
}

#[derive(Clone, Serialize)]
pub struct Download {
    pub id: String,
    pub received: u64,
    pub total: u64,
}

#[derive(Clone, Serialize)]
pub struct OperationalDefaults {
    pub silence_seconds: f32,
    pub no_speech_seconds: f32,
    pub recording_limit_seconds: u32,
    pub recovery_seconds: u64,
    pub inference_threads: i32,
}

#[derive(Clone, Serialize)]
pub struct Snapshot {
    pub revision: u64,
    pub app_name: String,
    pub settings: Settings,
    pub phase: Phase,
    pub message: String,
    pub microphone_open: bool,
    pub microphone_name: Option<String>,
    pub microphone_level: f32,
    pub wake_listening: bool,
    pub whisper_loaded: bool,
    pub inference_runs: u64,
    pub phrase_detections: u64,
    pub target_status: String,
    pub input_detection_supported: bool,
    pub startup_registered: Option<bool>,
    pub shortcuts_registered: bool,
    pub pending_text: bool,
    pub download: Option<Download>,
    pub defaults: OperationalDefaults,
    pub languages: Vec<Language>,
}

#[derive(Clone, Serialize)]
pub struct Language {
    pub id: String,
    pub label: String,
}

#[derive(Clone)]
pub struct Runtime {
    pub app: tauri::AppHandle,
    pub commands: Sender<Command>,
    pub status: Arc<Mutex<Snapshot>>,
    pub authorization: Arc<AtomicU64>,
    pub stopping: Arc<AtomicBool>,
    pub interrupt: Arc<AtomicU8>,
    pub tray_available: Arc<AtomicBool>,
    pub downloading: Arc<AtomicBool>,
    pub cancel_download: Arc<AtomicBool>,
    pub recovery: Arc<Mutex<Option<String>>>,
    pub model_root: PathBuf,
    pub overlay: OverlayHandle,
}

impl Runtime {
    pub fn start(app: tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let (commands, receiver) = bounded(defaults::CONTROL_CAPACITY);
        let test_directory = cfg!(debug_assertions)
            .then(|| std::env::var_os("DICTATION_TEST_DIRECTORY"))
            .flatten()
            .map(PathBuf::from);
        let config_directory = test_directory
            .clone()
            .unwrap_or(app.path().app_config_dir()?);
        let (store, settings) = Store::open(&config_directory);
        let message = store.load_error.clone().unwrap_or_else(|| {
            "Open Settings from the tray to choose a microphone and install your local models."
                .into()
        });
        let mut languages = vec![Language {
            id: "auto".into(),
            label: "Detect language automatically".into(),
        }];
        languages.extend((0..=whisper_rs::get_lang_max_id()).filter_map(|id| {
            Some(Language {
                id: whisper_rs::get_lang_str(id)?.into(),
                label: whisper_rs::get_lang_str_full(id)?.into(),
            })
        }));
        let status = Snapshot {
            revision: 0,
            app_name: app.config().product_name.clone().unwrap_or_default(),
            phase: if settings.paused {
                Phase::Paused
            } else {
                Phase::Idle
            },
            settings: settings.clone(),
            message,
            microphone_open: false,
            microphone_name: None,
            microphone_level: 0.0,
            wake_listening: false,
            whisper_loaded: false,
            inference_runs: 0,
            phrase_detections: 0,
            target_status: "No supported editable input detected".into(),
            input_detection_supported: cfg!(windows),
            startup_registered: None,
            shortcuts_registered: false,
            pending_text: false,
            download: None,
            defaults: OperationalDefaults {
                silence_seconds: defaults::WAKE_SILENCE_SECONDS,
                no_speech_seconds: defaults::NO_SPEECH_SECONDS,
                recording_limit_seconds: defaults::MAX_RECORDING_SECONDS,
                recovery_seconds: defaults::RECOVERY_SECONDS,
                inference_threads: defaults::INFERENCE_THREADS,
            },
            languages,
        };
        let runtime = Self {
            model_root: test_directory
                .unwrap_or(app.path().app_data_dir()?)
                .join("models"),
            app,
            commands,
            status: Arc::new(Mutex::new(status)),
            authorization: Arc::new(AtomicU64::new(0)),
            stopping: Arc::new(AtomicBool::new(false)),
            interrupt: Arc::new(AtomicU8::new(0)),
            tray_available: Arc::new(AtomicBool::new(false)),
            downloading: Arc::new(AtomicBool::new(false)),
            cancel_download: Arc::new(AtomicBool::new(false)),
            recovery: Arc::new(Mutex::new(None)),
            overlay: OverlayHandle::default(),
        };
        crate::platform::start(runtime.clone());
        let worker_runtime = runtime.clone();
        std::thread::Builder::new()
            .name("dictation".into())
            .spawn(move || crate::worker::run(worker_runtime, receiver, store, settings))?;
        Ok(runtime)
    }

    pub fn snapshot(&self) -> Result<Snapshot, String> {
        self.status
            .lock()
            .map(|s| s.clone())
            .map_err(|_| "Runtime status is unavailable.".into())
    }

    pub fn update(&self, edit: impl FnOnce(&mut Snapshot)) {
        let snapshot = self.status.lock().ok().map(|mut status| {
            edit(&mut status);
            status.revision += 1;
            status.clone()
        });
        if let Some(snapshot) = snapshot {
            let _ = self
                .app
                .emit_to(defaults::SETTINGS_WINDOW, defaults::STATUS_EVENT, snapshot);
        }
    }

    pub fn send(&self, action: Action) -> Result<(), String> {
        if matches!(action, Action::Quit | Action::Restart) {
            self.shutdown();
            if matches!(action, Action::Restart) {
                self.app.request_restart();
            } else {
                self.app.exit(0);
            }
            return Ok(());
        }
        if matches!(action, Action::Toggle | Action::Finish)
            && self.snapshot().is_ok_and(|s| {
                matches!(
                    s.phase,
                    Phase::Preparing | Phase::Transcribing | Phase::Inserting
                )
            })
        {
            return Ok(());
        }
        if matches!(action, Action::Cancel | Action::Pause) {
            self.authorization.store(0, Ordering::Release);
            self.interrupt.fetch_max(
                if matches!(action, Action::Pause) {
                    2
                } else {
                    1
                },
                Ordering::AcqRel,
            );
            let _ = self.commands.try_send(Command::Interrupt);
            return Ok(());
        }
        self.commands
            .try_send(Command::Action(action))
            .map_err(|_| "The control queue is busy. Try again.".into())
    }

    pub fn shutdown(&self) {
        self.authorization.store(0, Ordering::Release);
        self.stopping.store(true, Ordering::Release);
        self.cancel_download.store(true, Ordering::Release);
        self.overlay.stop();
        let _ = self.commands.try_send(Command::Quit);
    }
}
