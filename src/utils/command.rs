use ansi_term::Color::Yellow;
use std::os::windows::process::CommandExt;
use std::process::{Command, Output};

const HIDE_WINDOW_FLAG: u32 = 0x08000000;

pub trait CommandHiddenWindowExt {
    fn with_hidden_window(&mut self) -> &mut Self;
}

impl CommandHiddenWindowExt for Command {
    fn with_hidden_window(&mut self) -> &mut Self {
        self.creation_flags(HIDE_WINDOW_FLAG);
        self
    }
}

/// 控制 [`run_commands`] / [`run_multiple_commands`] 的执行与日志行为。
#[derive(Clone, Copy, Debug)]
pub struct RunCommandOptions {
    pub hide_window: bool,
    pub use_admin: bool,
    /// 为 `true` 时在执行前向 stdout 打印完整命令字符串。
    pub print_command: bool,
}

impl RunCommandOptions {
    pub const fn new(hide_window: bool, use_admin: bool, print_command: bool) -> Self {
        Self {
            hide_window,
            use_admin,
            print_command,
        }
    }

    /// `tasklist` 等进程探测：隐藏窗口、不提权、**不打印**命令（避免轮询刷屏）。
    pub const fn tasklist() -> Self {
        Self::new(true, false, false)
    }
}

/// 运行命令：非提权时使用 `cmd /C`，避免经 PowerShell 转发导致 `netsh` 等本地化输出被错误转码（打印/日志乱码）。
/// 提权时仍通过 PowerShell `Start-Process -Verb RunAs` 拉起 `cmd /C …`。
/// /C 执行后退出；/K 保留窗口（本函数未使用）。
pub fn run_commands<T: AsRef<str>>(cmd: T, opts: RunCommandOptions) -> Output {
    let text_ref = cmd.as_ref();

    if opts.use_admin {
        let arg = format!(
            "powershell -Command \"Start-Process cmd '/C {}'\" -Verb RunAs",
            text_ref
        );
        let mut com = Command::new("powershell");
        if opts.hide_window {
            com.with_hidden_window();
        }
        if opts.print_command {
            println!("Running command: {}", Yellow.paint(&arg));
        }
        com.arg(arg)
            .output()
            .unwrap_or_else(|_| panic!("run_cmd_by_admin error {}", text_ref))
    } else {
        let mut com = Command::new("cmd.exe");
        if opts.hide_window {
            com.with_hidden_window();
        }
        if opts.print_command {
            println!("Running command: {}", Yellow.paint(text_ref));
        }
        com.args(["/C", text_ref])
            .output()
            .unwrap_or_else(|_| panic!("run_commands error {}", text_ref))
    }
}

/// 运行 `tasklist` 等进程探测命令：等价于 [`run_commands`] + [`RunCommandOptions::tasklist`]。
pub fn run_tasklist_query<T: AsRef<str>>(cmd: T) -> Output {
    run_commands(cmd, RunCommandOptions::tasklist())
}

/// 运行多条命令
pub fn run_multiple_commands<S: AsRef<str>>(cmds: &[S], opts: RunCommandOptions) -> Output {
    let cmd_str = cmds
        .iter()
        .map(|s| s.as_ref())
        .collect::<Vec<&str>>()
        .join(" && ");
    run_commands(&cmd_str, opts)
}

pub fn run_cmd(args: &[&str]) -> Result<String, String> {
    let output = Command::new("cmd")
        .with_hidden_window()
        .args(["/C"])
        .args(args)
        .output()
        .map_err(|e| format!("执行命令失败: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Err(if stderr.is_empty() { stdout } else { stderr })
    }
}

pub fn run_powershell(script: &str) -> Result<String, String> {
    let output = Command::new("powershell")
        .with_hidden_window()
        .args(["-NoProfile", "-Command", script])
        .output()
        .map_err(|e| format!("执行PowerShell失败: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(stderr)
    }
}
