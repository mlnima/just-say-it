# Just Say It: project initialization

Status: build specification, not an implemented application. Prepared on 18 October 2026.

This document is the handoff to the model that will build Just Say It. Read [AGENTS.md](AGENTS.md) first. The owner should be able to edit this plan without understanding every library. Product rules are written in plain language; implementation details explain how to preserve those rules.

## 1. The goal

Just Say It lets a person write with their voice in the text input they are already using. The input may belong to a browser, desktop program, editor, or game chat. The person activates dictation with their chosen keyboard shortcut or wake phrase, speaks, and finishes. The application converts the speech to text and inserts it at the active caret or selection.

Build for Windows, Ubuntu, and macOS using Rust, Tauri 2, and React with TypeScript. Keep the application small, responsive, private by default, and easy to extend. OpenAI Whisper is the first speech recognition engine. Later, users should be able to select other local or API providers and optionally send a transcript through an LLM for cleanup.

“Universal” describes the intended reach. It is not permission to promise access to protected or unsupported inputs. Make supported cases work well and explain limitations without losing the person's words.

## 2. Scope and proposed decisions

The first usable release includes every item below. Milestones later in this document describe the order of implementation; they do not remove requirements.

| Required outcome | What it means |
| --- | --- |
| Three operating systems | Real installed application behavior on Windows, Ubuntu, and macOS, with a published compatibility matrix. |
| Active-input dictation | Speak, finish, and insert text into supported inputs without manually moving text through the settings window. |
| Configurable shortcuts | User-selected toggle, hold-to-talk where supported, and cancel bindings with conflict reporting. |
| Configurable voice activation | User-selected wake phrase, phrase validation, and a real listening test. No fixed brand phrase or per-phrase training requirement for supported languages. |
| Clear stopping behavior | Finish, cancel, pause all listening, configurable silence timeout, and a maximum recording duration. |
| Local Whisper | Real microphone audio processed locally, offline after model installation; no account or API key required. |
| Background operation | Tray/menu access, operation with settings closed, optional launch at login, one instance per user session. |
| Truthful settings | React UI for actual configuration, permissions, model status, activation, and microphone state. |
| Extensible processing | Small speech-provider and text-processor interfaces; no dependency on one vendor in the domain core. |
| Maintainable code | Clear module ownership, meaningful tests, and fewer than 300 lines in every authored code file. |

These are recommended starting decisions, chosen to make the first version reliable. They can be changed by editing this plan before implementation:

- Run Whisper locally through `whisper.cpp` and an evaluated Rust binding or a small native wrapper. OpenAI's Whisper model and a hosted transcription API are different deployment choices. Local inference is the initial choice here. [Whisper](https://github.com/openai/whisper), [whisper.cpp](https://github.com/ggml-org/whisper.cpp)
- Insert the final transcript once the person finishes. Do not rewrite text in another app every time a partial transcript changes. Live partial text may appear in a non-focus-stealing status surface. Continuous committed dictation is future work.
- Use a separate, small local keyword detector for wake phrases. Do not run full Whisper continuously just to look for a wake phrase.
- Interpret “service” as background operation in the signed-in desktop session with optional startup registration. Do not implement a machine-wide Windows service, root daemon, or macOS LaunchDaemon for typing. Windows services cannot directly interact with the user's desktop. A separate system service would need its own justified requirement and a user-session companion. [Microsoft service guidance](https://learn.microsoft.com/en-us/windows/win32/services/interactive-services)
- Use one Tauri desktop application and one small Rust core crate initially. Do not start with microservices, a plugin marketplace, a separate web server, or a large monorepo.
- First release implements one real speech provider and a pass-through text processor. Remote providers and LLM cleanup are planned extensions, not required integrations in the initial build.
- Initial architecture targets are Windows x86-64, Ubuntu x86-64, and macOS Apple Silicon and Intel. Select and record exact supported OS releases and minimum versions in the first milestone after checking dependency support. Additional CPU targets are future work, not implicitly supported.

## 3. The everyday experience

### First run

1. Explain what microphone access and desktop typing access are used for. Request permissions only when the person enables the related feature.
2. Let the person choose a real microphone or explicitly choose “Follow system default.” Show a live level test only while that test is active.
3. Let them choose an installed local model or approve a model download. Show size, language coverage, license, disk needs, and progress. Do not silently download a large model.
4. Let them record and test shortcut bindings. Show whether each binding was actually registered.
5. Offer voice activation as a separate opt-in choice. Let them enter and test their own phrase. Explain that other people or audio from speakers may trigger it; this is not speaker authentication.
6. Save the person's choices and provide a dictation test in the app, followed by a guided test in another application. An in-app test alone does not prove desktop typing works.

No wake phrase, shortcut, microphone, provider credential, or startup preference should appear selected unless it was saved. Recommended values may be displayed separately as suggestions. Selecting a suggestion and saving it creates a real setting.

### A normal dictation session

1. The person places the caret in the destination input.
2. Their shortcut or wake phrase starts a session. The app records the strongest target identity available from the OS and displays a listening indicator without taking focus.
3. The person speaks. Audio stays local with the initial provider.
4. They finish using the same toggle shortcut, by releasing hold-to-talk, by saying the configured stop phrase, or by reaching the enabled silence timeout.
5. The app stops recording, transcribes, applies the selected text processor, rechecks the destination, and attempts one insertion.
6. The app reports its actual delivery result. A completed submission returns to idle; a rejected, partial, or uncertain attempt enters pending recovery. On return to idle, the local wake detector resumes only if enabled and not paused; otherwise the microphone is closed.

Never automatically send a message, click a button, press Enter, or execute a command. Dictation inserts text only. Avoid automatic newline insertion in the first release because some controls treat it as submission; keep multiline content in the recovery view until the destination's behavior is known and the person explicitly requests insertion.

## 4. Activation, finish, cancel, and pause

| Action | Behavior |
| --- | --- |
| Toggle shortcut | In idle: start. During recording: finish. During finalization: ignore repeated start/finish presses and show the current status. |
| Hold-to-talk | Start on press; finish on release. Enable only when the platform supplies reliable press and release events. Recover from a missed release using the recording limit. |
| Wake phrase | Start only while wake listening is enabled and no session is active. Do not include the wake phrase in the transcript. |
| Stop phrase | Finish the current recording. Recognize only an isolated configured command with clear speech boundaries; remove only the recognized command span from audio/transcript. |
| Cancel shortcut or button | Stop the active session and discard uncommitted content. Ignore its late inference results. Cancellation cannot undo text already delivered. |
| Silence timeout | When enabled, finish after the configured silence following detected speech. Silence before any speech ends without insertion after a separate no-speech timeout. |
| Recording limit | Finish at a configured maximum length and explain why. Never grow the audio buffer without limit. |
| Pause all listening | Cancel active work, stop wake detection, release the microphone, and invalidate pending insertion. Resume requires an explicit user action. |
| Quit | Stop capture and workers, release shortcuts and resources, and exit. No background microphone process remains. |

Cancel ends one utterance and then returns to the configured idle mode; wake listening may resume. Pause all listening is the privacy control that keeps the microphone off. Persist the paused state separately from wake enablement and honor it across app restarts and login startup until an explicit Resume action. Label these differently.

Offer a configurable cancel chord; do not globally hijack plain Escape. Plain Escape may cancel when the app's own window has focus. Require at least one usable finish/cancel route before recording starts. Provide tray/menu controls as a backup.

Wake and stop phrases must differ. Reject empty phrases and explain confusingly similar or unsupported phrases. An exact sentence containing the words of a stop phrase is ordinary dictation unless it meets the isolated-command rule. Offer an explicitly saved option to disable voice stopping when dictating those words. Do not implement broad substring replacement of command words.

Suggested values for onboarding, not automatically selected settings: a 2-second silence timeout for wake-started sessions, a 5-second no-speech timeout, and a 60-second recording limit. Hold-to-talk normally ends on release; toggle normally ends on the next press. Silence ending for those modes is a separate choice. Keep recommended values and operational bounds in the shared defaults module. Validate all saved timing values and describe the active policy in the UI.

Audio played by the application must not trigger activation. Keep sounds optional and test them with speakers, not just headphones. Add a small configurable cooldown after finish/cancel to prevent immediate repeated triggers, without dropping the next intentional utterance.

## 5. Custom wake phrases are a real feature

Use a `WakeDetector` interface, separate from `SpeechProvider`. The detector consumes small audio frames and emits a phrase event with timing information. The same detector family may recognize the stop phrase during recording if it meets the boundary and latency requirements.

Evaluate sherpa-onnx open-vocabulary keyword spotting first. Its documented approach supports changing keywords without retraining, with model-specific tokenization and recognition thresholds. This is a candidate, not a claim that a selected Rust package already meets every requirement. Confirm redistribution rights, native packaging, supported languages, runtime phrase changes, and CPU use before committing to it. [Keyword spotting documentation](https://k2-fsa.github.io/sherpa/onnx/kws/index.html)

- Translate user-entered phrases into the selected detector's token format inside the app; do not require the end user to install Python or run a training script.
- Expose the detector's actual supported languages separately from Whisper's dictation languages. Do not promise every phrase in every language.
- Provide a phrase test that measures successful detections and explains likely false triggers. Very short, common phrases should produce a warning and a better-phrase suggestion, not a fixed mandatory phrase.
- Prove that changing the phrase in settings changes live recognition after saving, without restarting the whole application or training a new model.
- Keep a short, bounded in-memory audio ring while wake listening is enabled. Use phrase timing to preserve speech immediately after the wake phrase without transcribing the command or earlier background conversation.
- If timing is too uncertain to separate command and dictation safely, show that activation needs a brief pause and validate that interaction. Do not quietly cut off the first word.
- Suspend wake activation while recording or finalizing. Only the selected stop-command detector may consume recording audio for commands.
- Never fall back to sending continuous background audio to a server.

The first milestone must prove customizable wake detection on the planned targets. If the candidate fails, investigate another local detector using evidence. Do not substitute a fixed phrase or omit the feature and call the release complete. Record any unresolved language or platform gap explicitly.

## 6. Operating systems and typing into other apps

Build a small `TextInserter` platform boundary. Report capabilities such as direct text input, clipboard paste, focus inspection, secure-field detection, and shortcut press/release support. Capability results have three states: available, unavailable, and unknown, with a reason.

| Platform | Implementation direction | Important limits and required evidence |
| --- | --- | --- |
| Windows | User-session process, native input/accessibility integration, Unicode-capable insertion, shortcut registration, startup registration. | Normal input injection cannot target higher-integrity apps. Test ordinary browsers/editors, focus changes, Unicode, and rejected elevation cases. Do not run the app as administrator to conceal this limit. |
| macOS | User-session app with microphone and required Accessibility approval; native text/event adapters and launch-at-login registration. | Test permission denial/revocation, secure inputs, non-US keyboards, signed bundles, and both planned CPU architectures. Request Input Monitoring only if the actual implementation requires it. |
| Ubuntu X11 | X11-compatible input/shortcut adapters, desktop-session startup, audio backend integration. | Test desktop focus changes, keyboard layouts, tray availability, permissions, and the chosen supported Ubuntu release. |
| Ubuntu Wayland | Detect the actual compositor and portal capabilities. Evaluate GlobalShortcuts and RemoteDesktop input-emulation paths, including libei when supported. | Do not assume X11 APIs work for native Wayland apps. Portals may require user approval and do not guarantee focused-field inspection. Test the actual Ubuntu GNOME session and portal versions. |

Windows input injection is constrained by integrity levels. [SendInput documentation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)

On macOS, check actual Accessibility trust and provide the required microphone purpose description. [Accessibility trust API](https://developer.apple.com/documentation/applicationservices/1459186-axisprocesstrustedwithoptions), [microphone usage description](https://developer.apple.com/documentation/bundleresources/information-property-list/nsmicrophoneusagedescription)

Tauri provides a global-shortcut plugin, but its underlying global-hotkey project's documented Linux support is X11. Validate the exact selected version rather than assuming Wayland support. Wayland needs an explicitly tested path. [Tauri shortcuts](https://v2.tauri.app/plugin/global-shortcut/), [global-hotkey platforms](https://github.com/tauri-apps/global-hotkey), [GlobalShortcuts portal](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.GlobalShortcuts.html)

For Wayland, input capture and input injection are different capabilities. Evaluate the RemoteDesktop portal for emulated input; the InputCapture portal is not a general typing API. Do not request screen capture merely to insert text. [RemoteDesktop portal](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html)

### Destination and delivery rules

- Prefer a native Unicode insertion method that has been proven in the target class of application. Do not assume a library claiming three platforms supports all required desktop sessions or text systems.
- Capture application/window identity and focused-control identity when available. Check again just before insertion and during any multi-step delivery. A matching window alone does not prove the same field is focused.
- If the known target changes, stop automatic delivery. Keep the final transcript in memory and let the person explicitly choose to insert it into their current target or copy it.
- When the OS cannot inspect focus reliably, show that limitation. Use an explicit “insert pending text” action after the person focuses the intended input. Do not label this mode as verified automatic target protection.
- The feasibility milestone must determine whether automatic insertion and the desired target protection can both be achieved on each target. If Wayland requires a more restricted workflow, document it as a support gap needing an owner decision; a copy-only fallback does not satisfy automatic-input acceptance.
- Do not bring an old target window to the foreground automatically. Never make the settings window the accidental destination.
- If a password or secure field is detected, refuse insertion. Detection is not guaranteed everywhere; say so instead of claiming complete password-field protection.
- Respect existing selections as the target application's normal typing behavior does. Do not read entire documents or simulate Select All to perform dictation.
- Wait for the activation chord's modifiers to be released before inserting. Test IME composition, dead keys, right-to-left text, emoji, punctuation, and non-English keyboard layouts.
- Distinguish submitted, confirmed, rejected, partial, and unknown results. “Submitted” means the OS accepted the complete input operation; “confirmed” requires evidence from the destination. Show “Sent to app” for submission, never “Verified inserted.” A successful OS dispatch can return to idle without forcing a recovery dialog, but keep the latest text temporarily available if the target ignored it.
- On partial or unknown delivery, preserve the transcript and show that the person should check the target. Never retry automatically; it could duplicate text.
- Give each session and delivery attempt an ID. Prevent duplicate attempts inside the app, but do not claim exactly-once delivery across a process crash or an uncooperative target.

### Clipboard policy

Clipboard paste is an optional compatibility method, not an invisible side effect. Explain that clipboard managers or cloud clipboard services may retain the text and that restoration cannot undo that exposure. Require the person to enable this method explicitly.

Before replacing the clipboard, preserve the formats the adapter can safely restore. If it cannot preserve the current content, explain the limitation and require an explicit copy/paste action rather than silently destroying it. Serialize app-owned clipboard operations. Restore only if the clipboard still contains the app-owned value and its ownership/change marker has not changed. Never overwrite content the person copied meanwhile. Account for asynchronous clipboard reads on Linux; a fixed short delay alone does not prove the target has pasted. If delivery or restoration is uncertain, report it without repeated pastes.

Games are supported only when their text input accepts normal permitted input. Some exclusive-fullscreen and anti-cheat protected games will reject automation. Test game chat where possible and record the result. Do not install drivers, bypass anti-cheat, or synthesize gameplay actions. Remote desktops and virtual machines are separate compatibility cases and must not inherit a support claim from the host editor test.

## 7. Background operation and application lifecycle

Keep the Rust runtime alive when the settings window closes. Create or show the React window when needed. The tray menu should offer status, finish/cancel when relevant, pause/resume listening, settings, pending text recovery, and quit. Starting dictation from a menu must not capture the menu as the destination; return focus and wait for an explicit target action if necessary.

Use Tauri's tray and autostart capabilities where they satisfy the target. Linux tray event behavior differs, so use actual menu actions rather than depending on a special click gesture. If the desktop has no usable tray, keep an accessible window/launcher path and do not hide all controls. [System tray](https://v2.tauri.app/learn/system-tray/), [Autostart](https://v2.tauri.app/plugin/autostart/)

- Start at login is off until the person explicitly saves it. Register only for the current user; make disable/uninstall remove that registration. Do not create competing startup mechanisms.
- Launching a second instance should reveal the first instance, not open a second microphone stream or inference engine.
- Startup must not begin recording a dictation. Resume wake monitoring only when the person previously enabled it, setup is complete, required permissions remain valid, and the persisted paused state is clear. Startup must never clear a pause automatically.
- Lock, suspend, logout, or session switching cancels active dictation, clears pending sensitive text, and releases capture. On unlock/resume, remain paused until explicitly resumed in the initial release.
- A missing chosen device must not silently switch to a different physical microphone. An explicitly saved “Follow system default” choice may follow OS changes and must show the active device.
- Device disconnection during recording stops the session. If usable partial audio exists, offer recovery after explaining the interruption; do not insert it automatically.
- Graceful shutdown must have bounded waits. If native inference cannot stop immediately, invalidate its result immediately and never deliver it later.
- After a crash, do not replay old insertion attempts or resume an unfinished recording. Explain that the previous session was interrupted if that can be determined without storing its content.

## 8. Architecture and ownership

Use a simple flow:

```text
Shortcut / local wake detector
          |
          v
Session coordinator -> audio capture -> speech provider
          |                                  |
          |                                  v
          |                            text processor
          |                                  |
          +---- target check <---------------+
                    |
                    v
             OS text inserter

React settings <-> typed Tauri commands/events <-> Rust runtime
```

The Rust core owns decisions. Platform and provider adapters perform external work. React displays saved settings and observed state and submits explicit user actions. It does not own microphone capture, background loops, secrets, or desktop typing.

| Boundary | Responsibility |
| --- | --- |
| `AudioSource` | Device discovery, capture lifecycle, audio frames, and device errors. |
| `WakeDetector` | Supported phrase/language reporting, phrase configuration, and timestamped command detections. |
| `SpeechProvider` | Model/provider readiness, transcription, capability reporting, progress, cancellation, and errors. |
| `TextProcessor` | Turn a transcript into final text; first implementation returns it unchanged. |
| `TextInserter` | Target/capability inspection, guarded insertion, and honest delivery outcomes. |
| `SettingsStore` | Typed configuration, validation, atomic persistence, and schema migrations. |
| Platform lifecycle adapters | Shortcuts, permission checks, lock/suspend signals, startup registration, and tray integration. |

Use traits or equally small typed interfaces at these actual substitution boundaries. Do not introduce a general plugin system, a dependency-injection framework, or one interface per trivial function.

Provider requests carry a session ID, language selection, audio format, cancellation signal, and deadlines. Results carry text, language when known, and optional timestamps or confidence only when supplied by that provider. Never invent confidence scores. Providers report supported languages, streaming/batch behavior, local/network use, and model requirements. Do not force future API providers to pretend they have local model paths.

### Session state

Represent states explicitly: `idle`, `preparing`, `recording`, `transcribing`, `processing_text`, `inserting`, `pending_delivery`, `paused`, and `error`. Track wake monitoring and readiness separately; “idle” alone must not imply the microphone is off.

One coordinator owns transitions and session IDs. Only one dictation session may be active. Rapid repeated triggers, duplicate UI events, and stale worker completions must not create a second recording or insertion. Commands should be idempotent where possible.

Finish freezes the audio and moves toward delivery. Cancel invalidates that session immediately. Pause and quit also block future activation. Once insertion has started, cancellation stops further work but may leave a partial insertion; report that outcome. The UI should receive a complete state snapshot when opened and ordered updates afterward.

### Concurrency and audio

Capture on the native audio path, not the browser microphone API. Resample once into the model's required format and preserve a sample/time mapping for command boundaries. Treat any initial sample-rate choice as an adapter requirement to verify.

Run inference on a dedicated worker with bounded queues. Do not spawn unbounded work per audio frame. Keep audio callbacks free of blocking I/O, inference, allocations, and logging. Surface buffer overflow as a recoverable error rather than silently losing words.

Use speech activity detection to avoid transcribing silence. Whisper may produce text from noisy or silent input; silence/no-speech tests are required. Keep capture limits bounded, cache one selected model when appropriate, and report warm-up status. Do not claim zero latency or full streaming behavior from a batch decoder.

## 9. Proposed project tree

This is an intended layout, not a list of existing files. Create files as their responsibilities are implemented; do not generate empty future modules. Split modules further before any code file reaches 300 lines.

```text
just-say-it/
  AGENTS.md
  project initialization.md
  README.md
  Cargo.toml                       # Workspace: core crate and Tauri app
  Cargo.lock
  rust-toolchain.toml
  package.json                     # One frontend package; pinned package manager
  <selected package-manager lockfile>
  tsconfig.json
  vite.config.ts
  index.html
  src/
    main.tsx
    App.tsx
    bridge/
      commands.ts
      events.ts
      types.ts                     # Typed IPC contract; split if needed
    features/
      setup/
      settings/
      status/
      recovery/
    components/                    # Only genuinely shared UI
    styles/
  src-tauri/
    Cargo.toml
    build.rs
    tauri.conf.json
    capabilities/
    icons/
    src/
      main.rs
      lib.rs                       # Composition and startup only
      commands/
      events.rs
      runtime.rs                   # Wire core to real adapters
      adapters/
        audio/
        speech/
          whisper.rs
        wake/
        insertion/
          windows/
          macos/
          linux_x11/
          linux_wayland/
        shortcuts/
        permissions/
        lifecycle/
        settings/
      tray.rs
  crates/
    dictation-core/
      Cargo.toml
      src/
        lib.rs
        session/
        ports/                     # Small adapter contracts
        settings/
          defaults.rs
          schema.rs
        text/
          passthrough.rs
        errors.rs
      tests/
  tests/
    fixtures/                      # Small, licensed synthetic/test recordings
    desktop/                       # Tests/procedures needing a real desktop
  docs/
    decisions.md
    compatibility.md
    validation.md
  .github/workflows/                # Add CI when repository hosting is configured
```

Use Rust module declarations as needed; the no-barrel rule applies to TypeScript `index.ts` files. Keep source-owned tests under the same file-size rule. Do not put models, recordings, credentials, installed dependencies, or build output in version control.

## 10. Settings, models, and the interface

Rust owns the settings schema and validates every change. Use a versioned settings file in the OS-specific user data/config location, written atomically. Separate user preferences from discovered hardware, permission state, installed-model state, and observed activation state.

| Settings area | Contents |
| --- | --- |
| Audio | Saved microphone choice, explicit system-default option, device test, active-device status. |
| Activation | Saved shortcuts, mode, wake enablement, phrase, phrase language, phrase test, supported sensitivity controls. |
| Stopping | Stop phrase, voice-stop choice, silence policy, no-speech timeout, recording limit. |
| Transcription | Provider, installed model, language or explicitly chosen automatic language detection, resource options the provider actually supports. |
| Delivery | Available insertion method, explicit clipboard permission, target limitation notices, pending result actions. |
| Background | Launch at login, close-to-tray preference, current startup registration and listening state. |
| Privacy and diagnostics | Current local/network processing mode, data retention explanation, redacted diagnostic export. |

Missing settings stay missing in the form. All dropdowns include an empty first option. Any runtime fallback outside the form comes from the shared defaults module and is identified when shown. A first-run recommendation becomes a setting only after the person saves it. Do not display an unimplemented provider as a selectable working option.

Apply configuration changes transactionally where possible: validate, prepare the new resource, persist, then switch. Keep the previous working state on failure. If an OS forces unregistration before a new shortcut can be tried, attempt rollback and report both failures honestly. Reconcile saved startup preferences with actual OS registration. Apply model, microphone, and provider changes between sessions, not halfway through a recording.

Model management needs explicit download approval, cancel/retry, integrity checking against trusted metadata, available-space checks, temporary download files, and atomic installation. Reject corrupt/incompatible models with a clear error. Permit importing an existing compatible model for offline installation. Treat model licenses separately from engine-code licenses. Never mark a model ready before loading has succeeded.

Keep the UI small: setup, settings, recording/status, and a temporary recovery view. Use clear text, keyboard navigation, screen-reader labels, system scaling, visible focus, and status indicators that are not color-only. Closing settings must not interrupt a recording or become the insertion target. A settings microphone test must not compete with an active session.

## 11. Privacy and security

- Default to local processing. After model setup, dictation and wake recognition must work with networking unavailable.
- Do not retain recordings or transcript history by default. Audio exists only in bounded memory for wake detection and the active session. Release it after processing/cancellation; avoid promising forensic erasure from OS swap or crash dumps.
- Keep at most the latest transcript temporarily for delivery recovery, including a completed submission that the destination may have ignored. Proposed expiry is five minutes, held in the shared defaults module and disclosed in the recovery view. Clear it on expiry, explicit discard, pause, lock, logout, or exit. A new recording may replace a completed result; resolve or explicitly discard a rejected, partial, or unknown result first. Cancelled uncommitted utterances are discarded, not added to recovery.
- Do not put transcript text, raw audio, clipboard data, full window titles, or credentials in routine logs, analytics, crash reports, or notifications.
- No analytics or automatic diagnostic uploads in the initial release. Redacted support exports are user initiated.
- Use the OS credential store when a future API integration needs a secret. If secure storage is unavailable, require a session-only credential or report the limitation; do not write plaintext as a fallback.
- Keep credentials and provider network calls in Rust. Do not expose raw secrets to React or broad filesystem/shell powers to the webview.
- Grant the minimum Tauri capabilities needed by each window. Use a restrictive content security policy, bundled UI assets, validated IPC payloads, and no remotely loaded UI code. Check both plugin permissions and custom-command access rules. [Tauri capabilities](https://v2.tauri.app/security/capabilities/)
- Treat dictated content and LLM output as text data. No shell execution, tool use, automatic submitting, or hidden command interpretation.
- Verify model/update sources, package signatures where applicable, and downloads. Do not fetch and execute arbitrary code in model setup.

## 12. Future providers and optional cleanup

Build the boundaries now; implement integrations only when separately requested. The first `TextProcessor` returns the transcript unchanged and does not load an LLM or make network requests.

A future cleanup step sits after transcription and before insertion. It must be optional, cancellable, and explicit about local versus network processing. Preserve the person's meaning, facts, names, numbers, and intent. Do not turn a short message into an invented story or answer the transcript as if it were a request to the model. Keep original and processed text available in the session for review/recovery.

Each processing step must have its own provider choice, credentials reference, timeout, and capability report. On cleanup failure, use the original transcript only if the person has explicitly selected that failure policy; otherwise offer the original for review. Never silently send data to a different vendor or change languages. Prevent transcript content from overriding the processor's instructions or requesting tools.

Useful later improvements, not first-release requirements:

- Personal vocabulary and pronunciation support for names and technical words.
- Continuous dictation using stable committed segments, with tested overlap handling and no rewriting of already inserted text.
- Optional local or API text cleanup with a before/after preview.
- Explicit per-application preferences, after target detection has been proven.
- More wake languages, CPU architectures, and Linux desktop environments.
- Optional retention/history and audio export with clear consent, deletion, and storage controls.

Do not add chat, general computer control, automatic message sending, cloud accounts, synchronization, or a marketplace to the initial project.

## 13. Performance goals

Measure in release builds with the settings window closed. Publish the hardware, OS, model, audio length, and warm/cold state. The figures below are proposed engineering targets, not achieved claims:

| Measurement | Initial target |
| --- | --- |
| Idle with wake listening off | Microphone closed, no model inference, average CPU below 1% of one logical core over five minutes. |
| Idle with wake listening on | Local detector only; aim below 5% of one logical core on the documented reference machine. Measure battery impact too. |
| Activation feedback | Visible recording status within 200 ms of an accepted trigger when resources are ready; distinguish preparing from recording. |
| Warm transcription | For a 10-second utterance, aim for final text within two seconds after finish on the chosen reference model/hardware. Record p50 and p95, not just the best run. |
| Cancel/pause response | Invalidate delivery immediately; aim to release active microphone capture within 250 ms. Measure native-worker cancellation separately. |
| Memory | Bounded buffers and queues; no steady growth across 100 sessions. Report base app, model, and wake-detector memory separately. |

Establish a memory budget after measuring the chosen models; do not invent one figure that hides model size. Offer a smaller model when hardware is slow. CPU inference is the baseline; optional acceleration should be enabled only after it is packaged and tested for the target. A tested CPU fallback for unavailable acceleration is an implementation choice, not a switch to another provider or to cloud processing.

Do not chase every acceleration backend in the first release. Load one selected speech model, avoid duplicate resampling/copies, limit concurrent native threads, and avoid polling when events are available. Measure model unloading/warm-up tradeoffs before adding another setting.

## 14. Implementation milestones

These are implementation steps. The acceptance checklist in section 15 remains independent.

### Milestone 0: prove the difficult boundaries

Inspect the workspace and record actual tooling. Select supported OS releases, architectures, package-manager version, Rust toolchain, and compatible Tauri 2 dependencies using official documentation. Pin the selected versions and lockfiles during setup.

Before building a large settings UI, make small disposable or reusable probes for microphone capture, local Whisper inference, editable wake/stop phrases, shortcut press/release, target inspection, Unicode insertion, and background startup. Test Windows, macOS, Ubuntu X11, and Ubuntu Wayland separately. Check native dependency size and model redistribution terms.

Record actual results in `docs/decisions.md` and `docs/compatibility.md`, including failed approaches and the resulting decision where useful. Resolve a failed probe with another evidence-supported approach. If required OS access, hardware, signing credentials, or a viable safe platform API is missing, keep that requirement blocked/unverified and report exactly what is needed. Continue independent work; do not call the entire release complete.

### Milestone 1: establish the small application shell

Create the Rust workspace, Tauri 2 app, React/TypeScript/Vite frontend, typed bridge, settings schema/defaults, and minimal tray/lifecycle wiring. Keep domain decisions in the core. Set up reproducible development commands and document only commands that actually exist. Configure narrow permissions and meaningful core-test execution.

### Milestone 2: deliver shortcut dictation end to end

Implement microphone selection, capture, speech activity detection, model installation/loading, local transcription, session cancellation, target guarding, and one insertion attempt. Add a recovery view for failed/uncertain delivery. Prove actual speech reaches a real external input on the current platform, then complete the other platform adapters. Do not stop at a console transcript or a demo input.

### Milestone 3: deliver customizable hands-free activation

Implement the selected wake engine, phrase configuration and testing, stop phrase, bounded pre-roll, silence ending, no-speech timeout, cooldown, and recording limit. Test immediate speech after activation, incidental mentions, competing voices, speaker audio, and settings changes. A fixed wake phrase is not a substitute.

### Milestone 4: finish settings and background behavior

Complete permission guidance, actual registration status, transactional settings changes, launch-at-login choice, one-instance behavior, pause/quit controls, device loss, lock/suspend handling, accessibility, and the tray-unavailable path. Verify the app remains usable with settings closed.

### Milestone 5: validate and package all targets

Run the acceptance checks below using real desktop sessions. Create native packages for the selected targets: a Windows installer, a macOS application distribution, and an Ubuntu package. Verify installation, launch, upgrade with settings preserved, startup registration, and uninstall. Test final packaged binaries, not only development launches.

Document build prerequisites, licenses/notices, compatibility limits, and measured performance. Use protected signing credentials for distribution; do not invent publisher identities or claim unsigned artifacts are signed/notarized. If those credentials are unavailable, provide test packages and mark public distribution validation unverified. Do not add an automatic updater merely because packaging exists; any future updater must verify signed updates.

After setup, keep `AGENTS.md` accurate with the actual layout and real install/dev/test/build commands. Supply a final handoff with completed criteria, observed evidence, and specific remaining blockers. Do not publish a release or push repository changes unless asked.

## 15. Acceptance checklist

These boxes start unchecked. Mark one complete only when its evidence exists. Record evidence in `docs/validation.md` with OS/version, architecture, desktop/session type, model, test setup, observed result, and relevant limitations.

- [ ] **Platforms:** The installed app is tested on the selected Windows, Ubuntu, and macOS targets. Ubuntu X11 and Wayland results are separate; macOS CPU targets are separate. Unsupported or inaccessible cases are marked honestly.
- [ ] **Local transcription:** A real microphone utterance becomes accurate usable text through the selected local Whisper model, with network disabled after model installation. Silence does not produce inserted text.
- [ ] **External input:** Dictation inserts into a native editor and browser inputs, including a multiline editor/contenteditable case with safe submission behavior, on every supported desktop session. Demonstrate actual target results, not only successful OS calls.
- [ ] **Games:** A representative available game's text/chat input is tested and its result recorded. If no suitable game is available, game compatibility remains unverified. Do not generalize one game result to all games.
- [ ] **Shortcuts:** User-chosen toggle and cancel bindings work outside the app; conflicts and rejected registrations are visible. Hold-to-talk works where supported, and unavailable release events are shown as a limitation.
- [ ] **Wake phrases:** Changing and saving the phrase changes actual recognition without retraining. Test two distinct supported phrases, rejection/handling of unsupported phrases, and repeated activation without duplicate sessions.
- [ ] **Stopping:** Finish inserts once; cancel discards; pause releases capture; quit leaves no capture process. Stop phrase, silence, no-speech timeout, missed key release, and recording limit behave as specified.
- [ ] **Target protection:** Switching apps or fields during recording/finalization does not silently insert into the wrong known target. Unknown target inspection uses the declared workflow and remains a documented support gap if it cannot satisfy automatic-input requirements.
- [ ] **Delivery failures:** Rejected, partial, and unknown insertions preserve recoverable text and do not auto-retry. Clipboard tests cover changed clipboard content, rich data, delayed consumption, and restoration failure when clipboard mode is enabled.
- [ ] **Background lifecycle:** Closing settings preserves background functionality; login startup obeys the saved choice; second launch reuses the first instance; lock/suspend clears active content and pauses listening. A pause survives restart/login until explicit Resume.
- [ ] **Truthful settings:** Unset fields remain unset, dropdowns start empty, saved values survive restart, failed changes preserve/report the actual state, and missing devices/models/permissions never appear ready.
- [ ] **Robustness:** Permission denial/revocation, unplugged mic, model corruption, download interruption, disk full, worker failure, rapid triggers, and late provider results produce controlled recoverable outcomes.
- [ ] **Text correctness:** Non-US layouts, accents, right-to-left scripts, emoji, selection replacement, and IME interaction are tested for the selected insertion methods. No accidental Enter, submit, or terminal execution occurs.
- [ ] **Privacy:** No background network transcription, routine content logging, saved audio/history, or plaintext credentials. Explicit local/network status and microphone state match runtime behavior.
- [ ] **Extensibility and code:** Provider/processor boundaries can be tested without Tauri or OS APIs; the core has no Whisper-specific configuration leakage; every authored code file is below 300 lines.
- [ ] **Performance and packaging:** Measure the stated targets and disclose misses. Install/uninstall and startup cleanup are tested on native packages; distribution signing status is accurate.

Use unit tests for session transitions, stale-result rejection, phrase-command boundaries, settings validation, recovery expiry, and insertion-attempt deduplication. Use real recorded fixtures for inference and wake detection, including speech, silence, noise, accents, near-matching phrases, and stop words used as ordinary content. Use consented or synthetic fixtures with known redistribution rights, never the owner's private recordings by default.

Use integration tests for model load/download failures and real adapters. Use desktop tests for permissions, focus, clipboard, actual keystrokes, tray, login startup, and games. Test doubles are useful for domain failures but cannot prove a platform works. Formatting, lint, typechecks, and builds are supporting checks only. No unavailable OS test may be replaced with a passing mock and marked complete.

## 16. Instructions to the implementing model

Build the application described here, following `AGENTS.md`. Begin by inspecting the actual workspace and making a short implementation checklist. Treat the first release as local voice dictation with both customizable shortcut and voice activation. Keep the final-text insertion behavior, stopping semantics, truthful settings, and cross-platform constraints intact.

Work through the milestones and verify real behavior before claiming completion. Resolve routine implementation details from project evidence and current official documentation. Keep the code small and readable. Do not implement the optional future roadmap or weaken acceptance criteria to fit available tools.

If a required platform feature proves impossible with the available supported APIs, explain the exact limitation and present the smallest concrete alternative for the owner to decide. Continue everything independent of that decision. Report each required outcome as done, blocked, or unverified, with actual evidence.
