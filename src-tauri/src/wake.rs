use dictation_core::{
    defaults,
    ports::{PhraseDetection, WakeDetector},
    settings::validate_phrase,
};
use sentencepiece_rs::SentencePieceProcessor;
use sherpa_onnx::{KeywordSpotter, KeywordSpotterConfig, OnlineStream};
use std::path::Path;

pub struct LocalWake {
    stream: OnlineStream,
    engine: KeywordSpotter,
}

pub fn keyword_line(directory: &Path, phrase: &str) -> Result<String, String> {
    let phrase = validate_phrase(phrase)?;
    let tokenizer = SentencePieceProcessor::open(directory.join("bpe.model"))
        .map_err(|_| "Wake tokenizer is missing or invalid. Reinstall the wake model.")?;
    let tokens = tokenizer
        .encode(&phrase)
        .map_err(|_| "This phrase could not be tokenized.")?;
    let vocabulary = std::fs::read_to_string(directory.join("tokens.txt"))
        .map_err(|_| "Wake vocabulary is missing.")?;
    if tokens.is_empty()
        || tokens.iter().any(|token| {
            !vocabulary
                .lines()
                .any(|line| line.split_whitespace().next() == Some(token.as_str()))
        })
    {
        return Err(
            "This phrase contains characters unsupported by the English wake model.".into(),
        );
    }
    Ok(format!(
        "{} @{}",
        tokens.join(" "),
        phrase.replace(' ', "_")
    ))
}

impl LocalWake {
    pub fn load(directory: &str, phrase: &str) -> Result<Self, String> {
        let directory = Path::new(directory);
        let keyword = keyword_line(directory, phrase)?;
        let mut config = KeywordSpotterConfig::default();
        let model = &mut config.model_config;
        model.transducer.encoder = Some(
            directory
                .join("encoder-epoch-12-avg-2-chunk-16-left-64.int8.onnx")
                .to_string_lossy()
                .into(),
        );
        model.transducer.decoder = Some(
            directory
                .join("decoder-epoch-12-avg-2-chunk-16-left-64.int8.onnx")
                .to_string_lossy()
                .into(),
        );
        model.transducer.joiner = Some(
            directory
                .join("joiner-epoch-12-avg-2-chunk-16-left-64.int8.onnx")
                .to_string_lossy()
                .into(),
        );
        model.tokens = Some(directory.join("tokens.txt").to_string_lossy().into());
        model.num_threads = defaults::WAKE_THREADS;
        model.provider = Some("cpu".into());
        config.keywords_buf = Some(keyword);
        config.keywords_threshold = defaults::WAKE_THRESHOLD;
        config.keywords_score = defaults::WAKE_SCORE;
        config.num_trailing_blanks = defaults::WAKE_TRAILING_BLANKS;
        let engine = KeywordSpotter::create(&config)
            .ok_or("The wake model could not load. Check the model files.")?;
        let stream = engine.create_stream();
        Ok(Self { stream, engine })
    }
}

impl WakeDetector for LocalWake {
    fn accept(&mut self, samples: &[f32]) -> Option<PhraseDetection> {
        self.stream
            .accept_waveform(defaults::SAMPLE_RATE as i32, samples);
        while self.engine.is_ready(&self.stream) {
            self.engine.decode(&self.stream);
            if let Some(result) = self
                .engine
                .get_result(&self.stream)
                .filter(|r| !r.keyword.is_empty())
            {
                let first = result.timestamps.first().copied().unwrap_or_default();
                let last = result.timestamps.last().copied().unwrap_or(first);
                return Some(PhraseDetection {
                    phrase: result.keyword.replace('_', " "),
                    start_seconds: result.start_time + first,
                    end_seconds: result.start_time + last,
                });
            }
        }
        None
    }

    fn reset(&mut self) {
        self.stream = self.engine.create_stream();
    }
}
