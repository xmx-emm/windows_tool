pub mod backups;
pub mod common_folders;
pub mod steam;

use crate::utils::{Println, run_commands};

/// window设置最长暂停天数
pub fn modify_windows_update_flight_settings_max_pause_days(days: i32) -> bool {
    let output = run_commands(
        format!(
            "reg add \"HKLM\\SOFTWARE\\Microsoft\\WindowsUpdate\\UX\\Settings\" /v FlightSettingsMaxPauseDays /t reg_dword /d {} /f",
            days
        ),
        true,
        false,
    );
    output.print_ln();
    output.status.success()
}
