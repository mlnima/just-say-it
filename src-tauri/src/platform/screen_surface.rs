use super::screen_paint::{self, Shape};
use dictation_core::{defaults, session::Phase};
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::MONITORINFO,
    UI::WindowsAndMessaging::*,
};

pub(super) struct ScreenSurface {
    windows: Vec<HWND>,
}

impl ScreenSurface {
    pub unsafe fn new(class: &WNDCLASSW) -> Result<Self, String> {
        let mut surface = Self {
            windows: Vec::new(),
        };
        for _ in 0..5 {
            let window = CreateWindowExW(
                WS_EX_LAYERED
                    | WS_EX_TRANSPARENT
                    | WS_EX_NOACTIVATE
                    | WS_EX_TOOLWINDOW
                    | WS_EX_TOPMOST,
                class.lpszClassName,
                windows::core::w!("Dictation activity"),
                WS_POPUP | WS_DISABLED,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(class.hInstance),
                None,
            )
            .map_err(|_| "Cannot create the screen overlay.")?;
            surface.windows.push(window);
            super::window_policy::exclude_from_capture(window)?;
        }
        Ok(surface)
    }

    pub fn control_window(&self) -> HWND {
        self.windows[0]
    }

    pub unsafe fn hide(&self) {
        for window in &self.windows {
            let _ = ShowWindow(*window, SW_HIDE);
        }
    }

    pub unsafe fn raise(&self) -> Result<(), String> {
        for window in &self.windows {
            SetWindowPos(
                *window,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
            )
            .map_err(|_| "Cannot keep the screen overlay visible.")?;
        }
        Ok(())
    }

    pub unsafe fn show(
        &self,
        info: &MONITORINFO,
        dpi: u32,
        phase: Phase,
        app_name: &str,
    ) -> Result<(), String> {
        let screen = info.rcMonitor;
        let width = screen.right - screen.left;
        let height = screen.bottom - screen.top;
        if width < 3 || height < 3 {
            return Err("The active display has no usable overlay area.".into());
        }
        let scale = |value: i32| {
            ((value as i64 * dpi as i64) / 96).clamp(1, i32::MAX as i64) as i32
        };
        let glow = scale(defaults::SCREEN_GLOW_WIDTH).min(width.min(height) / 3);
        let label_width = scale(defaults::SCREEN_LABEL_WIDTH).min(width);
        let label_height = scale(defaults::SCREEN_LABEL_HEIGHT).min(height);
        let label_x = screen.left + (width - label_width) / 2;
        let label_y = (info.rcWork.bottom - scale(defaults::SCREEN_LABEL_GAP) - label_height)
            .clamp(screen.top, screen.bottom - label_height);
        let regions = [
            (
                RECT { bottom: screen.top + glow, ..screen },
                Shape::Top,
            ),
            (
                RECT { top: screen.bottom - glow, ..screen },
                Shape::Bottom,
            ),
            (
                RECT {
                    left: screen.left,
                    right: screen.left + glow,
                    top: screen.top + glow,
                    bottom: screen.bottom - glow,
                },
                Shape::Left,
            ),
            (
                RECT {
                    left: screen.right - glow,
                    right: screen.right,
                    top: screen.top + glow,
                    bottom: screen.bottom - glow,
                },
                Shape::Right,
            ),
            (
                RECT {
                    left: label_x,
                    top: label_y,
                    right: label_x + label_width,
                    bottom: label_y + label_height,
                },
                Shape::Label,
            ),
        ];
        let color = match phase {
            Phase::Recording => defaults::SCREEN_RECORDING_COLOR,
            _ => defaults::SCREEN_PROCESSING_COLOR,
        };
        let state = match phase {
            Phase::Preparing => "Preparing dictation…",
            Phase::Recording => "Recording · Microphone on",
            Phase::Transcribing => "Processing speech · Microphone off",
            Phase::Inserting => "Inserting text · Microphone off",
            _ => return Err("The screen overlay received an inactive phase.".into()),
        };
        let label = format!("{app_name} · {state}");
        for (window, (bounds, shape)) in self.windows.iter().zip(regions) {
            screen_paint::draw(
                *window,
                bounds,
                shape,
                color,
                &label,
                scale(defaults::SCREEN_LABEL_FONT_SIZE),
            )?;
            SetWindowPos(
                *window,
                Some(HWND_TOPMOST),
                bounds.left,
                bounds.top,
                bounds.right - bounds.left,
                bounds.bottom - bounds.top,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )
            .map_err(|_| "Cannot display the screen overlay.")?;
        }
        Ok(())
    }
}

impl Drop for ScreenSurface {
    fn drop(&mut self) {
        // SAFETY: the surface is created, displayed and destroyed exclusively by its owning thread.
        unsafe {
            for window in &self.windows {
                let _ = DestroyWindow(*window);
            }
        }
    }
}
