use crate::utils::path::backups_explorer_registry_path;
use crate::utils::{Println, run_commands};
use std::path::Path;

/// reg export "HKLM" C:\Users\32099\Documents\wx\test.reg /f
fn backups_registry<T: AsRef<str>>(registry_path: T, output_path: T) -> bool {
    run_commands(
        format!(
            "reg export \"{}\" \"{}\" /y",
            registry_path.as_ref(),
            output_path.as_ref()
        ),
        true,
        false,
    )
    .print_ln();
    Path::new(output_path.as_ref()).is_file()
}

/// 备份资源管理器注册表
pub fn backups_explorer_registry<T: AsRef<str>>(categorize: T) -> bool {
    match backups_explorer_registry_path(categorize, true) {
        Some(registry_path) => backups_registry(
            r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer",
            registry_path.as_str(),
        ),
        None => false,
    }
}

/// 检查是否有备份注册表
pub fn check_backups_explorer_registry<T: AsRef<str>>(categorize: T) -> bool {
    match backups_explorer_registry_path(categorize, false) {
        Some(registry_path) => {
            Path::new(registry_path.as_str()).is_file()
        }
        None => false,
    }
}
