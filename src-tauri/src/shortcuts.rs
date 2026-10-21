use crate::runtime::Runtime;
use dictation_core::{defaults, settings::Settings};
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub fn plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            let runtime = app.state::<Runtime>();
            let Ok(snapshot) = runtime.snapshot() else {
                return;
            };
            if snapshot
                .settings
                .toggle_shortcut()
                .parse::<Shortcut>()
                .is_ok_and(|configured| configured.id() == shortcut.id())
            {
                let _ = runtime.send_shortcut();
            }
        })
        .build()
}

pub fn apply(app: &tauri::AppHandle, settings: &Settings) -> Result<(), String> {
    let shortcut = settings
        .toggle_shortcut()
        .parse::<Shortcut>()
        .map_err(|_| {
            format!(
                "Invalid shortcut. Use a chord such as {}.",
                defaults::TOGGLE_SHORTCUT
            )
        })?;
    if shortcut.key == tauri_plugin_global_shortcut::Code::Escape && shortcut.mods.is_empty() {
        return Err("Use a modifier chord instead of globally reserving plain Escape.".into());
    }
    app.global_shortcut()
        .unregister_all()
        .map_err(|_| "Existing shortcuts could not be released.")?;
    app.global_shortcut()
        .register(shortcut)
        .map_err(|_| "Shortcut registration was rejected or conflicts with another application.")?;
    Ok(())
}
