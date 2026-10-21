use dictation_core::{
    defaults,
    session::{safe_inline_text, Delivery},
};
use std::{
    sync::{
        atomic::{AtomicIsize, AtomicU64, AtomicU8, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use windows::Win32::{
    Foundation::{HWND, RECT},
    System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_MULTITHREADED,
    },
    UI::{Accessibility::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

#[derive(Clone, Default)]
pub struct OverlayHandle {
    pub window: Arc<AtomicIsize>,
    pub mode: Arc<AtomicU8>,
}

impl OverlayHandle {
    pub fn set_mode(&self, mode: u8) {
        self.mode.store(mode, Ordering::Release);
        let window = self.window.load(Ordering::Acquire);
        if window != 0 {
            // SAFETY: the posted message owns no pointers; a destroyed HWND simply rejects it.
            let _ = unsafe {
                PostMessageW(
                    Some(HWND(window as _)),
                    WM_APP + 1,
                    Default::default(),
                    Default::default(),
                )
            };
        }
    }
    pub fn stop(&self) {
        let window = self.window.load(Ordering::Acquire);
        if window != 0 {
            let _ = unsafe {
                PostMessageW(
                    Some(HWND(window as _)),
                    WM_CLOSE,
                    Default::default(),
                    Default::default(),
                )
            };
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Target {
    pub process_id: u32,
    pub window: isize,
    pub runtime_id: Vec<i32>,
    pub bounds: RECT,
    pub editable: bool,
    pub secure: bool,
    pub own_process: bool,
    pub(super) known_password: bool,
    pub(super) keyboard_window: Option<super::ShortcutTarget>,
}

impl Target {
    pub fn known(&self) -> bool {
        self.window != 0
            && self.process_id != 0
            && self.editable
            && !self.secure
            && !self.own_process
            && !self.runtime_id.is_empty()
    }
    pub fn same(&self, other: &Self) -> bool {
        let same_field = self.known()
            && other.known()
            && self.window == other.window
            && self.process_id == other.process_id
            && self.runtime_id == other.runtime_id;
        self.keyboard_window.as_ref().map_or(same_field, |window| {
            window.window == other.window
                && window.is_current()
                && !other.known_password
                && !other.own_process
                && (same_field || (!self.known() && !other.known()))
        })
    }

    pub fn window_only(&self) -> bool {
        self.keyboard_window.is_some() && !self.known()
    }
}

pub struct Focus {
    automation: std::mem::ManuallyDrop<IUIAutomation>,
    focus_events: Option<IUIAutomationFocusChangedEventHandler>,
}

impl Focus {
    pub fn new() -> Result<Self, String> {
        // This adapter is created, used and dropped on one dedicated MTA worker.
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .map_err(|_| "Windows accessibility initialization failed.")?;
            let initialized = (|| -> windows::core::Result<Self> {
                let automation: IUIAutomation2 =
                    CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)?;
                automation
                    .SetConnectionTimeout(dictation_core::defaults::ACCESSIBILITY_TIMEOUT_MILLIS)?;
                automation.SetTransactionTimeout(
                    dictation_core::defaults::ACCESSIBILITY_TIMEOUT_MILLIS,
                )?;
                Ok(Self {
                    automation: std::mem::ManuallyDrop::new(automation.into()),
                    focus_events: None,
                })
            })();
            initialized.map_err(|_| {
                CoUninitialize();
                "Windows input inspection is unavailable.".into()
            })
        }
    }

    pub fn inspect(&self) -> Target {
        unsafe { super::inspection::inspect(&self.automation) }
    }

    pub fn shortcut_target(&self, window: Option<super::ShortcutTarget>) -> Target {
        let Some(window) = window.filter(|window| window.is_current()) else {
            return Target::default();
        };
        let mut target = self.inspect();
        if window.window != target.window || !window.is_current()
            || target.known_password || target.own_process
        {
            return Target::default();
        }
        target.keyboard_window = Some(window);
        target
    }

    pub(super) fn document_id(&self, target: &Target) -> Option<Vec<i32>> {
        unsafe { super::inspection::document_id(&self.automation, target) }
    }

    pub fn watch_changes(mut self) -> Result<Self, String> {
        // Register and remove on the dictation MTA worker, which owns no windows.
        let handler: IUIAutomationFocusChangedEventHandler =
            super::focus_events::FocusEvents.into();
        unsafe { self.automation.AddFocusChangedEventHandler(None, &handler) }
            .map_err(|_| "Windows input focus notifications are unavailable.")?;
        self.focus_events = Some(handler);
        Ok(self)
    }

    pub fn insert(
        &self,
        target: &Target,
        text: &str,
        authorization: &AtomicU64,
        id: u64,
    ) -> Delivery {
        if !safe_inline_text(text)
            || !target.same(&self.inspect())
            || authorization.load(Ordering::Acquire) != id
        {
            return Delivery::Rejected;
        }
        // A finish shortcut may still be held after a short transcription.
        // Wait on the worker before sending anything, checking cancellation and
        // the original field throughout. The input operation is attempted once.
        let deadline = Instant::now()
            + Duration::from_millis(defaults::INSERTION_KEY_RELEASE_MILLIS);
        while [VK_CONTROL, VK_SHIFT, VK_MENU, VK_LWIN, VK_RWIN]
            .iter()
            .any(|key| unsafe { GetAsyncKeyState(key.0 as i32) < 0 })
        {
            if Instant::now() >= deadline
                || authorization.load(Ordering::Acquire) != id
                || !target.same(&self.inspect())
            {
                return Delivery::Rejected;
            }
            std::thread::sleep(Duration::from_millis(defaults::INSERTION_KEY_POLL_MILLIS));
        }
        let layout = target.keyboard_window.as_ref().map(|window| window.layout());
        let events = super::keyboard_events::events(text, layout);
        if !target.same(&self.inspect()) || authorization.load(Ordering::Acquire) != id
            || target.keyboard_window.as_ref().map(|window| window.layout()) != layout
        {
            return Delivery::Rejected;
        }
        // SAFETY: a bounded, fully initialized input slice remains alive for this synchronous call.
        let sent = unsafe { SendInput(&events, std::mem::size_of::<INPUT>() as i32) } as usize;
        if sent > 0 && sent < events.len() {
            super::keyboard_events::release_pressed(&events[..sent]);
        }
        match sent {
            0 => Delivery::Rejected,
            count if count != events.len() => Delivery::Partial,
            _ if !target.same(&self.inspect()) => Delivery::Unknown,
            _ => Delivery::Submitted,
        }
    }
}

impl Drop for Focus {
    fn drop(&mut self) {
        unsafe {
            if let Some(handler) = self.focus_events.take() {
                let _ = self.automation.RemoveFocusChangedEventHandler(&handler);
            }
            std::mem::ManuallyDrop::drop(&mut self.automation);
            CoUninitialize();
        }
    }
}

pub fn start(runtime: crate::runtime::Runtime) {
    std::thread::spawn(move || {
        if let Err(error) = super::watcher::run(runtime.clone()) {
            runtime.update(|s| {
                s.input_detection_supported = false;
                s.message = error;
            });
        }
    });
}
