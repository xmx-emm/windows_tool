use crate::utils::filesystem::backups_explorer_registry_path;
use crate::utils::{CommandHiddenWindowExt, Println};
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

/// PowerShell 单引号字符串内的 `'` 需加倍。
fn ps_single_quoted_segment(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/// 方法一：当前进程直接 `reg.exe EXPORT`，无 UAC 弹窗（应用已提权或仅导出可读子键时通常即可成功）。
fn backups_registry_direct(registry_key: &str, output_file: &str) -> bool {
    println!(
        "reg export [方式一: 当前进程] \"{}\" -> \"{}\"",
        registry_key, output_file
    );
    let output = match Command::new("reg.exe")
        .arg("EXPORT")
        .arg(registry_key)
        .arg(output_file)
        .arg("/y")
        .with_hidden_window()
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            println!("reg.exe 启动失败: {e}");
            return false;
        }
    };
    output.print_ln();
    output.status.success() && Path::new(output_file).is_file()
}

/// 方法二：`Start-Process reg.exe -Verb RunAs -Wait -PassThru`，等待提权后的 reg 结束并取 `ExitCode`。
/// 与旧的 `Start-Process cmd '/C ...' -Verb RunAs`（无 `-Wait`）不同，可与子进程退出码对齐。
fn backups_registry_elevated_wait(registry_key: &str, output_file: &str) -> bool {
    let arglist = format!(
        "@({}, {}, {}, '/y')",
        ps_single_quoted_segment("EXPORT"),
        ps_single_quoted_segment(registry_key),
        ps_single_quoted_segment(output_file),
    );
    let script = format!(
        "$p = Start-Process -FilePath reg.exe -Verb RunAs -ArgumentList {arglist} -Wait -PassThru; \
         if ($null -eq $p -or $null -eq $p.ExitCode) {{ exit 1 }}; \
         exit $p.ExitCode"
    );
    println!("reg export [方式二: UAC 提权] powershell -Command …");
    let output = match Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .with_hidden_window()
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            println!("powershell 启动失败: {e}");
            return false;
        }
    };
    output.print_ln();
    let ok = output.status.success() && Path::new(output_file).is_file();
    if !ok {
        println!(
            "方式二仍失败: powershell 退出码 {:?}，目标文件存在? {}",
            output.status.code(),
            Path::new(output_file).is_file()
        );
    }
    ok
}

fn backups_registry<K, F>(registry_key: K, output_file: F) -> bool
where
    K: AsRef<OsStr>,
    F: AsRef<OsStr>,
{
    let key = registry_key.as_ref().to_string_lossy();
    let out = output_file.as_ref().to_string_lossy();

    if backups_registry_direct(&key, &out) {
        return true;
    }
    println!("方式一未成功，改用方式二（将弹出 UAC，请在提示中允许）…");
    backups_registry_elevated_wait(&key, &out)
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
