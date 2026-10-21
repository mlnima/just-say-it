use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE},
};

pub(super) unsafe fn exclude_from_capture(window: HWND) -> Result<(), String> {
    // The caller owns this top-level HWND on the current thread. Never show an
    // overlay when Windows cannot apply the requested capture exclusion.
    SetWindowDisplayAffinity(window, WDA_EXCLUDEFROMCAPTURE)
        .map_err(|_| "Windows could not exclude the dictation overlay from screen capture.".into())
}
