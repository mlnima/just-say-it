use dictation_core::{
    defaults,
    ports::{SpeechProvider, SpeechRequest},
    settings::SpeechDevice,
};
use std::sync::atomic::Ordering;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

pub struct LocalWhisper {
    context: WhisperContext,
    prepared: Option<WhisperState>,
    pub device_request: SpeechDevice,
}

impl LocalWhisper {
    pub fn load(path: &str, device: SpeechDevice) -> Result<Self, String> {
        let mut params = WhisperContextParameters::default();
        params.use_gpu(device == SpeechDevice::Gpu);
        if device == SpeechDevice::Gpu {
            params.gpu_device(crate::speech_device::require_gpu()?.index);
        }
        let context = WhisperContext::new_with_params(path, params)
            .map_err(|_| match device {
                SpeechDevice::Cpu => "Whisper model could not load. Select a valid whisper.cpp GGML model.",
                SpeechDevice::Gpu => "Whisper could not load the model with GPU offload. Check GPU memory and its driver, or select CPU.",
            })?;
        let mut speech = Self {
            context,
            prepared: None,
            device_request: device,
        };
        speech.prepare()?;
        Ok(speech)
    }

    pub fn prepare(&mut self) -> Result<(), String> {
        if self.prepared.is_none() {
            self.prepared = Some(
                self.context.create_state()
                    .map_err(|_| "Whisper could not allocate its working state.")?,
            );
        }
        Ok(())
    }
}

impl SpeechProvider for LocalWhisper {
    fn transcribe(&mut self, request: SpeechRequest<'_>) -> Result<String, String> {
        self.prepare()?;
        let mut state = self.prepared.take()
            .ok_or("Whisper working state is unavailable.")?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        let available = std::thread::available_parallelism().map_or(
            defaults::INFERENCE_THREADS,
            |count| count.get().min(defaults::INFERENCE_THREADS as usize) as i32,
        );
        params.set_n_threads(available);
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
        // Used state contains utterance data. Drop it after reading the result;
        // keep only an empty prepared state between sessions, never past audio.
        Ok(text.trim().to_owned())
    }
}
