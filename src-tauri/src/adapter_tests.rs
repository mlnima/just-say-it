use crate::{audio, speech::LocalWhisper, store::Store, wake::LocalWake};
use dictation_core::{
    defaults,
    ports::{SpeechProvider, SpeechRequest, WakeDetector},
    settings::Settings,
};
use std::{
    path::PathBuf,
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

fn artifact(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../artifacts")
        .join(path)
}

fn wave(path: &str) -> Vec<f32> {
    let mut reader = hound::WavReader::open(artifact(path)).expect("Generate test audio first.");
    assert_eq!(reader.spec().sample_rate, defaults::SAMPLE_RATE);
    assert_eq!(reader.spec().channels, 1);
    let mut samples: Vec<_> = reader
        .samples::<i16>()
        .map(|n| n.unwrap() as f32 / 32768.0)
        .collect();
    samples.extend(vec![0.0; defaults::SAMPLE_RATE as usize]);
    samples
}

#[test]
fn atomic_settings_preserve_unreadable_originals_and_real_unset_values() {
    let folder = tempfile::tempdir().unwrap();
    let (store, mut settings) = Store::open(folder.path());
    settings.wake_phrase = Some("hello notebook".into());
    store.save(&settings).unwrap();
    let (_, restored) = Store::open(folder.path());
    assert_eq!(restored, settings);
    assert!(restored.wake_enabled.is_none());
    let path = folder.path().join("settings.json");
    std::fs::write(&path, b"unreadable settings").unwrap();
    let (store, _) = Store::open(folder.path());
    assert!(store.save(&Settings::default()).is_err());
    assert_eq!(std::fs::read(path).unwrap(), b"unreadable settings");
}

#[test]
#[ignore = "requires downloaded real models and locally synthesized audio"]
fn keyword_model_recognizes_two_user_phrases_without_retraining() {
    let model = artifact("models/sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01");
    for (phrase, file) in [
        ("hello little notebook", "test-audio/wake-one.wav"),
        ("good morning computer", "test-audio/wake-two.wav"),
    ] {
        let mut detector = LocalWake::load(model.to_str().unwrap(), phrase).unwrap();
        let mut detected = false;
        for frame in wave(file).chunks(defaults::FRAME_SAMPLES) {
            if let Some(result) = detector.accept(frame) {
                assert_eq!(result.phrase, phrase.to_uppercase());
                assert!(result.end_seconds >= result.start_seconds);
                detected = true;
                break;
            }
        }
        assert!(detected, "The configured phrase was not detected: {phrase}");
        detector.reset();
        assert!(vec![0.0; defaults::SAMPLE_RATE as usize * 2]
            .chunks(defaults::FRAME_SAMPLES)
            .all(|frame| detector.accept(frame).is_none()));
    }
}

#[test]
#[ignore = "requires downloaded real Whisper model and synthetic audio"]
fn whisper_transcribes_real_audio_and_honors_invalidated_authorization() {
    whisper_rs::install_logging_hooks();
    let model = artifact("models/ggml-tiny-q5_1.bin");
    let mut speech = LocalWhisper::load(model.to_str().unwrap()).unwrap();
    let audio = wave("test-audio/dictation.wav");
    let authorization = Arc::new(AtomicU64::new(1));
    let text = speech
        .transcribe(SpeechRequest {
            session_id: 1,
            samples: &audio,
            language: "en",
            authorization: authorization.clone(),
        })
        .unwrap();
    let normalized = text.to_lowercase();
    assert!(
        normalized.contains("sentence") && normalized.contains("notebook"),
        "Expected synthetic utterance was not recognized."
    );
    authorization.store(0, std::sync::atomic::Ordering::Release);
    assert!(speech
        .transcribe(SpeechRequest {
            session_id: 1,
            samples: &audio,
            language: "en",
            authorization
        })
        .is_err());
}

#[test]
#[ignore = "opens the real default microphone briefly; requires desktop audio access"]
fn microphone_capture_produces_resampled_frames_and_releases_stream() {
    assert!(!audio::devices().unwrap().is_empty());
    let (sender, ready) = crossbeam_channel::bounded(1);
    let mut capture = audio::Capture::open(defaults::SYSTEM_DEFAULT_DEVICE, sender).unwrap();
    ready.recv_timeout(Duration::from_secs(3)).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert!(!capture.drain().unwrap().is_empty());
    drop(capture);
}

#[test]
#[ignore = "requires locally generated synthetic speech"]
fn activity_detector_classifies_speech_and_silence() {
    let mut detector = crate::voice_activity::VoiceActivity::new();
    for frame in vec![0.0; defaults::SAMPLE_RATE as usize].chunks(defaults::FRAME_SAMPLES) {
        assert!(!detector.accept(frame).unwrap());
    }
    let speech_frames = wave("test-audio/dictation.wav")
        .chunks(defaults::FRAME_SAMPLES)
        .filter(|frame| detector.accept(frame).unwrap())
        .count();
    assert!(speech_frames >= 10);
    for frame in vec![0.0; defaults::SAMPLE_RATE as usize].chunks(defaults::FRAME_SAMPLES) {
        assert!(!detector.accept(frame).unwrap());
    }
}
