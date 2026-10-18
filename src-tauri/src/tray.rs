use crate::{
    runtime::{Action, Runtime},
    show_settings,
};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager,
};

pub fn create(app: &tauri::AppHandle) -> tauri::Result<()> {
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let finish = MenuItem::with_id(app, "finish", "Finish dictation", true, None::<&str>)?;
    let cancel = MenuItem::with_id(app, "cancel", "Cancel dictation", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pause all listening", true, None::<&str>)?;
    let resume = MenuItem::with_id(app, "resume", "Resume listening", true, None::<&str>)?;
    let recovery = MenuItem::with_id(
        app,
        "recovery",
        "Temporary text recovery…",
        true,
        None::<&str>,
    )?;
    let restart = MenuItem::with_id(app, "restart", "Restart application", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &settings, &separator, &finish, &cancel, &pause, &resume, &recovery, &restart, &quit,
        ],
    )?;
    let mut builder = TrayIconBuilder::with_id("dictation")
        .tooltip(app.config().product_name.clone().unwrap_or_default())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            let runtime = app.state::<Runtime>();
            let action = match event.id.as_ref() {
                "settings" | "recovery" => {
                    show_settings(app);
                    None
                }
                "finish" => Some(Action::Finish),
                "cancel" => Some(Action::Cancel),
                "pause" => Some(Action::Pause),
                "resume" => Some(Action::Resume),
                "restart" => Some(Action::Restart),
                "quit" => {
                    runtime.shutdown();
                    app.exit(0);
                    None
                }
                _ => None,
            };
            if let Some(action) = action {
                let _ = runtime.send(action);
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    match builder.build(app) {
        Ok(_) => {
            app.state::<Runtime>()
                .tray_available
                .store(true, std::sync::atomic::Ordering::Release);
            Ok(())
        }
        Err(error) => {
            show_settings(app);
            app.state::<Runtime>().update(|s| {
                s.message = format!("Tray unavailable; keep settings open for access. {error}")
            });
            Ok(())
        }
    }
}
