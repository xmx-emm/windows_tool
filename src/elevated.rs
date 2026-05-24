use crate::utils::{CommandHiddenWindowExt, Println};
use ansi_term::Color::{Red, Yellow};
use std::env;
use std::process::{Command, exit};

/// 若当前进程以**提升的管理员令牌**（UAC 提升后）运行则返回 `true`；否则 `false`。
///
/// 参考：[TokenElevation](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ne-winnt-token_information_class)、
/// [yandexx/is_elevated](https://github.com/yandexx/is_elevated)。
///
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
            let buffer_len = size_of::<TOKEN_ELEVATION>() as u32;
            let result = GetTokenInformation(
                current_token_ptr,
                TokenElevation,
                token_elevation_type_ptr as LPVOID,
                buffer_len,
                &mut size,
            );
            if result != 0 {
                return token_elevation.TokenIsElevated != 0;
            }
        }
    }
    false
}

/// 使用 PowerShell `Start-Process -Verb RunAs` 以管理员权限**再启动一份当前可执行文件**。
///
/// **注意**：提权后的新进程可能无法接收从未提权窗口拖入的文件；若 `is_exit` 为 `true` 且用户同意 UAC，当前进程会 `exit(0)`。
pub fn request_restart_with_privileges_elevate(is_hide_window: bool, is_exit: bool) {
    // 当前用户不是管理员，尝试以管理员权限重新启动此程序
    let current_exe = env::current_exe().expect("Failed to get current executable path");
    println!(
        "current executable path is {}",
        Yellow.paint(current_exe.display().to_string())
    );
    let mut com = Command::new("powershell");
    if is_hide_window {
        com.with_hidden_window();
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
