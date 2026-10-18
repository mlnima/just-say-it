use dictation_core::{
    defaults,
    ports::{SpeechProvider, SpeechRequest},
};
use std::sync::atomic::Ordering;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct LocalWhisper {
    context: WhisperContext,
}

impl LocalWhisper {
    pub fn load(path: &str) -> Result<Self, String> {
        let mut params = WhisperContextParameters::default();
        params.use_gpu(false);
        let context = WhisperContext::new_with_params(path, params)
            .map_err(|_| "Whisper model could not load. Select a valid whisper.cpp GGML model.")?;
        Ok(Self { context })
    }
}

impl SpeechProvider for LocalWhisper {
    fn transcribe(&mut self, request: SpeechRequest<'_>) -> Result<String, String> {
        let mut state = self
            .context
            .create_state()
            .map_err(|_| "Whisper could not allocate its working state.")?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(defaults::INFERENCE_THREADS);
        params.set_translate(false);
        params.set_language((request.language != "auto").then_some(request.language));
        params.set_no_context(true);
        params.set_print_realtime(false);
        params.set_print_progress(false);
        params.set_print_timestamps(false);
        params.set_print_special(false);
        let id = request.session_id;
        let authorization = request.authorization;
        crate::speech_cancel::run(
            &mut state,
            params,
            request.samples,
            id,
            authorization.clone(),
        )
        .map_err(|_| {
            if authorization.load(Ordering::Acquire) != id {
                "Dictation cancelled."
            } else {
                "Local transcription failed."
            }
        })?;
        if authorization.load(Ordering::Acquire) != id {
            return Err("Dictation cancelled.".into());
        }
        let mut text = String::new();
        for segment in state.as_iter() {
            // Do not insert segments that Whisper itself rates as predominantly non-speech.
            if segment.no_speech_probability() < defaults::MAX_NO_SPEECH_PROBABILITY {
                text.push_str(
                    &segment
                        .to_str_lossy()
                        .map_err(|_| "Whisper returned invalid text.")?,
                );
            }
        }
        Ok(text.trim().to_owned())
    }
}
