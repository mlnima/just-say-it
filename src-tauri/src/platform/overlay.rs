use dictation_core::defaults;
use windows::{
    core::w,
    Win32::{
        Foundation::{COLORREF, HWND, RECT},
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
};

pub(super) enum ButtonAction {
    Activate,
    Dismiss,
}

pub(super) fn hit_test(mode: u8, point: isize) -> Option<ButtonAction> {
    let x = point as i16 as i32;
    let y = (point >> 16) as i16 as i32;
    if !(0..defaults::OVERLAY_HEIGHT).contains(&y) || x < 0 {
        return None;
    }
    match mode {
        1 if (defaults::OVERLAY_WIDTH..width(mode)).contains(&x) => Some(ButtonAction::Dismiss),
        1 | 2 | 4 if x < defaults::OVERLAY_WIDTH => Some(ButtonAction::Activate),
        _ => None,
    }
}

fn width(mode: u8) -> i32 {
    defaults::OVERLAY_WIDTH
        + if mode == 1 {
            defaults::OVERLAY_DISMISS_WIDTH
        } else {
            0
        }
}

pub unsafe fn paint(window: HWND, mode: u8) {
    // Every GDI object created here is restored and deleted before EndPaint.
    let mut paint = PAINTSTRUCT::default();
    let dc = BeginPaint(window, &mut paint);
    let recording = mode == 2;
    let background = CreateSolidBrush(COLORREF(if recording { 0x00313ba4 } else { 0x00353f17 }));
    let pen = CreatePen(PS_SOLID, 1, COLORREF(0x00657552));
    let previous_brush = SelectObject(dc, background.into());
    let previous_pen = SelectObject(dc, pen.into());
    let _ = RoundRect(
        dc,
        0,
        0,
        width(mode),
        defaults::OVERLAY_HEIGHT,
        12,
        12,
    );
    let font = CreateFontW(
        -14,
        0,
        0,
        0,
        500,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY,
        0,
        w!("Segoe UI"),
    );
    let previous_font = SelectObject(dc, font.into());
    SetBkMode(dc, TRANSPARENT);
    SetTextColor(dc, COLORREF(0x00f8fff1));
    let idle = !matches!(mode, 2..=4);
    if idle {
        // Tauri embeds the icon as IDI_APPLICATION; Windows owns the shared handle.
        let icon = GetModuleHandleW(None).and_then(|module| {
            LoadImageW(
                Some(module.into()),
                IDI_APPLICATION,
                IMAGE_ICON,
                20,
                20,
                LR_SHARED,
            )
        });
        if let Ok(icon) = icon {
            let _ = DrawIconEx(
                dc,
                14,
                (defaults::OVERLAY_HEIGHT - 20) / 2,
                HICON(icon.0),
                20,
                20,
                0,
                None,
                DI_NORMAL,
            );
        }
    }
    let label = match mode {
        2 => "■  Finish",
        3 => "…  Working",
        4 => "↳  Insert",
        _ => "Dictate",
    };
    let mut label: Vec<u16> = label.encode_utf16().collect();
    let mut bounds = RECT {
        left: if idle { 32 } else { 0 },
        top: 0,
        right: defaults::OVERLAY_WIDTH,
        bottom: defaults::OVERLAY_HEIGHT,
    };
    DrawTextW(
        dc,
        &mut label,
        &mut bounds,
        DT_SINGLELINE | DT_CENTER | DT_VCENTER,
    );
    if mode == 1 {
        let separator = defaults::OVERLAY_WIDTH;
        let _ = MoveToEx(dc, separator, 7, None);
        let _ = LineTo(dc, separator, defaults::OVERLAY_HEIGHT - 7);
        let mut close_bounds = RECT {
            left: separator,
            top: 0,
            right: width(mode),
            bottom: defaults::OVERLAY_HEIGHT,
        };
        DrawTextW(
            dc,
            &mut [0x00d7],
            &mut close_bounds,
            DT_SINGLELINE | DT_CENTER | DT_VCENTER,
        );
    }
    SelectObject(dc, previous_font);
    SelectObject(dc, previous_pen);
    SelectObject(dc, previous_brush);
    let _ = DeleteObject(font.into());
    let _ = DeleteObject(pen.into());
    let _ = DeleteObject(background.into());
    let _ = EndPaint(window, &paint);
}

pub unsafe fn position(window: HWND, bounds: RECT, mode: u8) {
    let monitor = MonitorFromRect(&bounds, MONITOR_DEFAULTTONEAREST);
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !GetMonitorInfoW(monitor, &mut info).as_bool() {
        let _ = ShowWindow(window, SW_HIDE);
        return;
    }
    let work = info.rcWork;
    let width = width(mode);
    let height = defaults::OVERLAY_HEIGHT;
    let x = bounds
        .left
        .clamp(work.left, (work.right - width).max(work.left));
    let above = bounds.top - height - defaults::OVERLAY_GAP;
    let y = if above >= work.top {
        above
    } else {
        bounds.bottom + defaults::OVERLAY_GAP
    }
    .clamp(work.top, (work.bottom - height).max(work.top));
    let _ = SetWindowPos(
        window,
        Some(HWND_TOPMOST),
        x,
        y,
        width,
        height,
        SWP_NOACTIVATE | SWP_SHOWWINDOW,
    );
    let _ = InvalidateRect(Some(window), None, false);
}
