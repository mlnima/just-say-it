use dictation_core::defaults;
use tauri::{menu::ContextMenu, tray::TrayIcon};
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::{
        Controls::{DRAWITEMSTRUCT, MEASUREITEMSTRUCT, ODT_MENU},
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::*,
    },
};

pub(super) fn apply(tray: &TrayIcon, menu: &tauri::menu::Menu<tauri::Wry>) -> tauri::Result<()> {
    let popup = menu.hpopupmenu()?;
    tray.with_inner_tray_icon(move |tray| unsafe {
        let window = HWND(tray.window_handle());
        // Tauri owns both handles on this UI thread and retains the menu for the tray's lifetime.
        SetWindowSubclass(window, Some(callback), popup as usize, popup as usize).ok()?;
        let menu = HMENU(popup as _);
        for position in 0..GetMenuItemCount(Some(menu)) {
            let mut item = MENUITEMINFOW {
                cbSize: size_of::<MENUITEMINFOW>() as u32,
                fMask: MIIM_FTYPE,
                ..Default::default()
            };
            GetMenuItemInfoW(menu, position as u32, true, &mut item)?;
            if item.fType.contains(MFT_SEPARATOR) {
                continue;
            }
            // Only the bitmap slot is measured by us. Windows still renders the real text,
            // selection, theme and accessibility; no extra menu entries are introduced.
            item.fMask = MIIM_BITMAP;
            item.hbmpItem = HBMMENU_CALLBACK;
            SetMenuItemInfoW(menu, position as u32, true, &item)?;
        }
        Ok::<_, windows::core::Error>(())
    })?
    .map_err(|error| tauri::Error::Io(std::io::Error::other(error)))
}

unsafe extern "system" fn callback(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    popup: usize,
) -> LRESULT {
    // Windows supplies these structures only for their corresponding synchronous messages.
    match message {
        WM_MEASUREITEM if lparam.0 != 0 => {
            let item = &mut *(lparam.0 as *mut MEASUREITEMSTRUCT);
            if item.CtlType == ODT_MENU {
                item.itemWidth = 0;
                item.itemHeight =
                    (GetSystemMetrics(SM_CYMENU) + defaults::TRAY_MENU_VERTICAL_PADDING) as u32;
                return LRESULT(1);
            }
        }
        WM_DRAWITEM if lparam.0 != 0 => {
            let item = &*(lparam.0 as *const DRAWITEMSTRUCT);
            if item.CtlType == ODT_MENU && item.hwndItem.0 as usize == popup {
                // The sizing slot stays empty; the surrounding row is drawn by Windows.
                return LRESULT(1);
            }
        }
        WM_NCDESTROY => {
            let _ = RemoveWindowSubclass(window, Some(callback), id);
        }
        _ => {}
    }
    DefSubclassProc(window, message, wparam, lparam)
}
