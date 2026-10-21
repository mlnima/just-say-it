use crate::{
    audio, downloads, models,
    runtime::{Action, Command, Runtime},
    runtime_status::Snapshot,
};
use crossbeam_channel::bounded;
use dictation_core::settings::Settings;
use std::sync::atomic::Ordering;
use tauri::State;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn get_snapshot(runtime: State<'_, Runtime>) -> Result<Snapshot, String> {
    runtime.snapshot()
}

#[tauri::command]
pub async fn list_microphones() -> Result<Vec<audio::Microphone>, String> {
    tauri::async_runtime::spawn_blocking(audio::devices)
        .await
        .map_err(|_| "Microphone discovery worker failed.")?
}

#[tauri::command]
pub async fn save_settings(runtime: State<'_, Runtime>, settings: Settings) -> Result<(), String> {
    let commands = runtime.commands.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (reply, result) = bounded(1);
        commands
            .try_send(Command::Save(settings, reply))
            .map_err(|_| "Settings queue is busy.")?;
        result
            .recv_timeout(std::time::Duration::from_secs(120))
            .map_err(|_| "Settings apply timed out. Reopen settings to inspect actual state.")?
    })
    .await
    .map_err(|_| "Settings worker failed.")?
}

#[tauri::command]
pub fn control(runtime: State<'_, Runtime>, action: Action) -> Result<(), String> {
    runtime.send(action)
}

#[tauri::command]
pub fn get_recovery(runtime: State<'_, Runtime>) -> Result<Option<String>, String> {
    runtime
        .recovery
        .lock()
        .map(|text| text.clone())
        .map_err(|_| "Recovery text is unavailable.".into())
}

#[tauri::command]
pub fn get_models(runtime: State<'_, Runtime>) -> Result<Vec<models::ModelEntry>, String> {
    models::list(&runtime.model_root)
}

#[tauri::command]
pub fn install_model(runtime: State<'_, Runtime>, id: String) -> Result<(), String> {
    downloads::install(&runtime, &id)
}

#[tauri::command]
pub fn cancel_download(runtime: State<'_, Runtime>) {
    runtime.cancel_download.store(true, Ordering::Release);
}

#[tauri::command]
pub async fn import_model(app: tauri::AppHandle, kind: String) -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dialog = app.dialog().file();
        let selected = match kind.as_str() {
            "speech" => dialog
                .add_filter("whisper.cpp GGML model", &["bin"])
                .blocking_pick_file(),
            "wake" => dialog.blocking_pick_folder(),
            _ => return Err("Unknown model kind.".into()),
        };
        let Some(selected) = selected else {
            return Ok(None);
        };
        let path = selected
            .into_path()
            .map_err(|_| "Only local model files can be imported.")?;
        let path = path
            .to_str()
            .ok_or("Model path is not valid Unicode.")?
            .to_owned();
        models::validate(&kind, &path)?;
        Ok(Some(path))
    })
    .await
    .map_err(|_| "Model import worker failed.")?
}

#[tauri::command]
pub fn test_microphone(runtime: State<'_, Runtime>) -> Result<(), String> {
    runtime
        .commands
        .try_send(Command::MicrophoneTest)
        .map_err(|_| "Test queue is busy.".into())
}

#[tauri::command]
pub fn test_phrase(runtime: State<'_, Runtime>) -> Result<(), String> {
    runtime
        .commands
        .try_send(Command::PhraseTest)
        .map_err(|_| "Test queue is busy.".into())
}
