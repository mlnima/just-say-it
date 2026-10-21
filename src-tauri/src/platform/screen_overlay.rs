use super::{screen_indicator::Indicator, screen_surface::ScreenSurface};
use crate::runtime::Runtime;
use std::{
    cell::RefCell,
    sync::atomic::{AtomicBool, AtomicIsize, Ordering},
};
use windows::{
    core::w,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{Accessibility::*, HiDpi::*, WindowsAndMessaging::*},
    },
};

static WINDOW: AtomicIsize = AtomicIsize::new(0);
static REFRESH_PENDING: AtomicBool = AtomicBool::new(false);
static RESTACK_PENDING: AtomicBool = AtomicBool::new(false);
thread_local! { static INDICATOR: RefCell<Option<Indicator>> = const { RefCell::new(None) }; }

#[derive(Clone, Default)]
pub struct ScreenOverlay;

impl ScreenOverlay {
    pub fn refresh(&self) {
        let window = WINDOW.load(Ordering::Acquire);
        if window != 0 && !REFRESH_PENDING.swap(true, Ordering::AcqRel) {
            // SAFETY: this message carries no pointers and is handled on the window's thread.
            let posted = unsafe {
                PostMessageW(Some(HWND(window as _)), WM_APP + 1, WPARAM(0), LPARAM(0))
            };
            if posted.is_err() {
                REFRESH_PENDING.store(false, Ordering::Release);
            }
        }
    }

    pub fn stop(&self) {
        let window = WINDOW.load(Ordering::Acquire);
        if window != 0 {
            // SAFETY: the owning thread destroys all five windows after receiving WM_CLOSE.
            let _ = unsafe {
                PostMessageW(Some(HWND(window as _)), WM_CLOSE, WPARAM(0), LPARAM(0))
            };
        }
    }
}

pub fn start(runtime: Runtime) {
    let owner = runtime.clone();
    let spawned = std::thread::Builder::new()
        .name("screen-overlay".into())
        .spawn(move || {
            if let Err(error) = run(owner.clone()) {
                owner.update(|s| {
                    s.screen_overlay_available = false;
                    s.screen_overlay_error = Some(error);
                });
            }
        });
    if spawned.is_err() {
        runtime.update(|s| {
            s.screen_overlay_error = Some("Cannot start the screen overlay thread.".into());
        });
    }
}

fn run(runtime: Runtime) -> Result<(), String> {
    // SAFETY: all HWNDs, hooks, GDI resources and thread-local state stay on this thread.
    unsafe {
        if SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).0.is_null() {
            return Err("Cannot configure screen overlay scaling.".into());
        }
        let module = GetModuleHandleW(None).map_err(|_| "Cannot load the overlay module.")?;
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: module.into(),
            lpszClassName: w!("DictationScreenActivity"),
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            return Err("Cannot register the screen overlay.".into());
        }
        let result = (|| {
            let surface = ScreenSurface::new(&class)?;
            let hook = SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_MOVESIZEEND,
                None,
                Some(foreground_event),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            );
            if hook.is_invalid() {
                return Err("Cannot track the active screen for the overlay.".into());
            }
            WINDOW.store(surface.control_window().0 as isize, Ordering::Release);
            INDICATOR.with(|state| {
                *state.borrow_mut() = Some(Indicator::new(runtime.clone(), surface));
            });
            runtime.update(|s| {
                s.screen_overlay_available = true;
                s.screen_overlay_error = None;
            });
            if runtime.stopping.load(Ordering::Acquire) {
                runtime.screen_overlay.stop();
            }
            let mut message = MSG::default();
            let result = loop {
                let received = GetMessageW(&mut message, None, 0, 0).0;
                if received <= 0 {
                    break received;
                }
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            };
            let _ = UnhookWinEvent(hook);
            WINDOW.store(0, Ordering::Release);
            REFRESH_PENDING.store(false, Ordering::Release);
            RESTACK_PENDING.store(false, Ordering::Release);
            let indicator = INDICATOR.with(|state| state.borrow_mut().take());
            drop(indicator);
            (result == 0)
                .then_some(())
                .ok_or("The screen overlay message loop failed.".into())
        })();
        let _ = UnregisterClassW(class.lpszClassName, Some(module.into()));
        result
    }
}

unsafe extern "system" fn foreground_event(
    _: HWINEVENTHOOK,
    _: u32,
    _: HWND,
    _: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    RESTACK_PENDING.store(true, Ordering::Release);
    ScreenOverlay.refresh();
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATEANDEAT as isize),
        WM_CLOSE => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_DISPLAYCHANGE | WM_SETTINGCHANGE | WM_DPICHANGED => {
            RESTACK_PENDING.store(true, Ordering::Release);
            ScreenOverlay.refresh();
            LRESULT(0)
        }
        value if value == WM_APP + 1 => {
            REFRESH_PENDING.store(false, Ordering::Release);
            // Positioning sends messages synchronously; never hold a RefCell borrow across it.
            if let Some(mut indicator) = INDICATOR.with(|state| state.borrow_mut().take()) {
                if let Err(error) = indicator.refresh(RESTACK_PENDING.swap(false, Ordering::AcqRel)) {
                    indicator.fail(error);
                }
                INDICATOR.with(|state| *state.borrow_mut() = Some(indicator));
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(window, message, wparam, lparam),
    }
}

