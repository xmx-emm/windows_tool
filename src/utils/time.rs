use chrono::Local;

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
