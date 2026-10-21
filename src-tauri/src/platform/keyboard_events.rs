use windows::Win32::UI::Input::KeyboardAndMouse::*;

pub(super) fn events(text: &str, layout: Option<HKL>) -> Vec<INPUT> {
    let mut inputs = Vec::new();
    // Only read Caps Lock; never change the user's keyboard toggles.
    let caps = unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1 != 0;
    let shift_scan = layout.map_or(0, |layout| unsafe {
        MapVirtualKeyExW(VK_LSHIFT.0.into(), MAPVK_VK_TO_VSC_EX, Some(layout))
    });
    for character in text.chars() {
        let mapped = layout.and_then(|layout| scan_key(character, layout, caps));
        if let Some((scan, shifted)) = mapped.filter(|(_, shifted)| !*shifted || shift_scan != 0) {
            if shifted {
                inputs.push(scan_event(shift_scan, false));
            }
            inputs.extend([scan_event(scan, false), scan_event(scan, true)]);
            if shifted {
                inputs.push(scan_event(shift_scan, true));
            }
        } else {
            let mut units = [0; 2];
            for unit in character.encode_utf16(&mut units).iter().copied() {
                inputs.extend([
                    input(unit, KEYEVENTF_UNICODE),
                    input(unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP),
                ]);
            }
        }
    }
    inputs
}

pub(super) fn release_pressed(accepted: &[INPUT]) {
    let mut pressed: Vec<KEYBDINPUT> = Vec::new();
    for event in accepted {
        // Every event was initialized as INPUT_KEYBOARD by this module.
        let key = unsafe { event.Anonymous.ki };
        let flags = key.dwFlags & !KEYEVENTF_KEYUP;
        let same = |other: &KEYBDINPUT| {
            other.wScan == key.wScan && other.dwFlags == flags
        };
        if key.dwFlags & KEYEVENTF_KEYUP != KEYBD_EVENT_FLAGS(0) {
            pressed.retain(|other| !same(other));
        } else {
            pressed.push(key);
        }
    }
    let releases: Vec<INPUT> = pressed.into_iter().rev()
        .map(|key| input(key.wScan, key.dwFlags | KEYEVENTF_KEYUP)).collect();
    if !releases.is_empty() {
        // Release only synthetic keys accepted without their matching key-up.
        // This never resends text or retries the partial delivery.
        unsafe { SendInput(&releases, std::mem::size_of::<INPUT>() as i32); }
    }
}

fn scan_key(character: char, layout: HKL, caps: bool) -> Option<(u32, bool)> {
    let unit = u16::try_from(character as u32).ok()?;
    // Mapping is read-only. Ctrl/Alt combinations and dead keys are not used
    // for text, since they could activate application commands.
    unsafe {
        let mapped = VkKeyScanExW(unit, layout);
        if mapped < 0 || mapped as u16 >> 8 > 1 {
            return None;
        }
        let key = mapped as u32 & 0xff;
        let scan = MapVirtualKeyExW(key, MAPVK_VK_TO_VSC_EX, Some(layout));
        if scan == 0 || scan >> 8 == 0xe1 {
            return None;
        }
        let shifted = mapped as u16 & 0x100 != 0;
        for shifted in [shifted, !shifted] {
            let mut state = [0; 256];
            state[VK_CAPITAL.0 as usize] = u8::from(caps);
            state[VK_SHIFT.0 as usize] = if shifted { 0x80 } else { 0 };
            state[key as usize] = 0x80;
            let mut output = [0; 8];
            // Bit 2 prevents changes to dead-key/keyboard state (Windows 10+).
            let count = ToUnicodeEx(key, scan, &state, &mut output, 4, Some(layout));
            if count == 1 && output[0] == unit {
                return Some((scan, shifted));
            }
        }
    }
    None
}

fn scan_event(scan: u32, up: bool) -> INPUT {
    let extended = if scan >> 8 == 0xe0 { KEYEVENTF_EXTENDEDKEY } else { KEYBD_EVENT_FLAGS(0) };
    let released = if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) };
    input((scan & 0xff) as u16, KEYEVENTF_SCANCODE | extended | released)
}

fn input(scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
