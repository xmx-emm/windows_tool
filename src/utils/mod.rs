//! 通用工具：IPv4 校验、管理员上下文执行命令、子进程输出解码（UTF-8 / GBK）、路径与时间格式等。
//!
//! 使用 `std::os::windows::process::CommandExt`，面向 Windows 主机。

pub mod ascii;
pub mod command;
pub mod filesystem;
pub mod hex;
pub mod net;
pub mod output;
pub mod string;
pub mod time;
pub mod unit_conversion;

pub use ascii::Str;
pub use command::{
    run_cmd, run_commands, run_multiple_commands, run_powershell, run_tasklist_query,
    CommandHiddenWindowExt, RunCommandOptions,
};
pub use net::check_ipv4_by_string;
pub use output::Println;
pub use time::{now_ymd, now_ymd_hms};
