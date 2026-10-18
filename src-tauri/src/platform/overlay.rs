use dictation_core::defaults;
use windows::{
    core::w,
    Win32::{
        Foundation::{COLORREF, HWND, RECT},
        Graphics::Gdi::*,
        UI::WindowsAndMessaging::*,
    },
};

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
        defaults::OVERLAY_WIDTH,
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
    let label = match mode {
        2 => "■  Finish",
        3 => "…  Working",
        4 => "↳  Insert",
        _ => "●  Dictate",
    };
    let mut label: Vec<u16> = label.encode_utf16().collect();
    let mut bounds = RECT {
        left: 0,
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
    SelectObject(dc, previous_font);
    SelectObject(dc, previous_pen);
    SelectObject(dc, previous_brush);
    let _ = DeleteObject(font.into());
    let _ = DeleteObject(pen.into());
    let _ = DeleteObject(background.into());
    let _ = EndPaint(window, &paint);
}

pub unsafe fn position(window: HWND, bounds: RECT) {
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
    let width = defaults::OVERLAY_WIDTH;
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
