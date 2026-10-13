use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
static ASLEEP: AtomicBool = AtomicBool::new(false);
static WAKES: AtomicU64 = AtomicU64::new(0);
pub fn asleep() -> bool {
    ASLEEP.load(Ordering::SeqCst)
}
pub fn wake_count() -> u64 {
    WAKES.load(Ordering::SeqCst)
}

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;
