#[cfg(test)]
mod adapter_tests;
mod audio;
mod commands;
#[cfg(test)]
mod desktop_tests;
mod downloads;
#[cfg(test)]
mod fixture_focus;
mod models;
mod platform;
mod runtime;
mod shortcuts;
mod speech;
mod speech_cancel;
mod store;
mod tray;
mod voice_activity;
mod wake;
mod worker;
mod worker_audio;
mod worker_control;
mod worker_delivery;
mod worker_settings;

use dictation_core::defaults;
use tauri::Manager;

pub fn show_settings(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(defaults::SETTINGS_WINDOW) {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let title = app.config().product_name.clone().unwrap_or_default();
    let result = tauri::WebviewWindowBuilder::new(
        app,
        defaults::SETTINGS_WINDOW,
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title(title)
    .inner_size(1060.0, 790.0)
    .min_inner_size(760.0, 600.0)
    .build();
    if result.is_err() {
        eprintln!("settings_window_unavailable");
    }
}

pub fn run() {
    whisper_rs::install_logging_hooks();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            show_settings(app)
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--background"])
                .build(),
        )
        .plugin(shortcuts::plugin())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window
                    .try_state::<runtime::Runtime>()
                    .is_some_and(|runtime| {
                        !runtime
                            .tray_available
                            .load(std::sync::atomic::Ordering::Acquire)
                    })
                {
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::save_settings,
            commands::list_microphones,
            commands::control,
            commands::get_recovery,
            commands::get_models,
            commands::install_model,
            commands::cancel_download,
            commands::import_model,
            commands::test_microphone,
            commands::test_phrase,
        ])
        .setup(|app| {
            let runtime = runtime::Runtime::start(app.handle().clone())?;
            app.manage(runtime);
            tray::create(app.handle())?;
            if std::env::args().any(|argument| argument == "--settings") {
                show_settings(app.handle());
            }
            Ok(())
        })
        .build(tauri::generate_context!());
    match app {
        Ok(app) => app.run(|app, event| match event {
            tauri::RunEvent::ExitRequested { api, .. }
                if !app
                    .state::<runtime::Runtime>()
                    .stopping
                    .load(std::sync::atomic::Ordering::Acquire) =>
            {
                api.prevent_exit()
            }
            tauri::RunEvent::Exit => app.state::<runtime::Runtime>().shutdown(),
            _ => {}
        }),
        Err(error) => eprintln!("startup_failed: {error}"),
    }
}
