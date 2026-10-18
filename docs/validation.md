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
