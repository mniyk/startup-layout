use std::time::{Duration, Instant};

use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{CloseHandle, BOOL, HWND, LPARAM, RECT},
        System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
            PROCESS_QUERY_LIMITED_INFORMATION,
        },
        UI::WindowsAndMessaging::{
            EnumWindows, GetWindowRect, GetWindowTextLengthW, GetWindowThreadProcessId,
            IsWindowVisible, SetWindowPos, SWP_NOZORDER, SWP_SHOWWINDOW, HWND_TOP,
        },
    },
};

// ─── PID based search ────────────────────────────────────────────────────────

struct EnumData {
    target_pid: u32,
    found: Option<HWND>,
}

unsafe extern "system" fn enum_proc_pid(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut EnumData);
    if IsWindowVisible(hwnd).as_bool() {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == data.target_pid && GetWindowTextLengthW(hwnd) > 0 {
            data.found = Some(hwnd);
            return BOOL(0);
        }
    }
    BOOL(1)
}

pub fn find_window_by_pid(pid: u32, timeout_ms: u32) -> Option<HWND> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);
    loop {
        let mut data = EnumData { target_pid: pid, found: None };
        unsafe { let _ = EnumWindows(Some(enum_proc_pid), LPARAM(&mut data as *mut _ as isize)); }
        if let Some(hwnd) = data.found { return Some(hwnd); }
        if Instant::now() >= deadline { break; }
        std::thread::sleep(Duration::from_millis(200));
    }
    None
}

// ─── Exe-path based search (fallback for Chromium-family apps) ───────────────

struct ExeCollector {
    windows: Vec<(HWND, u32)>,
}

unsafe extern "system" fn enum_proc_collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let c = &mut *(lparam.0 as *mut ExeCollector);
    if IsWindowVisible(hwnd).as_bool() && GetWindowTextLengthW(hwnd) > 0 {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        c.windows.push((hwnd, pid));
    }
    BOOL(1)
}

fn get_exe_path_for_pid(pid: u32) -> Option<String> {
    unsafe {
        let hprocess = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = vec![0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            hprocess,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        );
        CloseHandle(hprocess).ok();
        if ok.is_ok() && len > 0 {
            Some(String::from_utf16_lossy(&buf[..len as usize]))
        } else {
            None
        }
    }
}

/// Fallback: find any visible window whose process exe path matches.
/// Used for Chromium-based apps (Brave, Chrome, etc.) that re-use an existing process.
pub fn find_window_by_exe_path(exe_path: &str, timeout_ms: u32) -> Option<HWND> {
    let target_lower = exe_path.to_lowercase();
    let deadline = Instant::now() + Duration::from_millis(timeout_ms as u64);

    loop {
        let mut collector = ExeCollector { windows: Vec::new() };
        unsafe {
            let _ = EnumWindows(
                Some(enum_proc_collect),
                LPARAM(&mut collector as *mut _ as isize),
            );
        }

        for (hwnd, pid) in &collector.windows {
            if let Some(path) = get_exe_path_for_pid(*pid) {
                if path.to_lowercase() == target_lower {
                    return Some(*hwnd);
                }
            }
        }

        if Instant::now() >= deadline { break; }
        std::thread::sleep(Duration::from_millis(200));
    }
    None
}

// ─── Position / rect ─────────────────────────────────────────────────────────

pub fn set_window_position(hwnd: HWND, x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
    unsafe {
        SetWindowPos(hwnd, HWND_TOP, x, y, width, height, SWP_NOZORDER | SWP_SHOWWINDOW)
            .map_err(|e| format!("SetWindowPos failed: {e}"))
    }
}

pub fn get_window_rect(hwnd: HWND) -> Option<(i32, i32, i32, i32)> {
    let mut rect = RECT::default();
    unsafe {
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            Some((rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top))
        } else {
            None
        }
    }
}
