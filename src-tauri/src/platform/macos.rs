use super::{ASLEEP, WAKES};
use std::sync::atomic::Ordering;
unsafe extern "C" {
    fn awaytime_watch_power(callback: extern "C" fn(i32)) -> *mut std::ffi::c_void;
    fn awaytime_unwatch_power(handle: *mut std::ffi::c_void);
    fn awaytime_idle_seconds() -> f64;
    fn awaytime_foreground_app() -> std::ffi::c_long;
    fn awaytime_restore_app(pid: std::ffi::c_long);
}
extern "C" fn power_event(event: i32) {
    if event == 0 {
        ASLEEP.store(true, Ordering::SeqCst);
    } else if ASLEEP.swap(false, Ordering::SeqCst) {
        WAKES.fetch_add(1, Ordering::SeqCst);
    }
}
pub fn watch_power() -> Result<impl FnOnce() + Send, String> {
    let handle = unsafe { awaytime_watch_power(power_event) } as usize;
    if handle == 0 {
        return Err("Workspace notifications unavailable".into());
    }
    Ok(move || unsafe { awaytime_unwatch_power(handle as *mut _) })
}
pub fn idle_ms() -> Result<u64, String> {
    let seconds = unsafe { awaytime_idle_seconds() };
    if !seconds.is_finite() || seconds < 0.0 {
        return Err("Idle observation unavailable".into());
    }
    Ok((seconds * 1000.0).floor() as u64)
}
pub fn foreground() -> usize {
    unsafe { awaytime_foreground_app() as usize }
}
pub fn restore_foreground(id: usize) {
    unsafe { awaytime_restore_app(id as _) };
}
