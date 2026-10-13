pub fn watch_power() -> Result<impl FnOnce() + Send, String> {
    Err::<fn(), _>("Linux sleep/wake observer pending desktop-specific integration".into())
}
pub fn idle_ms() -> Result<u64, String> {
    if std::env::var("XDG_SESSION_TYPE").as_deref() == Ok("wayland")
        || std::env::var_os("WAYLAND_DISPLAY").is_some()
    {
        return Err("Wayland idle observation needs a compositor-supported service".into());
    }
    let xlib = x11_dl::xlib::Xlib::open().map_err(|e| e.to_string())?;
    let xss = x11_dl::xss::Xss::open().map_err(|e| e.to_string())?;
    // Query an idle counter only. Do not capture input events.
    unsafe {
        let display = (xlib.XOpenDisplay)(std::ptr::null());
        if display.is_null() {
            return Err("X11 display unavailable".into());
        }
        let mut event = 0;
        let mut error = 0;
        if (xss.XScreenSaverQueryExtension)(display, &mut event, &mut error) == 0 {
            (xlib.XCloseDisplay)(display);
            return Err("XScreenSaver extension unavailable".into());
        }
        let info = (xss.XScreenSaverAllocInfo)();
        if info.is_null() {
            (xlib.XCloseDisplay)(display);
            return Err("X11 idle allocation failed".into());
        }
        let ok = (xss.XScreenSaverQueryInfo)(display, (xlib.XDefaultRootWindow)(display), info);
        let idle = (*info).idle;
        (xlib.XFree)(info.cast());
        (xlib.XCloseDisplay)(display);
        if ok == 0 {
            Err("X11 idle query failed".into())
        } else {
            Ok(idle)
        }
    }
}
pub fn foreground() -> usize {
    0
}
pub fn restore_foreground(_id: usize) {}
