use windows::{
    core::PWSTR,
    Win32::{
        Foundation::CloseHandle,
        System::Threading::{
            CreateProcessW, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOW,
        },
    },
};

pub fn launch_app(executable_path: &str, arguments: Option<&str>) -> Result<u32, String> {
    let cmd = match arguments {
        Some(args) => format!("\"{}\" {}", executable_path, args),
        None => format!("\"{}\"", executable_path),
    };

    let mut cmd_utf16: Vec<u16> = cmd.encode_utf16().chain(std::iter::once(0)).collect();

    let mut si = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut pi = PROCESS_INFORMATION::default();

    unsafe {
        CreateProcessW(
            None,
            PWSTR(cmd_utf16.as_mut_ptr()),
            None,
            None,
            false,
            PROCESS_CREATION_FLAGS(0),
            None,
            None,
            &mut si,
            &mut pi,
        )
        .map_err(|e| format!("CreateProcessW failed: {e}"))?;

        let pid = pi.dwProcessId;
        CloseHandle(pi.hProcess).ok();
        CloseHandle(pi.hThread).ok();
        Ok(pid)
    }
}
