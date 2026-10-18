# Implementation decisions

Recorded 18 October 2026. The original initialization document remains the release specification; this implementation is an initial Windows build.

## Runtime and resource ownership

Rust owns microphone capture, settings, activation, session authorization, inference, and delivery. A single coordinator serializes sessions and insertion attempts. React is a settings/status surface; it does not capture audio or type into other applications.

The process starts with a native tray and no settings webview. Settings are created on demand and destroyed when closed. The exit-request handler preserves background operation until explicit Quit. A tray-unavailable session keeps its settings window accessible.

Whisper loads after activation and performs batch inference only after recording finishes. Two CPU inference threads are the current operational policy. The loaded context expires after 60 idle seconds. Model selection loads a candidate to validate it and then releases it. Voice activation uses a separate English INT8 sherpa-onnx keyword model with one thread. WebRTC VAD classifies speech before transcription. The initial text processor returns the transcription unchanged.

CPAL callbacks downmix into a preallocated SPSC ring and signal a bounded channel. Resampling, wake detection, VAD, and recording buffers run on the coordinator. Audio callbacks do not run inference, log content, or perform filesystem work. Audio stalls and overflow stop capture; usable interrupted recordings are routed to recovery rather than automatically inserted.

The control queue is bounded. Cancel/pause invalidate the insertion authorization immediately, with a separate priority interrupt flag so a full ordinary control queue cannot lose a pause. Whisper's native cancellation callback retains scoped data until the synchronous native call returns; the callback also enforces an inference deadline.

Runtime timing, thread, buffer, and delivery limits are owned by `crates/dictation-core/src/defaults.rs`. Saved missing preferences remain null. The speech-provider contract contains audio, language, cancellation authorization, and session identity; its core settings use `speech_model`, with Whisper-specific loading kept in the adapter.

## Wake interaction

The English 3.3M GigaSpeech keyword model was selected for a small, customizable local detector. Its BPE tokenizer runs through sentencepiece-rs, so end users do not install Python or train a phrase. Two distinct synthetic English phrases were detected with the actual model after changing the configuration.

The current interaction requires a brief pause after the phrase. A bounded quiet boundary separates the activation command from dictation; the UI describes this requirement and shows preparing/recording state. Immediate speech after a wake phrase remains an acceptance gap. Speaker audio and other people can trigger the detector; it is not speaker authentication.

English keyword support is separate from Whisper's multilingual dictation support. Unsupported characters are rejected. Stop commands require an isolated acoustic boundary; ordinary substring removal is not used. Comprehensive stop-command/noise testing remains required.

Primary references consulted: [sherpa-onnx keyword spotting](https://github.com/k2-fsa/sherpa/blob/master/docs/source/onnx/kws/index.rst), [English model and BPE format](https://k2-fsa.github.io/sherpa/onnx/kws/pretrained_models/index.html), [sherpa-onnx Rust 1.13.8](https://docs.rs/sherpa-onnx/1.13.8/sherpa_onnx/), [WhisperContext 0.16.0](https://docs.rs/whisper-rs/0.16.0/whisper_rs/struct.WhisperContext.html). Exact locked crate sources were also inspected for CPAL, WebRTC VAD, SentencePiece, Windows bindings, and cancellation APIs.

## Windows input adapter

Focus/foreground notifications are event driven. WinEvent callbacks coalesce work into the native helper's message loop. UI Automation queries run outside the real-time callback, with explicit connection/transaction timeout settings. The input offer is a small GDI window using `WS_EX_NOACTIVATE` and `MA_NOACTIVATE`, avoiding another webview for each detected input.

The target identity contains the foreground window, provider process, and UI Automation runtime ID. Writable inputs are accepted; password/unknown protection flags and the application's own process are excluded. Own-window exclusion checks the native foreground owner before inspecting the accessibility provider, because WebView2 settings fields can report a different provider PID. The adapter rechecks identity before dispatch and after the OS call. Modifiers still held at delivery cause refusal, preserving the text for explicit recovery.

Unicode is dispatched through a single bounded SendInput operation. Enter/control characters are not sent. Complete OS acceptance is reported as submitted, not confirmed insertion. Changed, rejected, partial, and uncertain results keep temporary recovery text without automatic retry. Native editor and Edge fixtures inspect their actual received text, including accents, emoji, and right-to-left script.

Unknown targets, full-screen games, elevated applications, IME behavior, and application-specific restrictions remain compatibility cases. No drivers, routine elevation, clipboard replacement, or permission bypasses were introduced.

References: [Microsoft WinEvent hooks](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook), [SendInput and integrity restrictions](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput).

## Settings, startup, and models

Settings are versioned and atomically replaced. Unreadable/invalid original files are preserved and block overwriting. Configuration changes validate resources and registrations before committing; failures attempt to restore previous registrations and listening behavior. Login startup uses Tauri's current-user autostart adapter and remains unset until saved explicitly.

Downloads require an explicit button action. The catalog pins source URLs, byte counts and SHA-256 values. Installation checks available disk space, bounds transfer/extraction size, rejects links/path traversal, verifies the checksum, loads the model, and then atomically installs it. Imports use a native picker and load validation. No models, recorded audio, or credentials are committed.

References: [Tauri tray](https://v2.tauri.app/learn/system-tray/), [Tauri autostart](https://v2.tauri.app/plugin/autostart/), [Tauri configuration](https://v2.tauri.app/reference/config/). Custom command permissions and plugin APIs were checked against their locked source versions.

## Build and packaging

Rust 1.95.0, Node 24.19.0, npm 11.6.1, Tauri Rust 2.12.4, CLI 2.12.3, React 19.3.1, TypeScript 7.0.2, and Vite 8.3.4 were used. Both dependency lockfiles are saved with the project. The workspace initially had no Git metadata, so no commits or repository operations were performed.

Windows builds generate Whisper bindings with libclang 18.1.1 and discovered MSVC/Windows SDK include paths. libclang lives only in ignored development tooling. End users receive native binaries without Python or the Rust/Node toolchain.

The sherpa-onnx shared native distribution isolates its C++ runtime from Whisper's build. Matching ONNX Runtime DLLs must be adjacent to the executable. The test wrapper also places them beside test executables to prevent loading Windows' older system ONNX Runtime. The Windows packaging configuration includes these matching libraries explicitly.

The icon is reproducible from `assets/icon.svg` through the pinned Tauri icon generator. Generated mobile/store icon sizes are not included in the desktop installer. No application license, publisher identity, signing credentials, automatic updater, or public release was selected.
