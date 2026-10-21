use windows::Win32::UI::{
    Input::KeyboardAndMouse::{GetKeyboardLayout, HKL},
    WindowsAndMessaging::*,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShortcutTarget {
    pub(super) window: isize,
    process: u32,
    thread: u32,
    control: isize,
}

impl ShortcutTarget {
    pub fn capture() -> Option<Self> {
        // Read desktop focus only. No process handles, memory, hooks or thread
        // attachment are used; this snapshot travels with the shortcut command.
        unsafe {
            let window = GetForegroundWindow();
            let mut process = 0;
            let thread = GetWindowThreadProcessId(window, Some(&mut process));
            if window.is_invalid() || process == 0 || thread == 0
                || process == std::process::id() || !IsWindowVisible(window).as_bool()
            {
                return None;
            }
            let mut information = GUITHREADINFO {
                cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
                ..Default::default()
            };
            GetGUIThreadInfo(thread, &mut information).ok()?;
            let menus = GUI_INMENUMODE | GUI_POPUPMENUMODE | GUI_SYSTEMMENUMODE;
            if information.hwndFocus.is_invalid()
                || GetAncestor(information.hwndFocus, GA_ROOT) != window
                || information.flags.0 & menus.0 != 0
                || GetForegroundWindow() != window
            {
                return None;
            }
            Some(Self {
                window: window.0 as isize,
                process,
                thread,
                control: information.hwndFocus.0 as isize,
            })
        }
    }

    pub(super) fn is_current(&self) -> bool {
        Self::capture().as_ref() == Some(self)
    }

    pub(super) fn layout(&self) -> HKL {
        // Reading the foreground thread's layout does not attach to that thread.
        unsafe { GetKeyboardLayout(self.thread) }
    }
}
