pub mod path;
pub mod unit_conversion;
pub mod string;
pub mod hex;
pub mod file;

use ansi_term::Color::{Green, Red, Yellow};
use chrono::Local;
use encoding_rs::GBK;
use std::os::windows::process::CommandExt;
use std::process::{Command, Output};

pub trait Str {
    fn is_ascii_digit(&self) -> bool;
}

pub trait Println {
    fn print_ln(&self);
    fn to_string(&self) -> String;
}

impl Println for Output {
    ///通过encoding_rs库打开控制台中文消息
    fn print_ln(&self) {
        let b = GBK.decode(&self.stdout).0;
        b.to_string().split("\r\n").for_each(|x| {
            println!("{}", Green.paint(x));
        });
        println!(
            "Exit code {}\n",
            Red.paint(self.status.code().unwrap().to_string())
        );
    }

    fn to_string(&self) -> String {
        GBK.decode(&self.stdout).0.to_string()
    }
}

impl Str for String {
    ///是ascii数字
    fn is_ascii_digit(&self) -> bool {
        for c in self.chars() {
            if !c.is_ascii_digit() {
                return false;
            }
        }
        true
    }
}

///
/// ```
/// println!("test check_ipv4_by_string");
/// use windows_tool::utils::check_ipv4_by_string;
/// assert!(check_ipv4_by_string(&"127.0.0.1".to_string()));
/// assert!(check_ipv4_by_string(&"127.0.0.15".to_string()));
/// assert!(!check_ipv4_by_string(&"asef.asef.0.1".to_string()));
/// assert!(!check_ipv4_by_string(&"12.0.0".to_string()));
/// ```
pub fn check_ipv4_by_string<T: AsRef<str>>(address: T) -> bool {
    let split = address.as_ref().split(".").collect::<Vec<&str>>();
    for x in &split {
        if x.parse::<u8>().is_err() {
            return false;
        }
    }
    split.len() == 4
}
pub static HIDE_WINDOW_FLAG: u32 = 0x08000000;

///  运行命令 管理员权限
/// /C 不保留窗口
/// /K 保留窗口
pub fn run_commands<T: AsRef<str>>(cmd: T, is_hide_window: bool, use_admin: bool) -> Output {
    let text_ref = cmd.as_ref();

    let arg = if use_admin {
        format!(
            "powershell -Command \"Start-Process cmd '/C {}'\" -Verb RunAs",
            text_ref
        )
    } else {
        text_ref.to_string()
    };

    let mut com = Command::new("powershell");
    if is_hide_window {
        com.creation_flags(HIDE_WINDOW_FLAG);
    }
    println!("Running command: {}", Yellow.paint(&arg));
    com.arg(arg)
        .output()
        .expect(format!("run_cmd_by_admin error {}", text_ref).as_str())
}

/// 运行多条命令
pub fn run_multiple_commands<S: AsRef<str>>(
    cmds: &[S],
    is_hide_window: bool,
    use_admin: bool,
) -> Output {
    let cmd_str = cmds
        .iter()
        .map(|s| s.as_ref())
        .collect::<Vec<&str>>()
        .join(" && ");
    run_commands(&cmd_str, is_hide_window, use_admin)
}

/// 获取当前本地时间并格式化
/// 2025-9-16
pub fn now_ymd() -> String {
    let format = "%Y-%m-%d";
    let now = Local::now().format(format);
    now.to_string()
}

/// 获取当前本地时间并格式化
/// 当前时间为: 2023年10月05日 14:30:45
pub fn now_ymd_hms() -> String {
    let format = "%Y年%m月%d日 %H-%M-%S";
    let now = Local::now().format(format);
    now.to_string()
}
