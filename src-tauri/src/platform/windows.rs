use dictation_core::session::{safe_inline_text, Delivery};
use std::sync::{
    atomic::{AtomicIsize, AtomicU64, AtomicU8, Ordering},
    Arc,
};
use windows::core::Interface;
use windows::Win32::{
    Foundation::{HWND, RECT},
    System::{
        Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_MULTITHREADED, SAFEARRAY,
        },
        Ole::*,
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
        self.known()
            && other.known()
            && self.window == other.window
            && self.process_id == other.process_id
            && self.runtime_id == other.runtime_id
    }
}

pub struct Focus {
    automation: std::mem::ManuallyDrop<IUIAutomation>,
}

impl Focus {
    pub fn new() -> Result<Self, String> {
        // This adapter is created, used and dropped on one dedicated MTA worker.
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .map_err(|_| "Windows accessibility initialization failed.")?;
            match CoCreateInstance::<_, IUIAutomation>(&CUIAutomation, None, CLSCTX_INPROC_SERVER) {
                Ok(automation) => {
                    if let Ok(options) = automation.cast::<IUIAutomation2>() {
                        let _ = options.SetConnectionTimeout(
                            dictation_core::defaults::ACCESSIBILITY_TIMEOUT_MILLIS,
                        );
                        let _ = options.SetTransactionTimeout(
                            dictation_core::defaults::ACCESSIBILITY_TIMEOUT_MILLIS,
                        );
                    }
                    Ok(Self {
                        automation: std::mem::ManuallyDrop::new(automation),
                    })
                }
                Err(_) => {
                    CoUninitialize();
                    Err("Windows input inspection is unavailable.".into())
                }
            }
        }
    }

    pub fn inspect(&self) -> Target {
        unsafe {
            let foreground = GetForegroundWindow();
            let window = foreground.0 as isize;
            let mut window_process = 0;
            GetWindowThreadProcessId(foreground, Some(&mut window_process));
            if window_process == std::process::id() {
                return Target {
                    window,
                    process_id: window_process,
                    own_process: true,
                    ..Default::default()
                };
            }
            let Ok(element) = self.automation.GetFocusedElement() else {
                return Target {
                    window,
                    ..Default::default()
                };
            };
            let process_id = element.CurrentProcessId().unwrap_or_default() as u32;
            let own_process = process_id == std::process::id();
            // An unknown password flag is treated as unsafe, never as permission to type.
            let secure = element
                .CurrentIsPassword()
                .map(|b| b.as_bool())
                .unwrap_or(true);
            let focused = element.CurrentHasKeyboardFocus().is_ok_and(|b| b.as_bool());
            let control = element.CurrentControlType().unwrap_or_default();
            let value: Result<IUIAutomationValuePattern, _> =
                element.GetCurrentPatternAs(UIA_ValuePatternId);
            let text: Result<IUIAutomationTextPattern, _> =
                element.GetCurrentPatternAs(UIA_TextPatternId);
            let editable = focused
                && match value {
                    Ok(pattern) => pattern.CurrentIsReadOnly().is_ok_and(|b| !b.as_bool()),
                    Err(_) => control == UIA_EditControlTypeId && text.is_ok(),
                };
            let runtime_id = element
                .GetRuntimeId()
                .ok()
                .and_then(|array| read_runtime_id(array).ok())
                .unwrap_or_default();
            Target {
                process_id,
                window,
                runtime_id,
                bounds: element.CurrentBoundingRectangle().unwrap_or_default(),
                editable,
                secure,
                own_process,
            }
        }
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
        // Modifier keys must be released; delivery is never blindly retried.
        let held = [VK_CONTROL, VK_SHIFT, VK_MENU, VK_LWIN, VK_RWIN]
            .iter()
            .any(|key| unsafe { GetAsyncKeyState(key.0 as i32) < 0 });
        if held {
            return Delivery::Rejected;
        }
        let events: Vec<INPUT> = text
            .encode_utf16()
            .flat_map(|unit| {
                [false, true].map(|up| INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(0),
                            wScan: unit,
                            dwFlags: KEYEVENTF_UNICODE
                                | if up {
                                    KEYEVENTF_KEYUP
                                } else {
                                    KEYBD_EVENT_FLAGS(0)
                                },
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                })
            })
            .collect();
        if !target.same(&self.inspect()) || authorization.load(Ordering::Acquire) != id {
            return Delivery::Rejected;
        }
        // SAFETY: a bounded, fully initialized input slice remains alive for this synchronous call.
        let sent = unsafe { SendInput(&events, std::mem::size_of::<INPUT>() as i32) } as usize;
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
            std::mem::ManuallyDrop::drop(&mut self.automation);
            CoUninitialize();
        }
    }
}

unsafe fn read_runtime_id(array: *mut SAFEARRAY) -> windows::core::Result<Vec<i32>> {
    // UIA owns a one-dimensional VT_I4 array returned to the caller. Always release it.
    let result = (|| {
        let lower = SafeArrayGetLBound(array, 1)?;
        let upper = SafeArrayGetUBound(array, 1)?;
        let length = (upper - lower + 1).max(0) as usize;
        if length > 128 {
            return Err(windows::core::Error::from_win32());
        }
        let mut pointer = std::ptr::null_mut();
        SafeArrayAccessData(array, &mut pointer)?;
        let values = if length == 0 {
            Vec::new()
        } else {
            std::slice::from_raw_parts(pointer as *const i32, length).to_vec()
        };
        SafeArrayUnaccessData(array)?;
        Ok(values)
    })();
    let _ = SafeArrayDestroy(array);
    result
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
