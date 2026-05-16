use windows::{
    Win32::{
        Foundation::{BOOL, LPARAM, RECT},
        Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
        },
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct MonitorInfo {
    pub index: usize,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub is_primary: bool,
}

struct EnumState {
    monitors: Vec<MonitorInfo>,
}

unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let state = &mut *(lparam.0 as *mut EnumState);

    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    if GetMonitorInfoW(hmonitor, &mut info).as_bool() {
        let rc = info.rcMonitor;
        let is_primary = (info.dwFlags & 1) != 0;
        let index = state.monitors.len();
        state.monitors.push(MonitorInfo {
            index,
            x: rc.left,
            y: rc.top,
            width: rc.right - rc.left,
            height: rc.bottom - rc.top,
            is_primary,
        });
    }
    BOOL(1)
}

pub fn enumerate_monitors() -> Vec<MonitorInfo> {
    let mut state = EnumState { monitors: Vec::new() };
    unsafe {
        let _ = EnumDisplayMonitors(
            HDC::default(),
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut state as *mut _ as isize),
        );
    }
    state.monitors.sort_by(|a, b| {
        b.is_primary.cmp(&a.is_primary).then(a.x.cmp(&b.x))
    });
    for (i, m) in state.monitors.iter_mut().enumerate() {
        m.index = i;
    }
    state.monitors
}

pub fn resolve_position(x: i32, y: i32, monitor_index: Option<usize>) -> (i32, i32) {
    let monitors = enumerate_monitors();
    match monitor_index {
        Some(idx) => {
            if let Some(m) = monitors.get(idx) {
                (m.x + x, m.y + y)
            } else {
                (x, y)
            }
        }
        None => (x, y),
    }
}
