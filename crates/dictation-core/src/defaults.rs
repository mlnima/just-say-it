// Operational fallbacks live here; missing settings remain missing in the UI.
use crate::settings::{ModelRetention, SpeechDevice};

pub const SCHEMA_VERSION: u32 = 4;
pub const SPEECH_DEVICE: SpeechDevice = SpeechDevice::Cpu;
pub const TOGGLE_SHORTCUT: &str = "Control+Shift+Space";
pub const SAMPLE_RATE: u32 = 16_000;
pub const AUDIO_BUFFER_SECONDS: usize = 2;
pub const FRAME_SAMPLES: usize = 320;
pub const MAX_RECORDING_SECONDS: u32 = 60;
pub const NO_SPEECH_SECONDS: f32 = 5.0;
pub const WAKE_SILENCE_SECONDS: f32 = 2.0;
pub const RECOVERY_SECONDS: u64 = 300;
pub const MODEL_RETENTION: ModelRetention = ModelRetention::Never;
pub const MODEL_IDLE_OPTIONS_SECONDS: &[u64] = &[
    0,
    15 * 60,
    30 * 60,
    60 * 60,
    2 * 60 * 60,
    3 * 60 * 60,
    4 * 60 * 60,
    5 * 60 * 60,
    8 * 60 * 60,
    12 * 60 * 60,
    24 * 60 * 60,
];
pub const COOLDOWN_MILLIS: u64 = 900;
pub const INSERTION_KEY_RELEASE_MILLIS: u64 = 1_500;
pub const INSERTION_KEY_POLL_MILLIS: u64 = 10;
pub const INFERENCE_THREADS: i32 = 4;
pub const WAKE_THREADS: i32 = 1;
pub const WAKE_THRESHOLD: f32 = 0.35;
pub const WAKE_SCORE: f32 = 1.5;
pub const WAKE_TRAILING_BLANKS: i32 = 2;
pub const PRE_ROLL_SECONDS: usize = 2;
pub const COMMAND_BOUNDARY_SECONDS: f32 = 0.35;
pub const SPEECH_RMS: f32 = 0.008;
pub const MIN_SPEECH_MILLIS: u64 = 180;
pub const CONTROL_CAPACITY: usize = 32;
pub const MAX_TRANSCRIPT_CHARS: usize = 16_000;
pub const SYSTEM_DEFAULT_DEVICE: &str = "system_default";
pub const STATUS_EVENT: &str = "runtime-status";
pub const SETTINGS_WINDOW: &str = "settings";
pub const TRAY_MENU_VERTICAL_PADDING: i32 = 16;
pub const DOWNLOAD_TIMEOUT_SECONDS: u64 = 900;
pub const MAX_MODEL_BYTES: u64 = 200_000_000;
pub const OVERLAY_WIDTH: i32 = 108;
pub const OVERLAY_DISMISS_WIDTH: i32 = 32;
pub const OVERLAY_DISMISS_SECONDS: u64 = 15 * 60;
pub const OVERLAY_DISMISS_CAPACITY: usize = 32;
pub const OVERLAY_HEIGHT: i32 = 34;
pub const OVERLAY_GAP: i32 = 6;
pub const SCREEN_GLOW_WIDTH: i32 = 28;
pub const SCREEN_GLOW_ALPHA: u8 = 176;
pub const SCREEN_RECORDING_COLOR: [u8; 3] = [250, 90, 108];
pub const SCREEN_PROCESSING_COLOR: [u8; 3] = [95, 177, 255];
pub const SCREEN_LABEL_WIDTH: i32 = 420;
pub const SCREEN_LABEL_HEIGHT: i32 = 44;
pub const SCREEN_LABEL_GAP: i32 = 24;
pub const SCREEN_LABEL_FONT_SIZE: i32 = 14;
pub const SCREEN_LABEL_BACKGROUND: [u8; 3] = [18, 23, 34];
pub const SCREEN_LABEL_ALPHA: u8 = 238;
pub const TEST_SECONDS: u64 = 15;
pub const METER_MILLIS: u64 = 200;
pub const MAINTENANCE_MILLIS: u64 = 250;
pub const ACCESSIBILITY_TIMEOUT_MILLIS: u32 = 300;
pub const ACCESSIBILITY_MAX_ANCESTORS: usize = 64;
pub const COMMAND_TOKEN_TAIL_SECONDS: f32 = 0.16;
pub const MAX_NO_SPEECH_PROBABILITY: f32 = 0.6;
pub const INFERENCE_TIMEOUT_SECONDS: u64 = 120;
pub const AUDIO_STALL_SECONDS: u64 = 3;
