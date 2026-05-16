mod models;
mod storage;
mod ui;
mod win;

use dioxus::prelude::*;
use ui::app::App;

fn main() {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::UI::HiDpi::{
            SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        };
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    let config = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("Startup Layout")
                .with_inner_size(dioxus::desktop::LogicalSize::new(720.0, 540.0))
                .with_min_inner_size(dioxus::desktop::LogicalSize::new(480.0, 360.0)),
        )
        .with_disable_context_menu(true);

    LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
}
