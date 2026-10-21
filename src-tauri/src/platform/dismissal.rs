use super::windows::{Focus, Target};
use dictation_core::defaults;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{KillTimer, SetTimer},
};

pub(super) const EXPIRY_TIMER: usize = 1;

struct Dismissal {
    process_id: u32,
    window: isize,
    document: Option<Vec<i32>>,
    expires: Instant,
}

#[derive(Default)]
pub(super) struct Dismissals(VecDeque<Dismissal>);

impl Dismissals {
    pub fn dismiss(&mut self, focus: &Focus, target: &Target, window: HWND) {
        self.expire(window);
        let document = focus.document_id(target);
        self.0.retain(|entry| {
            entry.window != target.window
                || entry.process_id != target.process_id
                || entry.document != document
        });
        if self.0.len() >= defaults::OVERLAY_DISMISS_CAPACITY {
            self.0.pop_front();
        }
        self.0.push_back(Dismissal {
            process_id: target.process_id,
            window: target.window,
            document,
            expires: Instant::now() + Duration::from_secs(defaults::OVERLAY_DISMISS_SECONDS),
        });
        self.schedule(window);
    }

    pub fn hidden(&self, focus: &Focus, target: &Target) -> bool {
        let same_window = |entry: &Dismissal| {
            entry.window == target.window && entry.process_id == target.process_id
        };
        if !self.0.iter().any(same_window) {
            return false;
        }
        let document = focus.document_id(target);
        self.0.iter().any(|entry| {
            same_window(entry)
                && (entry.document.is_none() || document.is_none() || entry.document == document)
        })
    }

    pub fn expire(&mut self, window: HWND) {
        let now = Instant::now();
        self.0.retain(|entry| entry.expires > now);
        self.schedule(window);
    }

    fn schedule(&self, window: HWND) {
        // A timer wakes the existing message loop only at the next expiry.
        unsafe {
            let _ = KillTimer(Some(window), EXPIRY_TIMER);
            if let Some(expires) = self.0.iter().map(|entry| entry.expires).min() {
                let delay = expires
                    .saturating_duration_since(Instant::now())
                    .as_millis()
                    .clamp(1, u32::MAX as u128) as u32;
                SetTimer(Some(window), EXPIRY_TIMER, delay, None);
            }
        }
    }
}
