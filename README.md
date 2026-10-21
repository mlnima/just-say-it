<p align="center">
  <img src="assets/icon.svg" width="76" height="76" alt="Just Say It icon" />
</p>

<h1 align="center">Just Say It</h1>
<p align="center"><strong>Less typing. More flow.</strong></p>
<p align="center">Local desktop dictation for the place you're already writing.</p>

<p align="center">
  <a href="https://github.com/mlnima/just-say-it/actions/workflows/build.yml"><img src="https://github.com/mlnima/just-say-it/actions/workflows/build.yml/badge.svg" alt="Windows build and release workflow status" /></a>
</p>

<p align="center">
  <a href="https://github.com/mlnima/just-say-it/releases/download/v0.1.0/Just.Say.It_0.1.0_x64-setup.exe"><strong>Download the Windows preview</strong></a> ·
  <a href="https://github.com/mlnima/just-say-it/releases/tag/v0.1.0">Release notes & checksums</a>
</p>

<p align="center">
  <a href="#how-it-works">How it works</a> ·
  <a href="#get-started">Get started</a> ·
  <a href="#recent-changes">Recent changes</a> ·
  <a href="#download">Installation</a> ·
  <a href="#current-status">Status</a> ·
  <a href="#built-to-stay-light">Performance</a> ·
  <a href="#development">Development</a>
</p>

Just Say It turns speech into text inside a supported, focused input. Say your chosen wake phrase, use a keyboard shortcut, or click the small input button. Speak, finish the recording, and let local Whisper transcription do the typing.

> **Windows x64 preview.** The Windows application has recorded desktop and installer checks. macOS and Ubuntu native adapters remain unfinished.
>
> **Source is ahead of the download.** The published **v0.1.0** installer predates the [recent changes](#recent-changes). The features below describe the current source; newer functionality still has [verification gaps](#current-status).

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/audio-dark.png" />
  <source media="(prefers-color-scheme: light)" srcset="docs/images/audio-light.png" />
  <img src="docs/images/audio-dark.png" alt="Just Say It audio settings: microphone and local Whisper model choices, dictation language, and a closed-microphone status in the dark interface." width="1120" />
</picture>

*Earlier captures of the actual Windows interface in its light and dark palettes, with the microphone closed. Screenshots predate the newer device, retention, shortcut and overlay controls described below.*

## Why it exists

Writing a message, filling out a form, or taking notes often means stopping to type. Just Say It reduces that keyboard work by putting spoken words into the field you're already using. You can keep working in your browser or desktop application while dictation stays available from the tray.

Local processing and resource use are central to the design. Whisper does not run continuously to search for a command. Optional voice activation uses a separate, small keyword detector; the speech model transcribes after an activated recording finishes.

## What you get

| Capability | What it does |
| --- | --- |
| Your own activation phrase | Configure an English wake phrase without training a new model for each phrase. |
| Three ways to start | Use your saved wake phrase, the record/stop shortcut, or the floating input button. The default shortcut is **Ctrl+Shift+Space**. |
| Local multilingual transcription | Run Whisper on your computer with a selected language or explicitly chosen automatic language detection. |
| CPU or GPU selection | Choose CPU or an initialized GPU backend, with the actual availability and loaded-model request shown separately. |
| Model memory control | Keep the speech model ready with **Never unload**, or release it immediately or after a selected idle timeout. |
| An input button that keeps your focus | Offer **Dictate** above supported focused inputs; change it to **Finish** while recording. Detecting a field alone never starts capture. |
| A quieter input button | Click **×** to snooze Dictate for 15 minutes on the detected page, or by window when page identity is unavailable. |
| Optional screen feedback | Enable an edge glow and bottom status message for preparation, recording and transcription. The indicator lets clicks pass through. |
| Temporary text recovery | Review the latest result when insertion is rejected or uncertain, then deliberately choose another input or copy it manually. |
| A native tray app | Close Settings while the runtime stays available. Enable **Launch at login** to start in the tray after signing in. |
| Clear listening controls | **Pause** releases microphone capture and survives restarts. **Restart application** and **Quit application** are available in Settings and the tray. |
| System-aware appearance | Follow the native light or dark theme, with a dark fallback when the theme cannot be determined. |

## How it works

1. **Focus an input.** Place your caret in a supported browser field, editor, or other accessible text control.
2. **Activate.** Say your saved phrase, press **Ctrl+Shift+Space** or your custom shortcut, or click **Dictate** above the field.
3. **Speak.** Wait for recording to begin. When using a wake phrase, pause briefly after the phrase before dictating.
4. **Finish.** Click **Finish**, press the toggle shortcut again, use the tray, or use an enabled stopping rule.
5. **Get your text.** Audio capture stops and Whisper transcribes locally. The app rechecks the destination before attempting insertion. If it cannot safely use that target, the words remain temporarily available in recovery.

Text appears after the recording is finished; this build does not stream words into the field as you speak. It inserts text without pressing Enter or submitting a form.

## Get started

### Open the desktop app

Download the [Windows installer](#download), or run the application from source using the [development instructions](#development). Models are separate downloads; they are not included in the installer.

Ordinary launch goes to the tray. Open **Settings** from the tray menu. Launching a second instance opens the existing instance's Settings. You can also pass `--settings` to the executable to open the window immediately.

### 1. Choose your audio and speech model

In **Audio & model**, shown above:

1. Choose your **Microphone**. **Follow system default** is an explicit choice if you want the app to use the current default device when opening capture.
2. Download the catalog Whisper model, or import a compatible local GGML model file.
3. Select that model in **Local Whisper model**. Downloading it does not select it automatically.
4. Choose a **Dictation language**, or deliberately select automatic detection.
5. Click **Save settings**, then **Test saved microphone** to check the observed input level.

The microphone test runs for 15 seconds without speech recognition. If the app is paused, use **Resume** before testing. Device, model and language choices begin unset; placeholders are examples, not saved configuration.

**Unload speech model after** defaults to **Never unload**, which keeps the selected model in memory and prepares it at startup, including while listening is paused. Other choices are **Immediately**, **15/30 minutes**, **1–5 hours**, or **8/12/24 hours** of inactivity. Existing unset policies migrate to Never unload; explicit timeout choices are preserved. Keeping model weights in memory does not enable listening or retain past recordings.

**Transcription device** offers **CPU** and **GPU**. It begins unset; the shared runtime policy uses CPU until you save a choice. The app checks Whisper's registered GPU devices and initializes a backend before making GPU selectable. An unavailable GPU shows a reason. Saving a device change prepares the selected model before replacing the current settings/model; failed changes preserve them. Existing settings migrate without choosing GPU automatically.

The source enables Vulkan for Windows/Linux and Metal for macOS. The app uses the first GPU whose backend initializes. Loaded-model status distinguishes CPU from a GPU offload request; some operations can still use the CPU. A Windows build and backend initialization on one NVIDIA RTX 3090 are recorded. GPU transcription, device switching and performance remain unverified; Linux/macOS runtime support is still unfinished.

### 2. Set your activation cue

Open **Activation**. For voice activation, download or import the supported English wake model, select it, enter your **Wake phrase**, enable **Listen for my wake phrase**, and save. Use **Test saved phrase** to check detections without starting a dictation session.

Choose an uncommon phrase of several words. The current keyword model supports English phrases even when your dictation language is different. Other voices or audio from speakers can trigger it; it does not identify your voice.

**Record / stop shortcut** starts recording on one press and finishes/transcribes on the next. The runtime registers **Ctrl+Shift+Space** when no custom shortcut is saved. To change it, enter a modifier chord and save; **Reset to default**, followed by saving, removes the custom binding. The UI shows the actual registration separately from the saved value. The shortcut works independently of wake listening, once audio/model/language setup is complete and listening is resumed.

Use **Cancel dictation** in the tray to discard an utterance. Optional isolated stop phrases, silence timing, no-speech timeouts and recording limits are further down the Activation page. Toggle sessions finish manually unless a stopping rule is explicitly configured; wake sessions have a two-second silence timeout when unset. The shared recording limit is 60 seconds and the no-speech timeout is five seconds when those settings are unset. Hold-to-talk remains unfinished.

**For this build, say the wake phrase, pause briefly, then speak when recording begins. Immediate speech joined to the phrase is not supported yet.**

<details>
<summary>Earlier Activation screenshot</summary>

![Earlier Activation settings with English keyword model setup, an unset wake phrase, listening switch and phrase test.](docs/images/activation-dark.png)

This capture predates the single **Record / stop shortcut** control. The separate cancel-shortcut field shown here is no longer present.

</details>

### 3. Choose recording feedback

In **Input & recovery**, enable **Show the input button** and save. Focus a supported text field in another app: **Dictate** appears above it. Clicking the button preserves the field's focus. During recording it becomes **Finish**.

Click **×** beside Dictate to snooze it for **15 minutes** on the detected page. Apps without an accessible page identity are snoozed by window. Shortcuts and configured wake listening remain available; **Finish** still appears during recording. Snoozes are temporary and clear on restart.

Enable **Show the screen-edge overlay** separately for a translucent edge glow and bottom status message during preparation, recording and transcription. It follows the active monitor, lets clicks pass through, and hides after completion, cancellation or pause. Exclusive fullscreen games and protected displays may hide it. Both overlay windows request exclusion from supported Windows capture paths; visibility, focus behavior and capture exclusion still need dedicated runtime checks.

Button and wake-phrase dictation require a supported editable field. On Windows, the toggle shortcut explicitly selects the foreground window for keyboard typing, including custom text inputs that accessibility cannot inspect. Open the intended input before starting. The app checks the original window, process and native focus before delivery; when an editable field is detectable, it also checks that field. Known password inputs are refused. The app's own Settings fields are excluded as destinations.

For game chat, open and focus chat yourself, press the toggle shortcut to record, then press it again to finish. Shortcut delivery uses Windows scan-code events for characters on the target keyboard layout and Unicode events for other characters. It does not paste, press Enter, open chat or modify the game process. Games can reject simulated input; compatibility remains unverified until tested in that game. Without an accessible field, the app cannot confirm which custom field inside the window is active or that it accepted the text.

Open **Show latest text** to review a rejected or uncertain result. Check whether any text already arrived before choosing **Choose a new input** or copying manually. Automatic insertion refuses multiline/control-character results. Temporary text expires after **five minutes** and clears on pause, lock or exit. Starting another dictation replaces the previous result; pending recovery no longer blocks configured wake listening or a fresh recording.

<details>
<summary>Earlier Input & recovery screenshot</summary>

![Input and recovery settings showing the optional floating input button, observed target status, insertion limitations, and temporary text recovery.](docs/images/input-dark.png)

This capture predates the screen-edge overlay and snooze controls.

</details>

### 4. Choose your background behavior

In the installed Windows app, open **Background & privacy**, enable **Launch at login**, then click **Save settings**. After booting, the app starts in the tray when the current user signs in. This is an optional user-session registration; installation alone does not enable it. Disable the switch and save to remove that registration.

**Observed startup registration** shows whether registration is actually present. Starting the app never begins a dictation recording; previously enabled wake listening can resume only when ready and not paused. Registration was toggled and checked in recorded Windows tests, but an actual reboot/login cycle and uninstall cleanup with startup enabled remain unverified.

| Action | Result |
| --- | --- |
| Close Settings | Destroy the settings webview and keep the native tray/runtime available. |
| Cancel a recording | Discard the utterance; configured wake listening may resume. |
| Pause | Cancel active work, clear temporary text, close microphone capture, and remain paused across restarts until **Resume**. |
| Restart application | Relaunch from saved settings and retain installed models. Clear the recording, temporary recovery text, and unsaved edits. |
| Quit application | Exit the application and release capture and shortcuts. |

The privacy panel shows actual microphone, keyword-detector, and Whisper-memory status, plus the number of transcription runs in the current process.

<details>
<summary>Background & privacy screenshot</summary>

![Background and privacy settings with unset login startup, observed registration, restart and quit buttons, and actual closed-microphone and unloaded-Whisper status.](docs/images/background-dark.png)

</details>

## Recent changes

The current source includes these changes after [`9297f40`](https://github.com/mlnima/just-say-it/commit/9297f40bc155398acefd911ee7ae0793b857f19c) (18 October 2026). They are **not included in the published v0.1.0 installer**. This list describes source changes; the [validation record](docs/validation.md) distinguishes build/UI observations from functional testing.

| Area | Changes since that commit |
| --- | --- |
| Record/stop shortcut | One editable toggle binding, **Ctrl+Shift+Space** as the runtime default, reset-to-default, registration status, and removal of the separate cancel binding. Tray cancellation remains available. |
| Input detection and button | UI Automation focus notifications, inspection limited to the focused native host, read-only document handling, the app icon on Dictate, and a 15-minute page/window snooze. |
| Screen feedback | Optional monitor-aware edge glow and status label, separate from the input button; capture-exclusion requests and non-activating overlay handling. |
| Repeated dictation | A fresh activation replaces pending recovery, pending text allows wake monitoring, and intentional shortcut/button presses bypass the wake cooldown. Delivery waits briefly for shortcut modifiers to be released. |
| Windows keyboard delivery | Snapshot the original window/process/thread/native focus at the shortcut press, recheck before insertion, and use layout-aware scan codes with Unicode for other characters. Custom/game inputs remain application-dependent. |
| Speech model lifecycle | Configurable idle unloading, **Never unload** as the default, startup preloading, and an empty prepared inference state between sessions. CPU inference uses up to four available threads. Latency gains have not been measured. |
| Transcription device | Optional CPU/GPU preference, GPU backend probing and availability reasons, model preparation before committing device changes, Vulkan/Metal build features, and observed loaded-model request. |
| Settings and tray | Schema 4 migrations preserve valid preferences while removing the old cancel field and applying the retention default where needed. The native Windows tray menu has grouped actions and increased spacing. |
| Build and test tooling | Checksum-pinned Vulkan SDK/runtime setup, Ninja discovery, loader/license staging and installer resources, Linux loader dependency, and additional shortcut/migration/input-detection test fixtures. Their presence alone does not establish a passing runtime result. |

## Built to stay light

The runtime uses Rust, Tauri 2, native microphone capture, and a native input button. React and TypeScript provide the Settings interface. Closing that window destroys its webview.

- Wake listening uses a separate INT8 keyword model with one inference thread.
- Whisper transcribes after recording finishes using up to four available CPU threads, with GPU offload when selected and available. Loading and unloading follow the saved model-retention policy.
- Audio callbacks use a preallocated, bounded ring. Inference, downloads, and filesystem work stay outside the audio callback and UI thread.
- Input detection follows Windows accessibility focus events instead of scanning the screen.

**Historical baseline, 18 October 2026:** observations on a Windows 11 / Intel Core i9-9900K reference machine, before the model-retention and GPU changes:

| Mode | Observed working set | Observed CPU | Measurement |
| --- | --- | --- | --- |
| Tray idle, wake listening off | 21.15 MiB | 0.00521% of one logical core | Installed release, five minutes; no descendant processes. |
| Wake listening on, Whisper unloaded | 96.1 MiB | 4.74% of one logical core | Native development process with the release keyword engine, 30 seconds; excludes other processes. |

These measurements describe the earlier build with Whisper unloaded. The current **Never unload** default can retain model weights and empty inference buffers, increasing idle memory after a model is selected. Current CPU/GPU latency, memory, battery use and long-term stability need separate measurements. See the [validation record](docs/validation.md) for the environment and evidence.

## Your voice stays local

Wake recognition and speech transcription run on the device. Once models are installed, both work without a network connection. Model downloads contact their listed upstream sources only when requested.

The app has no cloud transcription, account system, analytics, automatic diagnostic uploads, or recording/transcript history. It keeps only bounded temporary audio for the current operation and the latest text for short-lived recovery. Dictation does not automatically replace clipboard contents, press Enter, send messages, or execute terminal commands.

When wake listening is enabled and ready, the microphone is open for keyword detection. **Pause** is the control that releases capture and keeps listening off across restarts.

## Current status

This is an initial Windows implementation, not the completed cross-platform release in the [project specification](project%20initialization.md).

| Target or capability | Status |
| --- | --- |
| Windows 11 x64 | Native implementation with recorded microphone, model, desktop, UI, packaging, theme, and restart checks. |
| Windows native inputs and Microsoft Edge | Earlier dedicated editor, browser-input and contenteditable fixtures received Unicode; changed/protected targets were exercised. Later keyboard-delivery changes require their own runtime checks. |
| CPU/GPU selection | The newer Windows release executable built and showed the device control; one RTX 3090 backend initialized. GPU transcription, device switching, no-driver startup and installer distribution of this change remain unverified. |
| New shortcut, overlay and retention behavior | Implemented in source. Recorded later builds and UI observations do not establish repeated dictation, idle-time expiry, screen capture behavior, focus preservation or game compatibility. |
| Live human wake-to-typing accuracy | Still needs broader end-to-end testing, including accents, noise, competing voices, and speaker audio. |
| macOS Apple Silicon and Intel | Native insertion/input detection and desktop validation remain unfinished. |
| Ubuntu X11 and Wayland | Native input adapters and desktop validation remain unfinished; these are separate compatibility cases. |
| Hold-to-talk | Unfinished; the record/stop shortcut is a toggle. Cancellation is available in the tray. |
| Public distribution | Windows preview installer is available from [GitHub Releases](#download) and is unsigned. Signing, complete distribution-license review, and the full release acceptance matrix remain outstanding. |

See [compatibility](docs/compatibility.md) for platform boundaries and [validation](docs/validation.md) for checks actually performed. Immediate post-wake speech, comprehensive stop-command handling, actual login/lock/suspend scenarios, games, elevated apps, keyboard layouts/IME, and the full install/upgrade/uninstall lifecycle still need acceptance evidence. A recorded owned-installation upgrade preserved settings; broader upgrade cases and uninstall cleanup with startup enabled remain unverified.

## Download

**[Download the Windows x64 preview installer](https://github.com/mlnima/just-say-it/releases/download/v0.1.0/Just.Say.It_0.1.0_x64-setup.exe)** · [Release notes and checksums](https://github.com/mlnima/just-say-it/releases/tag/v0.1.0) · [All releases](https://github.com/mlnima/just-say-it/releases)

### Install on Windows

1. Download and run the **`-setup.exe`**. This is a normal NSIS installer with a native uninstaller; the download is not a portable-only executable.
2. Launch the installed app and open **Settings** from the tray.
3. Download/select models and save your microphone, language and activation choices.
4. To start automatically after signing in, enable **Background & privacy → Launch at login** and save.

The published installer includes the native inference libraries. Users do not need source code, Node.js, Python, Rust or CMake. Models are separate downloads. If WebView2 is missing, setup may need an internet connection to install it. Installation, installed launch and native uninstallation have recorded Windows checks; the complete lifecycle matrix remains unfinished.

**The v0.1.0 download is the earlier preview.** Pushing newer source does not replace that installer. Build the current checkout to use the [recent changes](#recent-changes); a new tagged release is needed to publish an updated download.

This preview is **unsigned**, so Windows may show an unknown-publisher or SmartScreen warning. The release includes a `SHA256SUMS` file for checking the download; a checksum is not a publisher signature. See the [release notes](docs/release-notes.md) and [validation record](docs/validation.md) for current limits.

| Platform | Download status |
| --- | --- |
| Windows x64 | NSIS `-setup.exe` preview installer. |
| Ubuntu | `.deb` is the intended installer format; native input adapters and desktop validation remain unfinished, so no Linux download is published. |
| macOS Apple Silicon / Intel | `.dmg` is the intended download format; native input adapters and desktop validation remain unfinished, so no Mac download is published. |

## Development

### Prerequisites

The documented development environment is Windows 11 x64 with:

- Visual Studio 2022 C++ desktop tools and a Windows SDK.
- CMake and the pinned Rust **1.95.0** toolchain.
- Node.js **24.19.0** and npm **11.6.1**.
- Python for the development-only libclang package used to generate native bindings.
- The Vulkan SDK for Windows/Linux GPU builds (`VULKAN_SDK` on Windows); macOS uses its Metal toolchain. Windows setup is pinned in `scripts/vulkan.json`.

End users do not need Python, Node.js, Rust, or CMake.

From the repository root:

```powershell
npm ci
python -m pip install --target artifacts/build-tools libclang==18.1.1
powershell -NoProfile -File scripts/install-vulkan-sdk.ps1
npm run desktop
```

`scripts/build-environment.mjs` discovers installed MSVC and Windows SDK headers. On Windows it uses the checkout-relative development libclang directory when present; an explicit `LIBCLANG_PATH` takes precedence. Other systems use their installed libclang without a Windows-specific path override. `.cargo/config.toml` disables build-host CPU tuning for Whisper. Use the provided wrappers for native commands. `.node-version`, `packageManager` in `package.json`, and `rust-toolchain.toml` pin the development tool versions.

The Vulkan setup script copies the checksum-pinned SDK under ignored `artifacts/build-tools` using LunarG's documented `copy_only=1` mode, without requiring administrator access. `build-environment.mjs` discovers that SDK unless `VULKAN_SDK` is explicitly set, and uses Visual Studio's bundled Ninja when available and no `CMAKE_GENERATOR` was supplied. This avoids MSBuild's file-tracker path-length limit in the nested Vulkan shader build. Windows build wrappers also prepare a checksum-pinned LunarG runtime archive. The native build stages its architecture-matched `vulkan-1.dll` and license beside the executable, and packaging includes them. This supplies the loader even when no GPU driver is installed; GPU availability still requires a compatible driver/device. No system graphics driver is installed or modified. Linux builds require Vulkan headers, loader and `glslc` (for example, Ubuntu's `libvulkan-dev` and `glslc` packages).

### Build and check

| Command | Purpose |
| --- | --- |
| `npm run build` | Strict TypeScript check and production frontend build. |
| `npm run test:core` | Desktop-independent core tests. |
| `node scripts/cargo.mjs test --workspace` | Workspace tests; real-device/model/desktop tests remain ignored unless explicitly enabled. |
| `npm run check:files` | Verify that each authored code file contains fewer than 300 lines. |
| `node --test scripts/tests/build-environment.test.mjs` | Check actual tool discovery and explicit environment overrides. |
| `npm run package` | Build a Windows NSIS test installer with its matching native inference libraries. |
| `node scripts/package.mjs --no-bundle` | Build the release executable and adjacent native libraries without an installer. |

The Windows installer is written to `target/release/bundle/nsis/Just Say It_0.1.0_x64-setup.exe`. Packaging does not publish or sign it. Keep the adjacent native DLLs with an unpackaged executable. Quit a running development app before rebuilding native dependencies because Windows locks loaded DLLs.

Windows packaging discovers Cargo's actual output directory, including `CARGO_TARGET_DIR`, and uses the selected `--target` and `--debug` options for the installer and its native DLLs. To pass options reliably from PowerShell, invoke the script directly. Cargo arguments follow `--`, for example `node scripts/package.mjs --ci -- --locked`.

### GitHub releases

The [build workflow](.github/workflows/build.yml) runs on fresh Windows runners for branch pushes and pull requests. It installs the pinned tools, checks file sizes and build tooling, runs workspace tests, and builds an installer. Real microphone/model/desktop tests remain separate, with prerequisites documented below. The workflow saves the installer, notices and SHA-256 checksum as a build artifact.

To publish a new Windows preview, update the matching versions in `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `Cargo.lock`, and `src-tauri/tauri.conf.json`, update the download links and release notes, commit and push, then create and push a new matching `v<version>` tag. The existing `v0.1.0` tag is already published. Branch pushes save CI artifacts; they do not update release downloads.

Only a successful tagged build publishes a GitHub **prerelease** with the installer and checksums. The tag must match the frontend, native package and Tauri versions. Publishing uses the repository's built-in Actions token with write permission limited to the release job. No personal token or computer-specific path belongs in the workflow. Reusing an existing release tag fails instead of replacing downloads silently.

macOS and Ubuntu releases require completion and verification of their native functionality before adding those packages to the release workflow.

Real adapter and desktop tests require the catalog models, locally generated speech fixtures, a real microphone, Microsoft Edge, Playwright, and an unlocked idle desktop:

```powershell
powershell.exe -NoProfile -File tests/desktop/generate-audio.ps1
# Place the catalog models in artifacts/models; see docs/validation.md.
# Set PLAYWRIGHT_MODULE_PATH if Playwright is supplied externally.
node scripts/cargo.mjs test --workspace -- --include-ignored --test-threads=1
```

Do not type while these tests own their dedicated fixture windows. They verify the fixture process before dispatching text. The Windows wrapper copies matching native libraries, including the Vulkan loader, beside the test executables; models, audio, browser profiles, and build output stay under ignored directories. Additional theme, restart, settings, runtime, and download checks are documented in the [validation record](docs/validation.md). New shortcut and input-detection fixtures live under `tests/desktop`; adding a fixture is not a record of executing it.

### Configuration and source map

Saved Windows preferences live at `%APPDATA%\app.justsayit.desktop\settings.json`. Installed models use the OS application data directory. Settings use schema **4**, migrate supported earlier schemas once, and are written atomically. Invalid or unreadable existing settings are preserved and block overwriting until recovered deliberately.

| Location | Responsibility |
| --- | --- |
| `crates/dictation-core` | Provider-independent settings, session, authorization, and speech-activity rules. |
| `src-tauri/src` | Audio, activation, local models, native platform adapters, delivery, tray, and lifecycle. |
| `src` | Typed command/event bridge and the Settings/status/recovery UI. |
| `crates/dictation-core/src/defaults.rs` | Shared runtime fallback policies, limits, and timings. |
| `src-tauri/tauri.conf.json` | Application identity and desktop configuration. |
| `src-tauri/model-catalog.json` | Pinned model sources, sizes, checksums, licenses, and language metadata. |
| `tests/desktop` | Dedicated desktop fixtures and real-runtime UI checks. |
| `docs` | Decisions, compatibility, validation, notices, and README screenshots. |

Rust owns runtime state and validation. New speech providers and optional text processing belong behind small explicit interfaces; roadmap items do not imply implemented features. See the [implementation decisions](docs/decisions.md) and [agent instructions](AGENTS.md) before making changes.

## License and notices

An application license has not been selected. See [third-party notices](docs/THIRD-PARTY-NOTICES.md) for component attribution and distribution-review status. Models are optional separate downloads with their own license information shown in the interface.
