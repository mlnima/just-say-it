#[cfg(windows)]
mod dismissal;
#[cfg(windows)]
mod focus_events;
#[cfg(windows)]
mod inspection;
#[cfg(windows)]
mod keyboard_events;
#[cfg(windows)]
mod keyboard_target;
#[cfg(windows)]
mod overlay;
#[cfg(windows)]
mod screen_indicator;
#[cfg(windows)]
mod screen_overlay;
#[cfg(windows)]
mod screen_paint;
#[cfg(windows)]
mod screen_surface;
#[cfg(not(windows))]
mod unavailable;
#[cfg(windows)]
mod watcher;
#[cfg(windows)]
mod window_policy;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
pub use unavailable::*;
#[cfg(windows)]
pub use keyboard_target::ShortcutTarget;
#[cfg(windows)]
pub use screen_overlay::{start as start_screen_overlay, ScreenOverlay};
#[cfg(windows)]
pub use windows::*;
