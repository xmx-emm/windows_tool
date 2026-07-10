//! Windows 注册表相关工具：资源管理器项备份、已知文件夹、Steam 安装路径与活动用户等。
//!
//! 需 `feature = "registry"`，且仅在 Windows 上可用。

pub mod backups;
pub mod common_folders;
pub mod steam;

use crate::utils::{Println, run_commands, RunCommandOptions};

/// 修改 Windows 更新「最长暂停天数」注册表值（`FlightSettingsMaxPauseDays`）。
pub fn modify_windows_update_flight_settings_max_pause_days(days: i32) -> Result<bool, String> {
    if !(0..=365).contains(&days) {
        return Err(format!("暂停天数超出范围(0..=365): {}", days));
    }
    let output = run_commands(
        format!(
            "reg add \"HKLM\\SOFTWARE\\Microsoft\\WindowsUpdate\\UX\\Settings\" /v FlightSettingsMaxPauseDays /t reg_dword /d {} /f",
            days
        ),
        RunCommandOptions::new(true, false, true),
    )?;
    output.print_ln();
    Ok(output.status.success())
}
