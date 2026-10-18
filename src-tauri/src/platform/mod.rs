#[cfg(windows)]
mod overlay;
#[cfg(not(windows))]
mod unavailable;
#[cfg(windows)]
mod watcher;
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
pub use unavailable::*;
#[cfg(windows)]
pub use windows::*;
