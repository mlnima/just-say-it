# Third-party components

This document is an attribution summary for the local test distribution, not a license for the application itself. The owner has not selected an application license.

| Component | License and source |
| --- | --- |
| Tauri and its official plugins | MIT / Apache-2.0; https://github.com/tauri-apps/tauri |
| React | MIT; https://github.com/facebook/react |
| whisper-rs 0.16.0 | Unlicense; https://codeberg.org/tazz4843/whisper-rs |
| whisper.cpp | MIT; https://github.com/ggml-org/whisper.cpp |
| OpenAI Whisper weights (optional separate download) | MIT; https://github.com/openai/whisper |
| sherpa-onnx 1.13.8 | Apache-2.0; https://github.com/k2-fsa/sherpa-onnx |
| ONNX Runtime (sherpa-onnx's matching native distribution) | MIT; https://github.com/microsoft/onnxruntime |
| English keyword model (optional separate download) | Apache-2.0 as declared in its bundled README; https://www.modelscope.cn/pkufool/sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01 |
| sentencepiece-rs 0.2.2 | Apache-2.0; https://github.com/Rayato159/sentencepiece-rs |
| CPAL 0.16.0 | Apache-2.0; https://github.com/RustAudio/cpal |

The native sherpa-onnx distribution includes additional third-party components. Retain its accompanying license and third-party notices for public distribution. Cargo.lock and package-lock.json record the remaining Rust and JavaScript dependency versions. A complete transitive-license distribution review and code signing remain release requirements; this initialization must not be represented as a reviewed public release.

Models are not bundled with the application. Their download controls show separate license and language information. Development speech fixtures are generated locally with Windows speech synthesis and are not committed or redistributed.
