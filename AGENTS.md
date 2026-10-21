# Just Say It: agent instructions

## Scope and current status

These instructions apply throughout this repository. Follow applicable instruction priority and explicit task instructions.

The repository now contains an initial Windows implementation: a Rust workspace/core, a Tauri 2 desktop runtime, a React/TypeScript settings UI, local Whisper and keyword-detector adapters, native Windows input detection/insertion, tray controls, and model/settings management. It is not the completed cross-platform release. macOS/Ubuntu native input adapters and the full release acceptance matrix remain unfinished. Read [docs/validation.md](docs/validation.md) before describing any outcome as verified.

Read the initialization document before implementing the application. It owns product behavior, proposed architecture, milestones, and acceptance criteria. This file owns the everyday development rules. Update the current-status and command information as real implementation becomes available; do not invent commands or test results.

## Product direction

The planned application helps people dictate into the active text input on Windows, Ubuntu, and macOS. It uses Rust, Tauri 2, and React with TypeScript. It runs in the user's desktop session, has a tray menu and optional launch at login, and supports user-defined shortcuts and wake phrases.

The first speech-to-text provider is local OpenAI Whisper inference. Later providers and optional text cleanup must fit behind small, explicit interfaces. Read the initialization document for what belongs in the first release and what is future work. Roadmap entries alone do not authorize implementation.

## Scope discipline

- Complete the requested task and inspect the changes. Run tests or validation commands only when the user explicitly requests them, as detailed below. Do not add unrelated features, dependencies, formatting, cleanup, or architecture changes.
- Inspect existing code, configuration, interfaces, and local instructions before editing. Reuse established patterns.
- Preserve existing user changes. Never overwrite, delete, revert, or recreate unrelated work.
- For investigation, planning, review, or explanation, do not change files unless implementation or documentation edits are requested.
- For multi-step work, write a short checklist before editing. Keep implementation steps separate from acceptance criteria. Derive acceptance criteria from the requested result, not from what is easiest to build.
- Resolve routine choices from evidence. Ask a focused question only when an unresolved choice changes behavior or scope. Do not repeatedly request permission for already authorized work.
- Check official documentation for unfamiliar or version-dependent details, matching the installed version. Cite the documentation actually consulted.
- Continue through failures caused by your changes and fix them. Rerun affected checks only within the user's explicit testing request. Report genuine blockers with the unfinished requirement and the specific missing input or access.
- Do not create commits, publish releases, or choose a project license on the owner's behalf unless asked.

## File size and organization

- Every code file created or modified must contain fewer than 300 total lines, including comments, blank lines, and tests.
- Split an oversized file involved in the task into smaller, cohesive files. Do not compress code or remove useful explanations to meet the limit.
- Markdown and data/configuration files such as JSON are exempt. Executable configuration scripts remain code.
- Keep each module focused on one responsibility. Prefer explicit calls and small interfaces over generic frameworks, service locators, or a large central manager.
- Do not create empty scaffolding for possible future features. Add modules when the current implementation needs them.
- Keep OS-specific code behind platform adapters and provider-specific code behind provider adapters.
- Do not retain duplicate implementations or compatibility branches for replaced internal code. Versioned user-setting migrations are allowed to preserve real user data; run migrations once instead of maintaining two runtime paths.
- Keep generated output reproducible. Do not hand-edit generated code; configure or split generation if needed to meet the code-file limit. Third-party dependencies belong outside authored source, not copied into oversized repository files.

## Rust responsibilities

- Rust owns audio, activation, session state, inference, settings validation, credentials, OS permissions, and text insertion.
- Keep Tauri commands thin. The domain core must be testable without launching a webview or reaching an operating system API.
- Use typed states, errors, and results. Distinguish unavailable, permission denied, cancelled, failed, submitted, confirmed, partially delivered, and delivery unknown.
- Use bounded queues and explicit cancellation. Never block the audio callback or UI thread with inference, downloads, disk access, or network requests.
- Keep real-time audio callbacks minimal. Avoid allocation, blocking locks, logging, and other unpredictable work in them.
- Do not use unchecked panics for recoverable input, device, permission, model, or provider failures.
- Isolate any necessary unsafe code or native interface code in small adapters and document its invariants. Include boundary checks when the user explicitly requests testing of that area.
- A cancelled or outdated session must never insert late text. Only one coordinator may authorize an insertion.

## React and TypeScript rules

- Use strict TypeScript and arrow functions for application functions and components. Rust follows normal Rust syntax.
- Use optional chaining for possibly missing children. Use `||` when all falsy values mean absent and `??` when `false`, `0`, or an empty string is a valid value.
- Prefer a short expression for a simple choice. Use readable `if` blocks for multi-step behavior; avoid nested ternaries and clever compression.
- Use template literals for multiline Markdown. Never construct Markdown using arrays joined with newline strings.
- Import and export from the owning file. Do not create `index.ts` barrel files.
- Keep components small and related UI logic together. Do not duplicate domain state or validation rules in React.
- Use a typed command/event bridge. Clean up subscriptions on unmount, handle reconnects, and ignore outdated session events.
- Add dependencies only when required for the task. Use one JavaScript package manager and its committed lockfile once selected.

## Naming and configuration

- Name things by their purpose. Avoid promotional words from prompts in identifiers or comments.
- Keep the product name and application identity in one configuration source. Do not scatter names, service identifiers, URLs, paths, timings, model names, or thresholds throughout the code.
- Stable protocol fields and enum variants are allowed; “no hardcoded names” does not require making every identifier configurable.
- Put configurable runtime defaults in one Rust-owned defaults module and expose them through the typed bridge. Do not duplicate them in React.
- Keep settings schema, saved preferences, and observed runtime status distinct. Saving a preference does not prove it was applied successfully.
- Use atomic settings writes and an explicit schema version. Preserve unreadable or invalid settings for recovery; do not silently overwrite them.
- Never store credentials in normal settings, logs, frontend storage, fixtures, or source control.

## Honest settings and UI

- Show saved values, unset values, and observed runtime status accurately. Never show a feature active because a value was filled in as a UI fallback.
- Settings must not substitute fallback values for missing settings. Use placeholders or a separate explanation of a recommended value.
- Every dropdown starts with an empty “Not selected” option. Select a real option only when an actual saved setting exists.
- Runtime fallback behavior outside settings is allowed only from the shared defaults module. Show its use explicitly when it matters to the user.
- First-run choices must be saved deliberately. Do not prefill fake microphones, models, shortcuts, provider accounts, endpoints, or wake phrases.
- Distinguish “enabled in settings,” “permission granted,” and “running.” Show why an enabled feature cannot run.
- Keep the interface accessible by keyboard and screen reader. Do not communicate microphone state through color alone.
- Never steal focus from the user's target input to show recording status.

## Platform, privacy, and delivery rules

- Windows, Ubuntu, and macOS are required targets. Ubuntu Wayland and X11 are distinct compatibility cases, not interchangeable names for Linux support.
- Do not claim universal typing from one successful editor test. Permissions, secure inputs, elevated apps, games, and desktop environments can restrict automation.
- Do not bypass permissions, anti-cheat systems, secure desktops, or OS security controls. Do not require routine administrator/root execution.
- Implement background operation in the signed-in user's session. Do not place microphone access or desktop typing in a machine-level service.
- Wake listening, launch at login, and cloud processing require explicit user choices. Pausing all listening must actually release microphone capture and remain in effect across restarts until explicitly resumed.
- Do not retain audio or transcript history by default. Keep only the bounded, temporary state needed for the active session or the latest delivery's recovery.
- Do not log raw audio, dictated text, clipboard content, window titles, or secrets. Diagnostics should use timings, capability status, and error codes.
- Do not switch from local processing to a cloud provider silently.
- Treat transcripts, model output, and remote provider responses as untrusted text. They must never execute commands or acquire OS privileges.
- Recheck the insertion target as far as the OS permits. Do not silently redirect delayed text to another app or field.
- Do not blindly retry text insertion or claim exactly-once delivery into arbitrary applications. Preserve recoverable text when delivery is partial or uncertain.
- Clipboard use must follow the initialization document's explicit consent and restoration rules. Never overwrite a newer clipboard value during cleanup.
- Do not simulate Enter, submit forms, send messages, or execute terminal commands as part of dictation.

## Verification and completion

- Before finishing, compare the actual changes against every requested outcome and inspect for omissions and unintended modifications.
- Run tests and validation commands only when the user explicitly asks. An implementation request alone does not authorize tests, builds, linting, typechecking, file-size checks, benchmarks, or other validation runs.
- When testing is requested, test only the changed behavior and the directly affected paths needed to check it. Do not run unrelated checks or full suites unless the user explicitly requests that scope.
- Do not automatically repeat checks after every edit. Any necessary rerun must remain within the user's requested scope. Stop testing immediately when the user asks.
- Do not launch or restart the application, open test windows or browser fixtures, inject mouse/keyboard input, or access the microphone for validation unless the user explicitly requests interactive desktop testing. Do not interrupt the user's normal desktop use.
- Without a testing request, finish with code and diff inspection. State that tests were not run; do not ask for testing permission after routine edits or claim runtime behavior was verified.
- Formatting, linting, typechecking, compilation, and mocked tests alone do not prove microphone capture, wake detection, global shortcuts, or cross-application insertion work.
- When the user requests testing, use unit tests for changed domain rules, integration tests for changed providers/adapters, and explicitly requested desktop tests for changed OS behavior. Include relevant failures and cancellation within that scope.
- Never weaken tests, suppress failures, or replace production behavior with mocks to claim success.
- Cross-platform build results do not prove runtime compatibility. Record the OS, architecture, desktop/session type, permissions, target app, and actual observed outcome.
- Mark unavailable platform verification as unverified. Do not replace a required platform with a different one or silently reduce acceptance criteria.
- Stop expanding verification once the relevant checks pass and no concrete correctness concern remains.
- For small tasks, report the result and verification concisely. For large tasks, report **Checklist result** (`done`, `blocked`, or `unverified` for each requested outcome), **Validation results**, and **Remaining issues** only when needed.
- Claim completion only when all requested outcomes have evidence. Keep the verified commands and validation record below accurate as implementation changes.

## Current layout and checked commands

- `crates/dictation-core`: provider-independent settings/session/authorization rules and tests.
- `src-tauri/src`: native adapters, runtime composition, bounded audio worker, commands, and tray.
- `src`: typed command/event bridge and settings/status/recovery UI.
- `assets/icon.svg`: authored icon source; Tauri-generated sizes live in `src-tauri/icons`.
- `docs`: decisions, compatibility, validation, and third-party attribution status.
- `tests/desktop`: dedicated external editor/browser fixtures and real-runtime UI checks.
- `.github/workflows/build.yml`: fresh Windows builds and tagged preview installer releases; macOS and Ubuntu packages are not published while their native functionality is unfinished.

Commands executed successfully during initialization (historical reference, not instructions to run them automatically):

| Command | Observed purpose |
| --- | --- |
| `npm install` | Installed the pinned frontend/CLI dependencies and generated package-lock.json. |
| `npm run dev` | Started the Vite development server on loopback port 1420. |
| `npm run desktop -- --no-watch -- -- --settings` | Built and launched the desktop runtime/settings window. Ordinary launch starts in the tray. |
| `npm run build` | Strict TypeScript check and production frontend build. |
| `npm run check:files` | Checked every authored code file against the fewer-than-300-line limit. |
| `npm run package` | Built the release frontend/native executable and a Windows x64 NSIS test installer with its matching native libraries. |
| `node scripts/cargo.mjs build -p just-say-it --no-default-features` | Built the Windows development executable with the discovered native toolchain. |
| `node scripts/cargo.mjs test --workspace -- --include-ignored --test-threads=1` | Ran eight core tests and seven actual adapter/desktop tests successfully with their documented prerequisites. |

The ignored desktop tests require real catalog models under `artifacts/models`, generated test audio, a real microphone, Microsoft Edge/Playwright, and an unlocked idle desktop. Set `PLAYWRIGHT_MODULE_PATH` when Playwright is supplied externally. They must never type into a window whose process identity is not their own dedicated fixture. Ordinary `cargo test` leaves those desktop/model tests ignored.

`scripts/build-environment.mjs` discovers MSVC/Windows SDK headers. Install development-only libclang as described in README; it is not an end-user dependency. Quit a running development executable before rebuilding its loaded native DLLs. Do not disable type/bindgen/layout checks or suppress native linker errors to obtain a successful build.

The debug-only `DICTATION_TEST_DIRECTORY` environment variable isolates test settings/models from real user preferences. Public/release builds ignore it. Keep test artifacts, native build tooling, recordings, models and browser profiles out of version control. See README and the validation record for packaging and runtime test procedures.
