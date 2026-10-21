use super::{
    dismissal::Dismissals,
    overlay::ButtonAction,
    windows::{Focus, Target},
};
use crate::runtime::{Action, Runtime};
use std::{
    cell::RefCell,
    sync::atomic::{AtomicBool, AtomicIsize, Ordering},
};
use windows::{
    core::w,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, RemoteDesktop::*},
        UI::{Accessibility::*, WindowsAndMessaging::*},
    },
};

struct Watcher {
    runtime: Runtime,
    focus: Focus,
    window: HWND,
    target: Target,
    dismissals: Dismissals,
}
thread_local! { static WATCHER: RefCell<Option<Watcher>> = const { RefCell::new(None) }; }
static EVENT_WINDOW: AtomicIsize = AtomicIsize::new(0);
static REFRESH_PENDING: AtomicBool = AtomicBool::new(false);

pub fn run(runtime: Runtime) -> Result<(), String> {
    let focus = Focus::new()?;
    unsafe {
        let module = GetModuleHandleW(None).map_err(|_| "Cannot load the application module.")?;
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: module.into(),
            lpszClassName: w!("DictationInputControl"),
            hCursor: LoadCursorW(None, IDC_HAND).unwrap_or_default(),
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            return Err("Input button registration failed.".into());
        }
        let window = CreateWindowExW(
            WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            class.lpszClassName,
            w!("Dictation"),
            WS_POPUP,
            0,
            0,
            dictation_core::defaults::OVERLAY_WIDTH,
            dictation_core::defaults::OVERLAY_HEIGHT,
            None,
            None,
            Some(module.into()),
            None,
        )
        .map_err(|_| "Cannot create the input button.")?;
        if let Err(error) = super::window_policy::exclude_from_capture(window) {
            let _ = DestroyWindow(window);
            return Err(error);
        }
        runtime
            .overlay
            .window
            .store(window.0 as isize, Ordering::Release);
        EVENT_WINDOW.store(window.0 as isize, Ordering::Release);
        let _ = WTSRegisterSessionNotification(window, NOTIFY_FOR_THIS_SESSION);
        WATCHER.with(|state| {
            *state.borrow_mut() = Some(Watcher {
                runtime,
                focus,
                window,
                target: Target::default(),
                dismissals: Dismissals::default(),
            })
        });
        let hooks: Vec<_> = [
            EVENT_OBJECT_FOCUS,
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_MOVESIZESTART,
            EVENT_SYSTEM_MOVESIZEEND,
        ]
        .into_iter()
        .map(|event| {
            SetWinEventHook(
                event,
                event,
                None,
                Some(focus_event),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            )
        })
        .collect();
        refresh();
        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).0 > 0 {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        for hook in hooks {
            let _ = UnhookWinEvent(hook);
        }
        EVENT_WINDOW.store(0, Ordering::Release);
        WATCHER.with(|state| {
            state.borrow_mut().take();
        });
    }
    Ok(())
}

unsafe extern "system" fn focus_event(
    _: HWINEVENTHOOK,
    _: u32,
    _: HWND,
    _: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    queue_refresh();
}

pub(super) fn queue_refresh() {
    // Never inspect inside callbacks. Coalesce UIA and WinEvent notifications into one message.
    let window = EVENT_WINDOW.load(Ordering::Acquire);
    if window != 0 && !REFRESH_PENDING.swap(true, Ordering::AcqRel) {
        let posted =
            unsafe { PostMessageW(Some(HWND(window as _)), WM_APP + 1, WPARAM(0), LPARAM(0)) };
        if posted.is_err() {
            REFRESH_PENDING.store(false, Ordering::Release);
        }
    }
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        // Consume the activation mouse-down. Mouse-up below performs the action
        // without passing a focus-changing button-down to the default handler.
        WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATEANDEAT as isize),
        WM_LBUTTONDOWN => LRESULT(0),
        WM_LBUTTONUP => {
            WATCHER.with(|state| {
                if let Some(state) = state.borrow_mut().as_mut() {
                    let mode = state.runtime.overlay.mode.load(Ordering::Acquire);
                    match super::overlay::hit_test(mode, lparam.0) {
                        Some(ButtonAction::Dismiss) => {
                            let target = state.focus.inspect();
                            if target.same(&state.target) {
                                state.dismissals.dismiss(&state.focus, &target, window);
                            }
                        }
                        Some(ButtonAction::Activate) => {
                            let _ = state.runtime.send(Action::Toggle);
                        }
                        None => {}
                    }
                }
            });
            queue_refresh();
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == super::dismissal::EXPIRY_TIMER => {
            refresh();
            LRESULT(0)
        }
        WM_PAINT => {
            WATCHER.with(|state| {
                if let Some(state) = state.borrow().as_ref() {
                    super::overlay::paint(
                        window,
                        state.runtime.overlay.mode.load(Ordering::Acquire),
                    );
                }
            });
            LRESULT(0)
        }
        WM_WTSSESSION_CHANGE
            if matches!(
                wparam.0 as u32,
                WTS_SESSION_LOCK
                    | WTS_SESSION_LOGOFF
                    | WTS_CONSOLE_DISCONNECT
                    | WTS_REMOTE_DISCONNECT
            ) =>
        {
            pause();
            LRESULT(0)
        }
        WM_POWERBROADCAST if wparam.0 == 4 => {
            pause();
            LRESULT(1)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(window);
            LRESULT(0)
        }
        WM_DESTROY => {
            let _ = WTSUnRegisterSessionNotification(window);
            PostQuitMessage(0);
            LRESULT(0)
        }
        value if value == WM_APP + 1 => {
            REFRESH_PENDING.store(false, Ordering::Release);
            refresh();
            LRESULT(0)
        }
        _ => DefWindowProcW(window, message, wparam, lparam),
    }
}

fn pause() {
    WATCHER.with(|state| {
        if let Some(state) = state.borrow().as_ref() {
            let _ = state.runtime.send(Action::Pause);
        }
    });
}

fn refresh() {
    let display = WATCHER.with(|state| {
        let mut state = state.borrow_mut();
        let state = state.as_mut()?;
        state.dismissals.expire(state.window);
        let mode = state.runtime.overlay.mode.load(Ordering::Acquire);
        if mode == 0 {
            return Some((state.window, mode, None));
        }
        let target = state.focus.inspect();
        state.target = target.clone();
        let message = if target.secure {
            "Protected or uninspectable field"
        } else if target.known() {
            "Editable input detected"
        } else {
            "No supported editable input detected"
        };
        let changed = state
            .runtime
            .snapshot()
            .is_ok_and(|s| s.target_status != message);
        if changed {
            state.runtime.update(|s| s.target_status = message.into());
        }
        let bounds = if mode == 1 && state.dismissals.hidden(&state.focus, &target) {
            None
        } else if target.known() {
            Some(target.bounds)
        } else if matches!(mode, 2 | 3) {
            let mut bounds = windows::Win32::Foundation::RECT::default();
            unsafe { GetWindowRect(GetForegroundWindow(), &mut bounds) }
                .ok()
                .map(|_| bounds)
        } else {
            None
        };
        Some((state.window, mode, bounds))
    });
    // Native positioning can send window messages synchronously; release the state borrow first.
    if let Some((window, mode, bounds)) = display {
        unsafe {
            match bounds {
                Some(bounds) => super::overlay::position(window, bounds, mode),
                None => {
                    let _ = ShowWindow(window, SW_HIDE);
                }
            }
        }
    }
}
