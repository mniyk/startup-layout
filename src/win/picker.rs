use std::time::{Duration, Instant};

use windows::{
    Win32::{
        Foundation::{CloseHandle, HWND},
        System::{
            ProcessStatus::K32GetModuleFileNameExW,
            Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
        },
        UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
    },
};

use crate::win::window::get_window_rect;

#[derive(Debug, Clone)]
pub struct CapturedWindow {
    pub executable_path: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

fn capture_hwnd(hwnd: HWND) -> Option<CapturedWindow> {
    unsafe {
        let (x, y, width, height) = get_window_rect(hwnd)?;

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        let hprocess = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid).ok()?;
        let mut buf = vec![0u16; 1024];
        let len = K32GetModuleFileNameExW(hprocess, None, &mut buf);
        CloseHandle(hprocess).ok();

        if len == 0 {
            return None;
        }

        Some(CapturedWindow {
            executable_path: String::from_utf16_lossy(&buf[..len as usize]),
            x,
            y,
            width,
            height,
        })
    }
}

/// フォアグラウンドウィンドウが自プロセス以外に切り替わった瞬間にキャプチャする。
/// timeout_ms 以内に切り替わらなければ None を返す。
pub fn capture_on_focus_change(timeout_ms: u32) -> Option<CapturedWindow> {
    let our_pid = std::process::id();
    let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);

    loop {
        std::thread::sleep(Duration::from_millis(100));

        let hwnd = unsafe { GetForegroundWindow() };
        if !hwnd.0.is_null() {
            let mut pid = 0u32;
            unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); }

            if pid != our_pid {
                return capture_hwnd(hwnd);
            }
        }

        if Instant::now() >= deadline {
            return None;
        }
    }
}
