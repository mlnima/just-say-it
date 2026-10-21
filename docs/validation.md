# Validation record

18 October 2026. This records the initial Windows build; it does not mark the original release checklist complete.

## Environment

- Windows 11 Pro, 10.0.26300, x86-64, signed-in interactive desktop.
- Intel Core i9-9900K, 16 logical processors.
- Microsoft Edge 155.0.4283.62.
- Rust 1.95.0 / MSVC 2022, Node 24.19.0, npm 11.6.1, CMake 4.1.0-rc1.
- Speech: whisper-rs 0.16.0, whisper.cpp model `ggml-tiny-q5_1.bin`, SHA-256 `818710568da3ca15689e31a743197b520007872ff9576237bda97bd1b469c3d7`.
- Keywords: sherpa-onnx 1.13.8, English GigaSpeech 3.3M INT8 model; archive SHA-256 `f170013b4716e41b62b9bfd809687c207cef798ef9bc6534d524e17af9b6561a`.
- Development fixtures were synthesized locally with Windows speech synthesis at mono 16 kHz, PCM16. No private microphone recording was written to disk.

## Executed checks

| Check | Actual result |
| --- | --- |
| Production frontend build | Passed strict TypeScript and Vite production build; approximately 242 kB JavaScript and 9 kB CSS before compression. |
| Code-file size | Every authored code file contains fewer than 300 lines. |
| Eight core tests | Passed cancellation/stale-result rejection, insertion deduplication, recovery expiry, pending-delivery blocking, null settings round-trip, null-character rejection before native calls, timing/phrase validation, silence policies, and control-character refusal. |
| Settings adapter | Passed atomic save/round-trip and preservation/refusal to overwrite an unreadable original. |
| Real keyword model | Two different configured English phrases were recognized without retraining; known silence produced no keyword event. |
| Real Whisper model | Recognized the expected synthetic utterance; an invalidated authorization caused cancellation/error rather than a late result. |
| WebRTC VAD | Known synthetic speech was detected; known silent frames were rejected. |
| Real microphone | Opened the system default device, produced resampled frames, and released the capture object. This alone does not prove a live human utterance's accuracy. |
| Native editor | A dedicated WinForms editor received accents, emoji and right-to-left text. A changed field refused the old target; password and read-only fields were refused. Actual destination text was inspected. |
| Microsoft Edge | Dedicated browser inputs and contenteditable received Unicode; changed-field delivery was refused. Actual DOM values were inspected. |
| Combined suite | Final combined run passed all 15 core/adapter/desktop tests, with ignored prerequisites deliberately enabled and tests serialized. |
| Real settings webview | Passed empty initial dropdowns, saving, reload persistence, editable phrase, pause, and observed closed microphone. Labels were exercised through accessible names. |
| Floating input button | With voice activation disabled, actual mouse clicks started recording and finished it. The foreground browser window remained unchanged. |
| Own settings exclusion | The actual native foreground settings window and its focused wake textbox were excluded as dictation targets; no input button appeared. This also covers WebView2 using a different accessibility provider process ID from the native app. |
| Model downloads | Both catalog models downloaded through the actual settings UI, passed size/checksum/native-load checks, installed, and appeared in the UI. Downloading did not silently select a model or change saved settings. |
| Background runtime | Closing settings kept the native runtime and wake listening alive; a second launch reopened the same running app's settings. No additional Whisper inference occurred during the idle observation. |
| Pause/resume | Actual capture closed on pause; resume restored the selected keyword detector without retaining an idle Whisper model. |
| Login startup | The real current-user startup registration was toggled and its observed state checked, then the original registration was restored. An actual OS logout/login cycle remains unverified. |
| Quit | The real settings Quit action closed the window and the process was no longer present. |
| Windows package | Final NSIS installer completed with exit code 0 in the owned test directory. Ordinary installed launch created no settings webview; bundled settings, empty wake phrase, closed microphone, second launch, close-to-tray, and Quit all passed. |
| Uninstall | The owned preceding test installation's native uninstaller exited with code 0 and removed its executable. The final installer was then installed afresh. Settings-preserving upgrades and uninstall with enabled startup still need dedicated checks. |

The external fixtures verify their own process identity before dispatch. Focus-dependent tests wait for actual fixture keyboard focus, not simply a successful OS dispatch. Earlier attempts exposed harness setup/focus issues, including cursor restoration during a focus test and a settings textbox focused only inside its webview while another application remained foreground. The corrected checks explicitly acquire and verify the owned fixture/window. Tests were not replaced with simulated adapters.

A live microphone can contain ambient sound. The deliberate button session may invoke Whisper on that audio; it is not a controlled silence fixture. Idle inference is checked by comparing the real run counter before and after the idle interval. Known silence is tested independently through the actual speech-activity and keyword adapters.

## Initial resource observations

With settings closed, voice listening enabled, and Whisper unloaded, the Windows development runtime's **native process** was observed over 30 seconds:

- CPU: 4.73958% of one logical core (approximately 0.296% of the reference machine's aggregate CPU capacity).
- Native process working set: 100,810,752 bytes (96.1 MiB).
- Native process private bytes: 62,738,432 bytes (59.8 MiB).
- No additional Whisper inference runs during the observation.

These are development-build observations with the release native keyword engine. They exclude any other processes and do not establish the full release performance targets, battery impact, p50/p95 transcription latency, or 100-session memory stability. Settings-open WebView2 resources must be accounted for separately.

The installed release, with wake listening off and no settings UI, was observed for 300.02 seconds: 0.00521% of one logical core, 22,175,744 bytes working set (21.15 MiB), and 5,804,032 private bytes (5.54 MiB). No descendant processes existed. This measurement preceded the final own-window exclusion fix. Its raw `visible_settings_window` flag used `Process.MainWindowHandle`, which identified a framework message window; native class inspection confirmed there was no settings window. The measurement helper now excludes those internal windows.

After opening and closing the installed settings UI, a separate 30.02-second observation measured 0.1041% of one logical core, 35,524,608 bytes native working set (33.88 MiB), and 9,908,224 private bytes (9.45 MiB). There were no descendant processes, the genuine settings window was absent, and WebView2 had exited. These short measurements are not battery or long-term stability results.

The final installed binary was observed separately for 30.02 seconds before opening settings: 20,865,024 bytes working set (19.90 MiB), 5,812,224 private bytes (5.54 MiB), no descendants, and no settings window. No CPU time increment was measurable at the OS counter's resolution during that short interval; this is not an absolute zero-CPU claim.

## Windows test distribution

`target/release/bundle/nsis/Just Say It_0.1.0_x64-setup.exe` is 9,481,938 bytes (9.04 MiB). The application and native libraries occupy approximately 37 MB before filesystem overhead; models are separate optional downloads. Authenticode reports **NotSigned**. No public release or signing claim is made.

The three installed native DLLs match the current release files by SHA-256. The installed executable differs from the current compiled executable only at Tauri's three-byte NSIS bundle marker (`UNK` to `NSS`), as defined in the locked Tauri utility source. The final installed settings UI was checked without development remote-debugging flags. It was left open with preferences unset; test settings and downloaded models remain isolated under ignored artifacts directories.

## Reproduction

Generate fixtures with `powershell.exe -NoProfile -File tests/desktop/generate-audio.ps1`. Obtain the exact two catalog models into:

```text
artifacts/models/ggml-tiny-q5_1.bin
artifacts/models/sherpa-onnx-kws-zipformer-gigaspeech-3.3M-2024-01-01/
```

Use the source URLs and checksums in `src-tauri/model-catalog.json`. Do not save those models or generated audio in version control.

```powershell
$env:PLAYWRIGHT_MODULE_PATH = '<installed Playwright module directory>'
node scripts/cargo.mjs test --workspace -- --include-ignored --test-threads=1
```

For isolated actual-runtime UI checks, build with `--no-default-features`, launch with `DICTATION_TEST_DIRECTORY` set to a new directory under artifacts and `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9223`, and pass `--settings`. The test-only directory override is ignored in release builds. Run `tests/desktop/settings-test.mjs` with the Playwright module directory; then run `runtime-test.mjs` with that directory and the actual app PID. The latter uses real microphone capture and dedicated external browser input; it restores startup registration and finishes paused. `download-test.mjs` checks both actual catalog downloads in an isolated directory without previous downloads. Never enable remote debugging for the distributed build.

## Remaining release evidence

Live human wake-to-dictation-to-external-text accuracy, immediate speech after the wake phrase, comprehensive stop-command boundaries, noise/competing voices/speaker audio, actual login, lock/suspend and revocation cases, game compatibility, non-US physical layouts/IME/selection replacement, all download failure modes, settings-preserving upgrades, and startup cleanup after uninstalling an enabled setup remain unverified. macOS and Ubuntu native input adapters are unfinished and their desktop results are unavailable. Hold-to-talk is unfinished. Public signing/notarization and a complete distribution license review remain outstanding.

The original acceptance checklist stays unchecked; individual passing probes do not establish the completed release.

## System theme follow-up — 18 October 2026

The settings UI now has light and dark palettes. It reads the native window theme and subscribes to changes using the installed `@tauri-apps/api` 2.12.4 APIs, matching the official [theme query](https://v2.tauri.app/reference/javascript/api/namespacewindow/#theme) and [theme event](https://v2.tauri.app/reference/javascript/api/namespacewindow/#onthemechanged) documentation. Missing or rejected theme queries retain the dark base palette. Theme detection does not save a preference or change the OS theme.

| Check | Actual result |
| --- | --- |
| `npm run build` | Passed strict TypeScript and the production frontend build. |
| `node scripts/cargo.mjs build -p just-say-it --no-default-features` | Built the Windows development runtime with the narrow native theme-read permission. |
| `npm run check:files` | All authored code files remain below 300 lines. |
| `npm run package` | Built the release frontend/native executable and Windows NSIS installer (9,482,078 bytes). The existing owned installation under `artifacts/installed-app` was updated with `/S /UPDATE`, exit code 0, and reopened without debugging flags. The settings file was unchanged. |
| `node tests/desktop/theme-test.mjs <Playwright module directory>` | Passed against an isolated real Windows settings webview and a separate hook fixture. The real native theme was dark, and the UI selected dark automatically. Both palettes were inspected across all four settings sections, including native form-control colors; saved preferences remained unchanged. |
| Theme hook fixture | Tauri's IPC test adapter verified light/dark event updates, dark fallback for null and failed queries despite a light browser preference, rejection of stale query results, and listener cleanup under React StrictMode. These are simulated theme-source cases, not actual OS theme switches. |
| Light appearance preservation | Resolving the new light palette variables reproduced the previous layout and control styles exactly. Dark and light screenshots were visually inspected. |
| Updated installed UI | An actual foreground-window screenshot showed the dark palette; sampled content and sidebar pixels matched the authored colors. The installed executable matches the rebuilt release except the expected three-byte NSIS bundle marker. The updated settings window was left open. |

The desktop test requires the development Vite server and the isolated WebView2 debugging setup documented above. Screenshots are under ignored `artifacts/theme-change`. This follow-up does not establish macOS/Ubuntu theme behavior; those runtime results remain unverified.

## Restart controls follow-up — 18 October 2026

**Restart application** is available in Settings under **Background & privacy** and in the native tray menu. Both send the same runtime action, invalidate delivery authorization, signal worker/download shutdown, and call Tauri 2.12.4's [`request_restart`](https://docs.rs/tauri/latest/tauri/struct.AppHandle.html#method.request_restart). The exit events release the single-instance registration before relaunch. Saved settings and models are retained; in-memory sessions and unsaved settings edits are cleared. Relaunch preserves the original startup arguments, including tray-only startup.

| Check | Actual result |
| --- | --- |
| `npm run build` | Passed strict TypeScript and the production frontend build. |
| `node scripts/cargo.mjs build -p just-say-it --no-default-features` | Built the Windows development executable. |
| `rustfmt --check src-tauri/src/runtime.rs src-tauri/src/worker_control.rs src-tauri/src/tray.rs` | Passed for the changed native files. |
| `npm run check:files` | All authored code files remain below 300 lines. |
| `node tests/desktop/restart-test.mjs <Playwright module directory>` | Passed against the real Windows runtime: the Settings button replaced the process during an actual microphone test, restored saved values instead of unsaved edits, and left the microphone closed. The saved settings file was identical before and after restart. |
| Native tray restart | The same test opened the native right-click menu with Settings closed, resolved **Restart application** by label, and dispatched its native menu command. A new process retained the saved pause and preferences with the microphone closed. A separate ordinary tray-only launch also restarted into the tray; reopening Settings reused that single new process. |
| `npm run package` | Built the release executable and Windows NSIS installer (9,485,575 bytes). The existing owned installation under `artifacts/installed-app` was updated with `/S /UPDATE`, exit code 0. |
| Updated installed application | Launched without debugging flags. Windows accessibility invoked the actual Settings navigation and restart button; the process changed. The actual native tray restart changed the process again. Both retained the original settings file's SHA-256, and exactly one installed application process remained. |

The test uses isolated debug settings under ignored `artifacts/restart-*`, the development Vite server, the existing native window helper, and WebView2's local debugging port. The tray helper sends the shell's right-click notification to the owned native tray window and dispatches the command found in the actual menu; it does not physically click the taskbar icon. An earlier coordinate-based menu test selected unreliably, so the passing test uses native menu command IDs. This is Windows runtime evidence; macOS/Ubuntu restart behavior remains unverified.

## Build portability and installer release — 18 October 2026

The authored source, configuration, build scripts and desktop fixtures were searched for personal profile names, absolute Windows drive paths and Unix home-directory paths. None were found. Recorded benchmark hardware in this document and the README is evidence about the test environment, not runtime configuration. Application data uses Tauri's OS-specific directory; microphone selection uses discovered devices or an explicitly saved system-default choice.

The Cargo configuration no longer forces a Windows development libclang path on every platform. The Windows build wrapper discovers the installed MSVC/SDK headers and uses checkout-relative development libclang only when installed; explicit environment overrides take precedence. Other platforms retain system libclang discovery. `GGML_NATIVE=OFF` remains set to avoid tuning Whisper to the build machine's CPU; this alone does not prove compatibility with every x64 CPU.

Windows packaging now resolves Cargo's output directory and preserves the selected target/debug options when bundling native libraries. The former fixed `target/release` resource map was removed. The default Windows release path was executed below; alternative targets and custom output directories were inspected in code but not built in this follow-up.

| Check | Actual result |
| --- | --- |
| `node --test scripts/tests/build-environment.test.mjs` | Both tests passed, including actual installed-header/libclang discovery and preservation of explicit environment overrides. |
| `npm run build` | TypeScript and Vite production build passed. |
| `npm run check:files` | Every authored code file remained below 300 lines. |
| `node scripts/cargo.mjs test --workspace --locked` | Eight core tests and the atomic-settings adapter test passed. Six model/device/desktop tests were explicitly reported ignored. |
| `node scripts/cargo.mjs test -p just-say-it --lib adapter_tests:: -- --include-ignored --skip microphone_capture --test-threads=1` | Four tests passed against the real keyword and Whisper models, synthetic speech/silence and atomic settings. Microphone and external-window insertion were not rerun here. |
| `node scripts/package.mjs --ci -- --locked` | Release compilation and NSIS bundling passed; installer size 9,484,233 bytes. |
| `node scripts/release-assets.mjs` | Prepared the matching installer, notices and SHA-256 checksum. A separate invocation with a mismatched tag was rejected before preparing downloads. |
| Existing-installation upgrade | The project-owned installation under `artifacts/installed-app` was upgraded with `/S /UPDATE`, exit code 0. The three installed native DLLs matched the release DLLs by SHA-256, and bundled notices were present. |
| Installed application | Ordinary launch stayed in the tray. A second launch opened the bundled Settings UI, which restored the saved wake phrase and showed closed microphone capture. Closing Settings preserved the runtime; Quit exited it. The settings file's SHA-256 was identical before and after the upgrade and launch checks. The owned Settings window was reopened afterward. |
| Signature | Authenticode reported `NotSigned`; signing and complete distribution-license review remain outstanding. |
| [Fresh GitHub Windows build](https://github.com/mlnima/just-say-it/actions/runs/39161539355) | Passed on `windows-2022` using a fresh checkout: pinned tool installation, file-size checks, both build-tool tests, frontend build, workspace tests, release compilation, NSIS packaging and artifact upload. No local development directories or models were supplied. |
| [Tagged GitHub release workflow](https://github.com/mlnima/just-say-it/actions/runs/39161835676) | Both build and release jobs passed for `v0.1.0`. The public Windows prerelease includes the CI-built installer, notices and checksum file. The completed clean-build logs contained no native-binding fallback messages. |
| Release filename and checksum | GitHub normalized spaces in the first uploaded installer name to dots. The release preparation script now stages that safe filename explicitly, the README link was corrected, and the published checksum entry was corrected to match the uploaded filename without changing the installer. The updated preparation script was executed locally and its staged filename/checksum verified against the actual file. |
| [Published Windows installer](https://github.com/mlnima/just-say-it/releases/tag/v0.1.0) | Downloaded the actual GitHub installer (9,493,230 bytes), notices and checksum file. All downloaded files matched their GitHub SHA-256 asset digests; the checksum entry also matched the installer. Installer SHA-256: `2c07d3b568210640ae54e5dbc31e16d1c92dd31a5210784a89e4377ac45d08dd`. |
| Published installer runtime | The downloaded CI-built installer upgraded the owned installation with exit code 0. Installed native DLL hashes matched the pinned libraries. Tray-only launch, second-launch Settings, saved phrase restoration, closed microphone status, close-to-tray and Quit passed. The saved settings file's SHA-256 stayed unchanged; the owned Settings window was reopened afterward. Signature remains `NotSigned`. |

The initial installed-UI probe expected an unset wake phrase and failed because this installation already had a saved phrase. An upgrade-specific check then verified the displayed phrase against the existing settings and the settings file's hash before/after the actual upgrade. The fresh-install test was not changed or marked passing. The upgrade-specific check is saved under ignored `artifacts/check-upgrade.ps1`.

The first published-download probe exposed the filename normalization issue above. A subsequent transport interruption left a partial local download; it was never executed. The final download check verified complete file sizes and SHA-256 digests before running the installer. The replaced checksum asset was retrieved using its current asset ID to avoid a cached response for the preceding filename entry. The download check is under ignored `artifacts/download-release.ps1`.

PowerShell's installed npm shim did not reliably forward the workspace arguments in `npm run cargo -- test --workspace --locked`. The direct Node wrapper succeeded; CI and the README now use it when passing native arguments. Models, recordings, native tools, installed copies, credentials and build outputs remain excluded from Git.

The release workflow uses fresh Windows runners, pinned Node/npm/Rust/libclang versions, workspace tests and NSIS packaging. Tagged builds validate the release version before publishing a Windows prerelease with an installer, notices and checksums. macOS and Ubuntu native adapters and runtime validation remain unfinished; no Mac or Linux installer is claimed.

## Screen activity overlay — 19 October 2026

Source implementation adds an optional Windows screen-edge glow and bottom status label, independently controlled by **Input & recovery → Show the screen-edge overlay**. The preference starts unset/off and is persisted through the existing settings store. Preparing, recording, transcription and insertion use the runtime phase; idle, recovery, cancellation, pause and shutdown hide the indicator. Transcription errors from shortcut/tray commands now terminate the failed session instead of leaving a processing phase active.

The native adapter uses four narrow edge windows and one label window, per-pixel premultiplied alpha, click-through/no-activation styles, and a dedicated message thread. It follows the foreground monitor, queries DPI from its own per-monitor-aware window and redraws on phase/display changes. It does not require a settings webview or an accessible input field. Initialization/rendering failures are reported in Settings; non-Windows adapters report the overlay unavailable.

API contracts were inspected against the installed `windows` 0.61.3 bindings and Microsoft's [layered-window hit testing](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features#layered-windows), [UpdateLayeredWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-updatelayeredwindow), [premultiplied alpha](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-blendfunction), [DIB synchronization](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createdibsection), [thread DPI awareness](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setthreaddpiawarenesscontext) and [window DPI](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdpiforwindow) documentation. Borderless desktop composition and exclusive fullscreen have different presentation paths; see Microsoft's [flip-model guidance](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/for-best-performance--use-dxgi-flip-model) and [fullscreen guidance](https://learn.microsoft.com/en-us/windows/win32/direct3darticles/dxgi-best-practices). This desktop overlay does not guarantee visibility over exclusive fullscreen games or protected displays.

Only code and diff inspection were performed for this change. No tests, builds, lint/type checks, file-size commands, app launches, microphone access or interactive desktop validation were requested or performed. Compilation, visual appearance, focus/click-through behavior, multiple monitors/DPI, saved-setting persistence and fullscreen/game compatibility remain **unverified at runtime**. The earlier validation results above do not cover this overlay.

## Speech model retention — 19 October 2026

Code inspection found that the speech worker retained Whisper weights between sessions but unloaded them after 60 seconds without inference; pausing also dropped the model. This establishes the previous lifetime policy, not the cause or duration of every reported delay. Whisper still creates a fresh inference state per transcription.

**Audio & model → Unload speech model after** now accepts immediate unload, 15/30 minutes, 1–5 hours, 8/12/24 hours, or **Never unload**. An unset preference uses the Rust-owned one-hour default and remains visibly unset with an explanation. The optional field can be absent in existing settings without rewriting them. Rust validates the presets and supplies them through the typed bridge.

The worker retains the selected model until its idle deadline, measures inactivity after use, and excludes active dictation from unloading. Immediate mode releases it after the session. Never unload preloads the saved model on the worker thread at startup and when applying that preference, including while listening is paused. Pausing still releases capture and temporary audio; retaining model weights does not enable listening. Preparing a changed model during settings application now reuses that loaded model after a successful save, preserving the previous model on failure. With no capture or temporary work, the worker waits for its deadline instead of periodically polling solely because a model is loaded.

| Requested check | Observed result |
| --- | --- |
| `node scripts/package.mjs --ci -- --locked` | Passed TypeScript, the Vite production build, optimized Windows native compilation, and NSIS packaging. The final build was rerun after the idle-pause timer correction and startup-preload error-path change. |
| Application replacement | Quit the previous release process through its Settings Quit action, then launched the rebuilt `target/release/just-say-it.exe --settings` without debugging flags. Exactly one application process remained, at that executable path (PID 35040). |
| Requested settings view | Windows accessibility found **Unload speech model after** in the real bundled release UI and scrolled it into view; it reported visible, with **Not selected** for the previously absent preference. The settings file's SHA-256 stayed unchanged during replacement. |

Code and final diffs were inspected. No unit suites, microphone tests, input injection, or elapsed idle-time tests were requested or run for this change. Timeout expiry, repeated real dictation, Never unload startup preloading, and measured latency improvements remain unverified at runtime; successful compilation and opening the settings control do not establish those outcomes. The reopened application uses the user's existing listening preferences.

## Dictation reactivation and overlay capture — 19 October 2026

Starting a dictation through the shortcut, input button or configured wake phrase now deliberately replaces a pending delivery result. Recovery remains available until another dictation starts, expires or is discarded. Pending delivery no longer suspends configured wake monitoring or hides the Dictate button. The cooldown still applies to wake activation; it no longer ignores an intentional shortcut/button press after the preceding session finishes. Native insertion waits up to 1.5 seconds for finish-shortcut modifiers to be released, checking cancellation and the original input throughout, before attempting the input operation once.

The input button consumes mouse activation without activating itself or forwarding its mouse-down to the default window procedure; mouse-up still performs its existing action. The passive screen indicator remains click-through and non-activating. Both overlays now request `WDA_EXCLUDEFROMCAPTURE` before display, and report an initialization failure if Windows refuses that request. This uses the installed `windows` 0.61.3 bindings and Microsoft's [mouse activation contract](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-mouseactivate) and [capture-exclusion API](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowdisplayaffinity). Capture exclusion applies to supported Windows capture paths; it is not a guarantee against every recorder or a way to display over exclusive fullscreen.

The user explicitly requested implementation, a build and an application launch, and reserved all functional testing for themselves. No tests, dictation sessions, microphone tests, keyboard/mouse injection, focus reproduction or screen recordings were performed. Actual shortcut reactivation, focus preservation, recording feedback, capture exclusion and ARK chat delivery remain **unverified at runtime**. Existing target-verification and security restrictions still apply.

| Requested action | Observed result |
| --- | --- |
| `node scripts/package.mjs --ci --no-bundle -- --locked` | Passed TypeScript, the Vite production build and optimized Windows native compilation. Produced `target/release/just-say-it.exe`; native compilation completed in 3 minutes 18 seconds. |
| Application replacement | The preceding process (PID 35040) exited through its own Quit action. The rebuilt executable was launched without Settings or debugging arguments; process inspection found one app process at the requested release path (PID 10596). |
| Existing settings | Enabled the requested screen indicator by atomically changing only `screen_overlay` to `true`. All other saved preferences were preserved, and the preceding settings file was backed up under ignored `artifacts/dictation-overlay-fix`. The app was launched normally to use those saved preferences. |

## Shortcut keyboard typing — 19 October 2026

The user explicitly requested ordinary keyboard simulation for game chat, including when the game does not expose an accessible text field. The global toggle shortcut now captures the foreground window, process, thread and native focused control at the keypress and passes that snapshot to the worker. Delivery rechecks that identity without focusing another window or attaching to the target process/thread. Detectable editable fields retain their field-identity checks, and known password controls are refused. A missing or outdated initial snapshot is rejected rather than replaced with a later foreground window. This window-scoped workflow applies only to explicit shortcut activation; wake phrases and the input button do not initiate delivery into an uninspectable input.

Shortcut delivery uses Windows `SendInput` with scan-code key-down/key-up events for characters available on the target thread's current keyboard layout. Mapping checks Caps Lock and verifies the resulting character without changing keyboard/dead-key state. Characters that cannot be represented by a normal key or Shift combination use Unicode input, preserving multilingual text without sending Ctrl/Alt command chords or changing the user's keyboard language. Partial dispatch releases unmatched synthetic key-downs without retrying text. No clipboard operation, Enter, chat-opening action, process memory access, DLL injection, in-process hook, elevation or anti-cheat bypass was added.

API contracts were checked against installed `windows` 0.61.3 bindings and Microsoft's [scan-code event documentation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-keybdinput), [character-to-key mapping](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-vkkeyscanexw), [scan-code mapping](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-mapvirtualkeyexw), [target keyboard layout](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getkeyboardlayout), and [non-mutating character translation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-tounicodeex).

The user must open and focus the intended game chat/input before starting and keep it active through delivery. Without an accessible field, the adapter cannot verify which custom field inside the same native window is active or confirm that the game accepted the keys. Simulated-input rejection and characters unavailable on the active layout remain compatibility boundaries. The user reserved functional testing: no tests, dictation sessions, key injection or game interactions were performed. Actual ARK acceptance, layout behavior, partial-dispatch cleanup and focus preservation remain **unverified at runtime**.

| Requested action | Observed result |
| --- | --- |
| `node scripts/package.mjs --ci --no-bundle -- --locked` | Passed TypeScript, Vite production assets and optimized Windows native compilation; the native release build completed in 3 minutes 48 seconds. |
| Application replacement | The preceding process (PID 10596) exited through its Quit action. The rebuilt release executable was launched without Settings/debugging arguments; process inspection found one app process at `target/release/just-say-it.exe` (PID 63220). |
| Current configuration | No saved preferences were edited for this change. Normal startup uses the existing microphone, models, language, activation and overlay settings. |

## Model retention default and preparation — 19 October 2026

Before editing, the current user's settings file contained `model_retention: "never"`; a read-only accessibility inspection of the running settings control also reported **Never unload**, and the observed model status reported **Loaded in memory**. The earlier reported unset selection was not present at inspection time. Code inspection showed that Never unload skips the idle-unload path, and the dictation path calls local Whisper with no network request. The network code inspected here performs explicit catalog-model downloads with HTTP GET; no audio/model upload was found in the dictation path. No packet capture or transcription timing experiment was performed.

The inspected implementation forced CPU inference to two threads with GPU acceleration disabled and created a new `WhisperState` inside each transcription call. These are observed implementation costs, not a measured breakdown of the user's delay. The installed whisper.cpp source defaults to at most four available CPU threads; this change uses that same cap, with the actual thread count bounded by Rust's available-parallelism result. The upstream [benchmark example](https://github.com/ggml-org/whisper.cpp/blob/master/examples/bench/README.md) also demonstrates a four-thread CPU run; it is not a benchmark result for this machine or application.

**Unload speech model after** now has a required Rust-owned policy with **Never unload** as its default and first real option. The user explicitly requested removal of **Not selected** for this control; other unset controls are unchanged. Settings schema 3 migrates schemas 1 and 2 once, mapping a missing/null retention policy to Never unload while preserving saved Never/timeout choices and other valid settings. The frontend displays that actual backend policy without substituting a missing value. Existing invalid settings remain protected by the store's refusal to overwrite them.

The selected model now prepares an empty inference state while loading. After an utterance, its used state is dropped rather than retaining audio/transcript buffers. Once delivery has completed, an empty state for the next dictation is prepared before configured wake capture reopens; immediate-unload mode skips this preparation. This moves allocation off the ordinary finish-to-typing path while keeping only model weights and empty work buffers between sessions. Never unload prepares the selected model at startup even when listening is paused. GPU acceleration remains disabled; no new backend or dependency was added.

The user reserved functional testing. No tests, benchmarks, dictation sessions, microphone tests or game-input injections were run. Actual latency improvement, repeated-dictation stability, timeout expiry and save/restart interaction remain **unverified at runtime** pending the user's testing. Source and final diffs were inspected.

| Requested action | Observed result |
| --- | --- |
| `node scripts/package.mjs --ci --no-bundle -- --locked` | Passed TypeScript, Vite production assets and optimized Windows native compilation; the native release build completed in 3 minutes 14 seconds. |
| Application replacement | The preceding process (PID 63220) exited through its Quit action. The rebuilt release executable was launched without Settings/debugging arguments; process inspection found one app process at `target/release/just-say-it.exe` (PID 40164). |
| Saved configuration | Startup migrated the real settings file from schema 2 to schema 3. Both the pre-launch backup and current file contained Never unload, automatic language detection, listening unpaused and the screen indicator enabled. The pre-launch file was backed up under ignored `artifacts/model-retention-fix`; no preferences were manually edited for this change. |

## CPU/GPU transcription selection — 19 October 2026

Added an optional saved `speech_device` preference, CPU/GPU choices in Audio settings, backend availability reporting, and device-aware model loading. Missing preferences remain unset in the UI and use the shared CPU runtime policy. Schema 4 migrates schemas 1–3 once without automatically selecting GPU or changing other valid preferences. Existing invalid settings remain protected.

The source enables Vulkan through the pinned whisper-rs 0.16.0 dependency for Windows/Linux and Metal for macOS. Detection enumerates GGML's GPU/integrated-GPU devices and attempts backend initialization without audio or inference. A small C++ boundary contains native driver exceptions during this probe. GPU is disabled in the UI when no backend initializes, and backend availability is checked again when saving/using GPU. Device changes prepare the model before committing settings; failed changes preserve the preceding configuration/model. Loaded-model status explicitly labels GPU as an offload request, not proof of measured GPU use or that every operation runs on it.

Windows build wrappers prepare the pinned LunarG Vulkan Runtime 1.4.321.0 archive; native builds and installer packaging stage its architecture-matched loader and accompanying license. This avoids making app startup depend on a system-installed Vulkan loader. The CI workflow now provisions a checksum-pinned Vulkan SDK for compilation. URLs and hashes were read from LunarG's official download API; the runtime archive was downloaded and its entry names/license inspected to implement extraction. Dependency resolution was updated offline with `cargo update --offline -p whisper-rs --precise 0.16.0`; this was lockfile maintenance, not compilation or functional validation.

API behavior was inspected in the installed whisper-rs 0.16.0 bindings and its whisper.cpp 1.8.3 sources, alongside the upstream [acceleration documentation](https://github.com/ggml-org/whisper.cpp/blob/v1.8.3/README.md#vulkan-gpu-support) and LunarG's [download/hash API](https://vulkan.lunarg.com/content/view/latest-sdk-version-api). Windows runtime archive: SHA-256 `f8ba79303face4b72a2f302f1a46e8c918995b54fbf24c0c9fc6c3d3ef525dca`; SDK installer: SHA-256 `baaa4f7ca11ed3d82aa1c102b21208915485bbaa473068c763daa425cca468bd`.

Only source/diff inspection was performed for the implementation. No tests, builds, lint, typechecks, file-size checks, benchmarks, GPU initialization probes, app launches/restarts, microphone capture or input injection were run. The currently running executable and real user preferences were not replaced or edited. Actual CPU/GPU transcription, device switching, unavailable-device handling, settings migration, loader packaging and cross-platform behavior remain **unverified at runtime**.

### Requested build and application replacement

The user subsequently requested building/restarting the application so the new option would appear. The existing release process (PID 40164) exited through its own **Quit application** control. Its saved preferences were backed up under ignored `artifacts/gpu-device-build` before replacement; no device choice or other preference was manually changed.

The checksum-pinned Vulkan SDK was copied into the project's ignored build-tools directory using LunarG's documented [`copy_only=1` mode](https://vulkan.lunarg.com/doc/view/latest/windows/getting_started.html). The initial default installer mode required administrator access and rolled back; copy-only setup succeeded without elevation. Setup now caches the verified download, suppresses download progress rendering and retains installer logs. The first native build failed at MSBuild's file-tracker path-length limit in the nested shader compiler. The build environment now discovers Visual Studio's bundled [Ninja generator](https://cmake.org/cmake/help/v4.1/generator/Ninja.html) when the caller has not selected a generator. Only the failed generated Whisper CMake build directory was cleared before retrying; compiler/binding checks were kept enabled.

| Requested outcome | Observed evidence |
| --- | --- |
| Updated Windows release build | `node scripts/package.mjs --ci --no-bundle -- --locked` passed strict TypeScript, Vite assets and optimized Windows x64 native compilation with Vulkan enabled. The successful native release build completed in 4 minutes 13 seconds. |
| Restart with the new option visible | Launched `target/release/just-say-it.exe --settings` (PID 56172). Windows accessibility inspection found the **Transcription device** ComboBox in Audio settings, with `IsOffscreen = false` and displayed value **Not selected**. |
| GPU availability | The new running app reported **GPU backend initialized: NVIDIA GeForce RTX 3090.** This is evidence of backend initialization, not a transcription benchmark. |
| Settings preservation | The app migrated schema 3 to schema 4. Read-back comparison found every preceding saved preference unchanged; the new `speech_device` remained null. |
| Windows runtime loader staging | The release directory contains the 1,615,800-byte architecture-matched `vulkan-1.dll` and accompanying `VulkanRT-License.txt`; the new app started successfully with these adjacent files. No installer bundle was produced in this run. |

No dictation session, CPU/GPU mode change, microphone test, game interaction, keyboard injection, unit suite or benchmark was performed. GPU transcription performance, mode-switch failure handling, no-driver startup, installer distribution and macOS/Linux behavior remain **unverified**. The requested running-app control and detected GPU availability were observed directly.
