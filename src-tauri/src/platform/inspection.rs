use super::windows::Target;
use windows::Win32::{
    Foundation::HWND,
    System::{Com::SAFEARRAY, Ole::*, Variant::VT_BOOL},
    UI::{Accessibility::*, WindowsAndMessaging::*},
};

pub unsafe fn inspect(automation: &IUIAutomation) -> Target {
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
    let Some(element) = focused_element(automation, foreground) else {
        return Target {
            window,
            ..Default::default()
        };
    };
    let process_id = element.CurrentProcessId().unwrap_or_default() as u32;
    let password = element
        .CurrentIsPassword()
        .map(|b| b.as_bool())
        .ok();
    // Field-based delivery requires an observed non-password field. A shortcut
    // may explicitly select a window, but never a known password control.
    let secure = password.unwrap_or(true);
    let focused = element.CurrentHasKeyboardFocus().is_ok_and(|b| b.as_bool());
    let enabled = element.CurrentIsEnabled().is_ok_and(|b| b.as_bool());
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
        editable: focused && enabled && editable(&element) && foreground == GetForegroundWindow(),
        secure,
        own_process: process_id == std::process::id(),
        known_password: password == Some(true),
        keyboard_window: None,
    }
}

unsafe fn focused_element(
    automation: &IUIAutomation,
    foreground: HWND,
) -> Option<IUIAutomationElement> {
    let mut information = GUITHREADINFO {
        cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };
    GetGUIThreadInfo(0, &mut information).ok()?;
    if information.hwndFocus.is_invalid()
        || GetAncestor(information.hwndFocus, GA_ROOT) != foreground
    {
        return None;
    }
    let element = automation.GetFocusedElement().ok()?;
    if element.CurrentHasKeyboardFocus().ok()?.as_bool()
        || !element.CurrentIsPassword().is_ok_and(|b| !b.as_bool())
    {
        return Some(element);
    }
    // Some hosted webviews report their native panel instead of the focused editor.
    // Search only inside the OS-focused native window, never the desktop or another app.
    let host = automation.ElementFromHandle(information.hwndFocus).ok()?;
    let condition = automation
        .CreatePropertyCondition(UIA_HasKeyboardFocusPropertyId, &true.into())
        .ok()?;
    host.FindFirst(TreeScope_Descendants, &condition).ok()
}

pub(super) unsafe fn document_id(
    automation: &IUIAutomation,
    target: &Target,
) -> Option<Vec<i32>> {
    if !target.known() || GetForegroundWindow().0 as isize != target.window {
        return None;
    }
    let mut element = focused_element(automation, HWND(target.window as _))?;
    if read_runtime_id(element.GetRuntimeId().ok()?).ok()? != target.runtime_id {
        return None;
    }
    let walker = automation.ControlViewWalker().ok()?;
    let started = std::time::Instant::now();
    let budget = std::time::Duration::from_millis(
        dictation_core::defaults::ACCESSIBILITY_TIMEOUT_MILLIS.into(),
    );
    let mut document = None;
    // Use the outer document so editors and frames on one page share a dismissal.
    // Runtime IDs stay in memory; no page URL, title or content is read.
    for _ in 0..dictation_core::defaults::ACCESSIBILITY_MAX_ANCESTORS {
        if started.elapsed() >= budget {
            return None;
        }
        let control = element.CurrentControlType().ok()?;
        if control == UIA_WindowControlTypeId {
            return document;
        }
        if control == UIA_DocumentControlTypeId {
            let id = read_runtime_id(element.GetRuntimeId().ok()?).ok()?;
            if !id.is_empty() {
                document = Some(id);
            }
        }
        let Ok(parent) = walker.GetParentElement(&element) else {
            return document;
        };
        element = parent;
    }
    None
}

unsafe fn editable(element: &IUIAutomationElement) -> bool {
    let value: Result<IUIAutomationValuePattern, _> =
        element.GetCurrentPatternAs(UIA_ValuePatternId);
    if let Ok(pattern) = value {
        return pattern.CurrentIsReadOnly().is_ok_and(|b| !b.as_bool());
    }
    let control = element.CurrentControlType().unwrap_or_default();
    if control != UIA_EditControlTypeId && control != UIA_DocumentControlTypeId {
        return false;
    }
    let text: Result<IUIAutomationTextPattern, _> = element.GetCurrentPatternAs(UIA_TextPatternId);
    text.and_then(|pattern| pattern.DocumentRange())
        .and_then(|range| range.GetAttributeValue(UIA_IsReadOnlyAttributeId))
        .is_ok_and(|value| value.vt() == VT_BOOL && bool::try_from(&value) == Ok(false))
}

unsafe fn read_runtime_id(array: *mut SAFEARRAY) -> windows::core::Result<Vec<i32>> {
    // UIA returns a caller-owned one-dimensional VT_I4 array. Always release it.
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
