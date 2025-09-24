use crate::utils::{HIDE_WINDOW_FLAG, Println};
use ansi_term::Color::{Red, Yellow};
use std::env;
use std::os::windows::process::CommandExt;
use std::process::{Command, exit};

/// https://github.com/yandexx/is_elevated
/// https://www.180it.com/archives/2099/
/// based on https://stackoverflow.com/a/8196291
/// Returns a boolean value, indicating whether the current process is elevated.
/// ## Example
/// ```rust
/// use windows_tool::elevated::is_elevated;
/// if !is_elevated() {
///     println!(
///         "Warning: the program isn’t running as elevated; some functionality may not work."
///     );
/// }
/// ```
pub fn is_elevated() -> bool {
    unsafe {
        use std::mem;
        use winapi::shared::minwindef::DWORD;
        use winapi::shared::minwindef::LPVOID;
        use winapi::um::processthreadsapi::GetCurrentProcess;
        use winapi::um::processthreadsapi::OpenProcessToken;
        use winapi::um::securitybaseapi::GetTokenInformation;
        use winapi::um::winnt::HANDLE;
        use winapi::um::winnt::TOKEN_ELEVATION;
        use winapi::um::winnt::TOKEN_QUERY;
        use winapi::um::winnt::TokenElevation;

        let mut current_token_ptr: HANDLE = mem::zeroed();
        let mut token_elevation: TOKEN_ELEVATION = mem::zeroed();
        let token_elevation_type_ptr: *mut TOKEN_ELEVATION = &mut token_elevation;
        let mut size: DWORD = 0;

        let result = OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut current_token_ptr);

        if result != 0 {
            let result = GetTokenInformation(
                current_token_ptr,
                TokenElevation,
                token_elevation_type_ptr as LPVOID,
                size_of::<winapi::um::winnt::TOKEN_ELEVATION_TYPE>() as u32,
                &mut size,
            );
            if result != 0 {
                return token_elevation.TokenIsElevated != 0;
            }
        }
    }
    false
}

/// 执行需要管理员权限的操作，但是会导致没有在管理员权限的文件窗口无法拖动文件到此窗口
/// 打开一个具有管理员权限的新实例并关闭当前程序
pub fn request_restart_with_privileges_elevate(is_hide_window: bool, is_exit: bool) {
    // 当前用户不是管理员，尝试以管理员权限重新启动此程序
    let current_exe = env::current_exe().expect("Failed to get current executable path");
    println!(
        "current executable path is {}",
        Yellow.paint(current_exe.display().to_string())
    );
    let mut com = Command::new("powershell");
    if is_hide_window {
        com.creation_flags(HIDE_WINDOW_FLAG);
    }
    let output = com
        .arg(format!(
            "powershell -Command \"Start-Process '{}'\" -Verb RunAs",
            current_exe.display()
        ))
        .output()
        .expect("Failed to execute command");
    output.print_ln();
    let status = output.status;
    if is_exit {
        if status.success() && status.code().unwrap() == 0 {
            //确认重启到管理员
            exit(0);
        } else {
            println!("{} {}", Red.paint("用户取消授权!"), status.code().unwrap());
        }
    }
}
