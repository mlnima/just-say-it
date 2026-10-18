# Compatibility status

18 October 2026. These are evidence boundaries, not a universal typing claim.

| Target | Current result |
| --- | --- |
| Windows 11 Pro 10.0.26300, x86-64, interactive desktop | Native runtime, CPAL microphone capture, real local Whisper and keyword models, native UI Automation input adapter, and Unicode SendInput implementation exist. See validation for actual checks. |
| Windows native editor | Dedicated WinForms inputs received Unicode text; changed fields, password inputs and read-only inputs were exercised. Desktop tests require an unlocked, otherwise idle desktop and verify exact fixture process identity before dispatch. |
| Microsoft Edge | Dedicated browser inputs and contenteditable received Unicode; field changes refused a delayed insertion. No claim extends to every browser/editor. |
| Full-screen games / anti-cheat inputs | Unverified. An inaccessible field does not receive blind automatic insertion; recovery remains available. |
| Elevated / secure desktops / password fields | Known password fields are refused. No routine administrator execution or security bypass is used. Secure-field detection is not universal. Elevated application behavior still needs its dedicated test matrix. |
| macOS Apple Silicon | Native insertion, input detection, permissions, startup, packaging and runtime verification remain unfinished. |
| macOS Intel | Native insertion, input detection, permissions, startup, packaging and runtime verification remain unfinished. |
| Ubuntu X11 x86-64 | Native insertion/input-detection adapter and installed desktop verification remain unfinished. |
| Ubuntu Wayland x86-64 | Compositor/portal input and target-inspection approach remains unresolved; no X11-to-Wayland compatibility claim is made. |

The non-Windows adapter returns explicit unavailable results. It does not simulate success. The Tauri/audio/model code is separated from that adapter, but cross-platform compilation alone would not establish functioning desktop dictation.

The keyword model supports English phrases. Dictation language choices come from the actual Whisper library. The first wake interaction requires a brief pause; immediate continuous speech after the command is unsupported in this initialization.

Windows package installation, bundled settings, close-to-tray and Quit were checked. The owned preceding test installation was uninstalled successfully, followed by a fresh final installation. Settings-preserving upgrades and startup cleanup with an enabled registration remain unverified.

Multi-monitor scaling, IME composition, non-US physical layouts, selection replacement, permission revocation, lock/suspend, startup after actual login, device replacement, and all failure combinations still need their full acceptance matrix. The original project acceptance checklist remains unchecked until that evidence exists.
