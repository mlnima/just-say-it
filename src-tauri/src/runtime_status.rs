use crate::{runtime::Download, speech_device::GpuAvailability};
use dictation_core::{session::Phase, settings::{Settings, SpeechDevice}};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct OperationalDefaults {
    pub toggle_shortcut: &'static str,
    pub silence_seconds: f32,
    pub no_speech_seconds: f32,
    pub recording_limit_seconds: u32,
    pub recovery_seconds: u64,
    pub input_button_dismiss_seconds: u64,
    pub inference_threads: i32,
    pub speech_device: SpeechDevice,
    pub model_idle_options_seconds: &'static [u64],
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
    pub gpu: GpuAvailability,
    pub speech_device_request: Option<SpeechDevice>,
    pub inference_runs: u64,
    pub phrase_detections: u64,
    pub target_status: String,
    pub input_detection_supported: bool,
    pub screen_overlay_available: bool,
    pub screen_overlay_error: Option<String>,
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
