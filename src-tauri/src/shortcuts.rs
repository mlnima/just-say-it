use crate::runtime::{Action, Runtime};
use dictation_core::settings::Settings;
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
            let action = if snapshot
                .settings
                .cancel_shortcut
                .as_ref()
                .is_some_and(|s| s.parse::<Shortcut>().is_ok_and(|s| s.id() == shortcut.id()))
            {
                Some(Action::Cancel)
            } else if snapshot
                .settings
                .toggle_shortcut
                .as_ref()
                .is_some_and(|s| s.parse::<Shortcut>().is_ok_and(|s| s.id() == shortcut.id()))
            {
                Some(Action::Toggle)
            } else {
                None
            };
            if let Some(action) = action {
                let _ = runtime.send(action);
            }
        })
        .build()
}

pub fn apply(app: &tauri::AppHandle, settings: &Settings) -> Result<(), String> {
    let shortcuts: Result<Vec<Shortcut>, _> =
        [&settings.toggle_shortcut, &settings.cancel_shortcut]
            .into_iter()
            .flatten()
            .map(|s| s.parse::<Shortcut>())
            .collect();
    let shortcuts =
        shortcuts.map_err(|_| "Invalid shortcut. Use a chord such as Control+Shift+Space.")?;
    if shortcuts.iter().any(|shortcut| {
        shortcut.key == tauri_plugin_global_shortcut::Code::Escape && shortcut.mods.is_empty()
    }) {
        return Err("Use a modifier chord instead of globally reserving plain Escape.".into());
    }
    if shortcuts.len() == 2 && shortcuts[0].id() == shortcuts[1].id() {
        return Err("Toggle and cancel shortcuts resolve to the same chord.".into());
    }
    app.global_shortcut()
        .unregister_all()
        .map_err(|_| "Existing shortcuts could not be released.")?;
    for shortcut in shortcuts {
        app.global_shortcut().register(shortcut).map_err(|_| {
            "Shortcut registration was rejected or conflicts with another application."
        })?;
    }
    Ok(())
}
