use dictation_core::{defaults, settings::Settings};

#[test]
fn unset_and_reset_shortcuts_use_the_default_without_saving_a_custom_value() {
    let mut settings = Settings::default();
    assert_eq!(settings.toggle_shortcut(), defaults::TOGGLE_SHORTCUT);
    assert!(settings.toggle_shortcut.is_none());

    settings.toggle_shortcut = Some("Control+Shift+F9".into());
    assert_eq!(settings.toggle_shortcut(), "Control+Shift+F9");
    settings.toggle_shortcut = None;
    assert_eq!(settings.toggle_shortcut(), defaults::TOGGLE_SHORTCUT);

    let saved = serde_json::to_value(&settings).unwrap();
    assert!(saved["toggle_shortcut"].is_null());
    assert!(saved.get("cancel_shortcut").is_none());
    assert_eq!(serde_json::from_value::<Settings>(saved).unwrap(), settings);
}
