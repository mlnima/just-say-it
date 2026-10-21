use super::screen_surface::ScreenSurface;
use crate::runtime::Runtime;
use dictation_core::session::Phase;
use std::sync::atomic::Ordering;
use windows::Win32::{
    Foundation::RECT,
    Graphics::Gdi::*,
    UI::{HiDpi::GetDpiForWindow, WindowsAndMessaging::*},
};

#[derive(PartialEq)]
struct Presentation {
    phase: Phase,
    monitor: RECT,
    work: RECT,
    dpi: u32,
}

pub(super) struct Indicator {
    runtime: Runtime,
    surface: ScreenSurface,
    rendered: Option<Presentation>,
    failed: bool,
}

impl Indicator {
    pub fn new(runtime: Runtime, surface: ScreenSurface) -> Self {
        Self {
            runtime,
            surface,
            rendered: None,
            failed: false,
        }
    }

    pub unsafe fn fail(&mut self, error: String) {
        self.failed = true;
        self.surface.hide();
        self.runtime.update(|s| {
            s.screen_overlay_available = false;
            s.screen_overlay_error = Some(error);
        });
    }

    // All native calls run on the overlay's own message thread, outside any state borrow.
    pub unsafe fn refresh(&mut self, restack: bool) -> Result<(), String> {
        if self.failed {
            return Ok(());
        }
        let snapshot = self.runtime.snapshot()?;
        let active = snapshot.settings.screen_overlay == Some(true)
            && !snapshot.settings.paused
            && !self.runtime.stopping.load(Ordering::Acquire)
            && self.runtime.interrupt.load(Ordering::Acquire) == 0
            && self.runtime.authorization.load(Ordering::Acquire) != 0
            && matches!(
                snapshot.phase,
                Phase::Preparing | Phase::Recording | Phase::Transcribing | Phase::Inserting
            );
        if !active {
            if self.rendered.take().is_some() {
                self.surface.hide();
            }
            return Ok(());
        }
        let monitor = MonitorFromWindow(GetForegroundWindow(), MONITOR_DEFAULTTOPRIMARY);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return Err("Cannot determine the screen overlay bounds.".into());
        }
        // The overlay thread is per-monitor aware. Position its control edge before querying DPI
        // so an unaware foreground application's virtualized DPI cannot distort the overlay.
        let moved = self.rendered.as_ref().is_none_or(|s| s.monitor != info.rcMonitor);
        if moved {
            SetWindowPos(
                self.surface.control_window(),
                None,
                info.rcMonitor.left,
                info.rcMonitor.top,
                1,
                1,
                SWP_NOACTIVATE | SWP_NOZORDER,
            )
            .map_err(|_| "Cannot position the screen overlay.")?;
        }
        let presentation = Presentation {
            phase: snapshot.phase,
            monitor: info.rcMonitor,
            work: info.rcWork,
            dpi: GetDpiForWindow(self.surface.control_window()).max(96),
        };
        if self.rendered.as_ref() != Some(&presentation) {
            self.surface.show(
                &info,
                presentation.dpi,
                snapshot.phase,
                &snapshot.app_name,
            )?;
            self.rendered = Some(presentation);
        } else if restack {
            self.surface.raise()?;
        }
        Ok(())
    }
}
