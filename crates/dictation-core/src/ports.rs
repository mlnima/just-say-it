use std::sync::{atomic::AtomicU64, Arc};

pub struct SpeechRequest<'a> {
    pub session_id: u64,
    pub samples: &'a [f32],
    pub language: &'a str,
    pub authorization: Arc<AtomicU64>,
}

pub trait SpeechProvider {
    fn transcribe(&mut self, request: SpeechRequest<'_>) -> Result<String, String>;
}

pub trait TextProcessor {
    fn process(&self, text: String) -> Result<String, String>;
}

pub struct PassThrough;

impl TextProcessor for PassThrough {
    fn process(&self, text: String) -> Result<String, String> {
        Ok(text)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PhraseDetection {
    pub phrase: String,
    pub start_seconds: f32,
    pub end_seconds: f32,
}

pub trait WakeDetector {
    fn accept(&mut self, samples: &[f32]) -> Option<PhraseDetection>;
    fn reset(&mut self);
}
