Windows x64 preview. Download the `-setup.exe` asset, run the installer, then open Settings from the tray. Models are optional separate downloads inside Settings.

The installer includes the native inference libraries. End users do not need Node.js, Python, Rust, CMake, or this source checkout. Windows may need an internet connection to install the WebView2 runtime when it is absent.

This build is unsigned. Windows may show a publisher or SmartScreen warning. Signing and the complete distribution-license review remain outstanding; see the attached third-party notices and the repository's validation record.

macOS `.dmg` and Ubuntu `.deb` downloads are not available: their native input adapters and desktop validation remain unfinished. This preview also has the limitations listed in the README, including unfinished hold-to-talk and broader live wake-to-dictation testing.

`SHA256SUMS` contains the installer's SHA-256 checksum. It detects a changed download; it is not a publisher signature.
