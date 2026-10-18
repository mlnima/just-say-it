use dictation_core::{
    defaults,
    session::{safe_inline_text, Delivery, Phase, Session},
    settings::{validate_phrase, Settings},
    speech_activity::SpeechActivity,
};
use std::time::{Duration, Instant};

#[test]
fn cancelled_and_replaced_sessions_cannot_deliver_late_text() {
    let mut session = Session::default();
    let first = session.start().unwrap();
    session.cancel(false);
    assert!(!session.authorize_delivery(first));
    let second = session.start().unwrap();
    session.phase = Phase::Transcribing;
    assert!(!session.authorize_delivery(first));
    assert!(session.authorize_delivery(second));
    assert!(!session.authorize_delivery(second));
}

#[test]
fn uncertain_delivery_blocks_new_recording_and_never_retries() {
    let mut session = Session::default();
    let id = session.start().unwrap();
    session.phase = Phase::Transcribing;
    assert!(session.authorize_delivery(id));
    session.delivered("temporary words".into(), Delivery::Unknown);
    assert_eq!(session.phase, Phase::PendingDelivery);
    assert!(session.start().is_err());
    assert!(!session.authorize_delivery(id));
    assert_eq!(session.recovery.as_ref().unwrap().text, "temporary words");
    session.cancel(true);
    assert!(session.recovery.is_none());
    assert!(session.start().is_err());
}

#[test]
fn successful_submission_keeps_only_a_temporary_recovery_result() {
    let mut session = Session::default();
    session.start().unwrap();
    session.delivered("words".into(), Delivery::Submitted);
    assert_eq!(session.phase, Phase::Idle);
    session.recovery.as_mut().unwrap().created =
        Instant::now() - Duration::from_secs(defaults::RECOVERY_SECONDS + 1);
    session.expire();
    assert!(session.recovery.is_none());
}

#[test]
fn unset_preferences_survive_serialization_without_fake_choices() {
    let settings = Settings::default();
    let restored: Settings =
        serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
    assert_eq!(settings, restored);
    assert_eq!(restored.wake_enabled, None);
    assert_eq!(restored.microphone, None);
    assert_eq!(restored.launch_at_login, None);
    assert!(restored.recording_ready().is_err());
}

#[test]
fn settings_reject_unsafe_timing_and_conflicting_phrases() {
    let mut settings = Settings::default();
    settings.silence_seconds = Some(f32::NAN);
    assert!(settings.validate().is_err());
    settings.silence_seconds = None;
    settings.wake_phrase = Some("hello notebook".into());
    settings.stop_phrase = Some("notebook".into());
    assert!(settings.validate().is_err());
    settings.stop_phrase = Some("finish these words".into());
    assert!(settings.validate().is_ok());
    assert!(validate_phrase("你好").is_err());
    assert_eq!(
        validate_phrase("  hello  notebook ").unwrap(),
        "HELLO NOTEBOOK"
    );
}

#[test]
fn settings_reject_null_characters_before_native_provider_calls() {
    for settings in [
        Settings {
            language: Some("en\0".into()),
            ..Default::default()
        },
        Settings {
            speech_model: Some("model\0.bin".into()),
            ..Default::default()
        },
    ] {
        assert!(settings.validate().is_err());
    }
}

#[test]
fn silence_ends_without_speech_and_wake_recording_ends_after_silence() {
    let settings = Settings::default();
    let mut activity = SpeechActivity::default();
    let silence = vec![0.0; defaults::SAMPLE_RATE as usize];
    for _ in 0..6 {
        activity.accept(&silence, false);
    }
    assert!(!activity.has_speech());
    assert_eq!(
        activity.finish_reason(&settings, true),
        Some("No speech detected")
    );
    activity = SpeechActivity::default();
    activity.accept(&vec![0.04; defaults::SAMPLE_RATE as usize], true);
    activity.accept(&silence, false);
    activity.accept(&silence, false);
    assert!(activity.has_speech());
    assert_eq!(
        activity.finish_reason(&settings, true),
        Some("Silence timeout")
    );
    assert_eq!(activity.finish_reason(&settings, false), None);
}

#[test]
fn inline_delivery_refuses_submission_characters_but_preserves_unicode() {
    assert!(safe_inline_text("Grüße 👋 مرحبا"));
    for text in ["", "line\nline", "line\rline", "tab\there", "\0", "\u{1b}"] {
        assert!(!safe_inline_text(text));
    }
}
