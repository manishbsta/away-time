use super::{ASLEEP, WAKES};
use std::sync::atomic::Ordering;
use windows_sys::Win32::{
    Foundation::*,
    System::{LibraryLoader::GetModuleHandleW, SystemInformation::GetTickCount},
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_POWERBROADCAST => {
            if wparam as u32 == PBT_APMSUSPEND {
                ASLEEP.store(true, Ordering::SeqCst);
            }
            if wparam as u32 == PBT_APMRESUMEAUTOMATIC && ASLEEP.swap(false, Ordering::SeqCst) {
                WAKES.fetch_add(1, Ordering::SeqCst);
            }
            1
        }
        WM_CLOSE => {
            unsafe { DestroyWindow(hwnd) };
            0
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}
pub fn watch_power() -> Result<impl FnOnce() + Send, String> {
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let thread = std::thread::spawn(move || unsafe {
        let class: Vec<u16> = "AwayTimePowerObserver\0".encode_utf16().collect();
        let instance = GetModuleHandleW(std::ptr::null());
        let wc = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: class.as_ptr(),
            ..std::mem::zeroed()
        };
        if RegisterClassW(&wc) == 0 {
            let _ = sender.send(Err("Power observer class unavailable".to_string()));
            return;
        }
        let hwnd = CreateWindowExW(
            0,
            class.as_ptr(),
            class.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance,
            std::ptr::null(),
        );
        if hwnd.is_null() {
            let _ = sender.send(Err("Power observer window unavailable".to_string()));
            UnregisterClassW(class.as_ptr(), instance);
            return;
        }
        let _ = sender.send(Ok(hwnd as usize));
        let mut message = std::mem::zeroed();
        while GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        UnregisterClassW(class.as_ptr(), instance);
    });
    let hwnd = receiver.recv().map_err(|e| e.to_string())??;
    Ok(move || {
        unsafe { PostMessageW(hwnd as HWND, WM_CLOSE, 0, 0) };
        let _ = thread.join();
    })
}
pub fn idle_ms() -> Result<u64, String> {
    let mut info = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    if unsafe { GetLastInputInfo(&mut info) } == 0 {
        return Err("Idle observation unavailable".into());
    }
    Ok(unsafe { GetTickCount() }.wrapping_sub(info.dwTime) as u64)
}
pub fn foreground() -> usize {
    unsafe { GetForegroundWindow() as usize }
}
pub fn restore_foreground(id: usize) {
    unsafe { SetForegroundWindow(id as HWND) };
}
