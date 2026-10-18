export type Settings = {
  schema_version: number;
  microphone: string | null;
  speech_model: string | null;
  language: string | null;
  wake_enabled: boolean | null;
  wake_phrase: string | null;
  wake_model: string | null;
  stop_enabled: boolean | null;
  stop_phrase: string | null;
  input_button: boolean | null;
  launch_at_login: boolean | null;
  toggle_shortcut: string | null;
  cancel_shortcut: string | null;
  silence_seconds: number | null;
  no_speech_seconds: number | null;
  recording_limit_seconds: number | null;
  paused: boolean;
};

export type Phase = 'idle' | 'preparing' | 'recording' | 'transcribing' | 'inserting' | 'pending_delivery' | 'paused' | 'error';
export type Action = 'toggle' | 'finish' | 'cancel' | 'pause' | 'resume' | 'discard' | 'prepare_pending' | 'restart' | 'quit';
export type Microphone = { id: string; label: string };
export type Model = {
  id: string;
  kind: 'speech' | 'wake';
  label: string;
  bytes: number;
  license: string;
  source: string;
  languages: string;
  installed_path: string | null;
};

export type Snapshot = {
  revision: number;
  app_name: string;
  settings: Settings;
  phase: Phase;
  message: string;
  microphone_open: boolean;
  microphone_name: string | null;
  microphone_level: number;
  wake_listening: boolean;
  whisper_loaded: boolean;
  inference_runs: number;
  phrase_detections: number;
  target_status: string;
  input_detection_supported: boolean;
  startup_registered: boolean | null;
  shortcuts_registered: boolean;
  pending_text: boolean;
  download: { id: string; received: number; total: number } | null;
  defaults: {
    silence_seconds: number;
    no_speech_seconds: number;
    recording_limit_seconds: number;
    recovery_seconds: number;
    inference_threads: number;
  };
  languages: { id: string; label: string }[];
};

export type CommandMap = {
  get_snapshot: { args: undefined; result: Snapshot };
  save_settings: { args: { settings: Settings }; result: void };
  list_microphones: { args: undefined; result: Microphone[] };
  control: { args: { action: Action }; result: void };
  get_recovery: { args: undefined; result: string | null };
  get_models: { args: undefined; result: Model[] };
  install_model: { args: { id: string }; result: void };
  cancel_download: { args: undefined; result: void };
  import_model: { args: { kind: Model['kind'] }; result: string | null };
  test_microphone: { args: undefined; result: void };
  test_phrase: { args: undefined; result: void };
};
