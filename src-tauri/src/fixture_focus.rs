#![cfg(windows)]
use windows::Win32::{
    Foundation::{HWND, LPARAM, POINT},
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

static POINTER: std::sync::Mutex<Option<(POINT, POINT)>> = std::sync::Mutex::new(None);

pub fn park(point: POINT, processes: &[u32]) {
    unsafe {
        let mut process = 0;
        GetWindowThreadProcessId(WindowFromPoint(point), Some(&mut process));
        assert!(
            processes.contains(&process),
            "Fixture is covered; refusing to move into another application."
        );
        let mut previous = POINT::default();
        GetCursorPos(&mut previous).unwrap();
        SetCursorPos(point.x, point.y).unwrap();
        let mut pointer = POINTER.lock().unwrap();
        let original = pointer.map(|(original, _)| original).unwrap_or(previous);
        *pointer = Some((original, point));
    }
}

pub fn restore() {
    if let Some((previous, placed)) = POINTER.lock().unwrap().take() {
        unsafe {
            let mut current = POINT::default();
            if GetCursorPos(&mut current).is_ok() && current.x == placed.x && current.y == placed.y
            {
                let _ = SetCursorPos(previous.x, previous.y);
            }
        }
    }
}

struct Search {
    process: u32,
    windows: Vec<HWND>,
}

unsafe extern "system" fn collect(window: HWND, data: LPARAM) -> windows::core::BOOL {
    let search = &mut *(data.0 as *mut Search);
    let mut process = 0;
    GetWindowThreadProcessId(window, Some(&mut process));
    if process == search.process && IsWindowVisible(window).as_bool() {
        search.windows.push(window);
    }
    true.into()
}

pub fn first_editor(process: u32) {
    // Test-only physical focus. Never click a point unless the visible hit belongs to this fixture.
    unsafe {
        let mut search = Search {
            process,
            windows: Vec::new(),
        };
        let _ = EnumWindows(Some(collect), LPARAM((&mut search as *mut Search) as isize));
        let top = search.windows.clone();
        search.windows.clear();
        for window in top {
            let _ = EnumChildWindows(
                Some(window),
                Some(collect),
                LPARAM((&mut search as *mut Search) as isize),
            );
        }
        let mut editors: Vec<_> = search
            .windows
            .into_iter()
            .filter_map(|window| {
                let mut class = [0_u16; 256];
                let count = GetClassNameW(window, &mut class);
                if !String::from_utf16_lossy(&class[..count as usize]).contains("EDIT") {
                    return None;
                }
                let mut bounds = Default::default();
                GetWindowRect(window, &mut bounds).ok()?;
                Some((window, bounds))
            })
            .collect();
        editors.sort_by_key(|(_, bounds)| bounds.top);
        let (editor, bounds) = editors
            .first()
            .expect("Fixture did not create a visible editor.");
        let point = POINT {
            x: (bounds.left + bounds.right) / 2,
            y: (bounds.top + bounds.bottom) / 2,
        };
        let mut hit_process = 0;
        GetWindowThreadProcessId(WindowFromPoint(point), Some(&mut hit_process));
        assert_eq!(
            hit_process, process,
            "Fixture is covered; refusing to click another application."
        );
        park(point, &[process]);
        let inputs = [MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP].map(|flags| INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dwFlags: flags,
                    ..Default::default()
                },
            },
        });
        assert_eq!(SendInput(&inputs, std::mem::size_of::<INPUT>() as i32), 2);
        let thread = GetWindowThreadProcessId(*editor, None);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            let mut information = GUITHREADINFO {
                cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
                ..Default::default()
            };
            let mut foreground_process = 0;
            GetWindowThreadProcessId(GetForegroundWindow(), Some(&mut foreground_process));
            if GetGUIThreadInfo(thread, &mut information).is_ok()
                && information.hwndFocus == *editor
                && foreground_process == process
                && GetAsyncKeyState(VK_LBUTTON.0 as i32) >= 0
            {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "The fixture did not receive the focus click."
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
