use std::fs;
use std::io::Write;
use std::path::Path;

/// 若路径存在且为只读，临时取消只读；返回原先是否只读。
pub fn clear_readonly_if_needed(path: &Path) -> Result<bool, String> {
    if !path.exists() {
        return Ok(false);
    }
    let meta = fs::metadata(path)
        .map_err(|e| format!("读取文件属性失败 {}: {e}", path.display()))?;
    let mut perms = meta.permissions();
    if !perms.readonly() {
        return Ok(false);
    }
    perms.set_readonly(false);
    fs::set_permissions(path, perms)
        .map_err(|e| format!("取消只读属性失败 {}: {e}", path.display()))?;
    Ok(true)
}

pub fn set_path_readonly(path: &Path, locked: bool) -> Result<(), String> {
    let meta = fs::metadata(path)
        .map_err(|e| format!("读取文件属性失败 {}: {e}", path.display()))?;
    let mut perms = meta.permissions();
    if perms.readonly() == locked {
        return Ok(());
    }
    perms.set_readonly(locked);
    fs::set_permissions(path, perms)
        .map_err(|e| format!("设置只读属性失败 {}: {e}", path.display()))
}

/// 安全写入文本：先写同目录临时文件，再 copy 覆盖目标（失败时尽量保留原文件），并处理只读属性。
pub fn write_text_file_atomic(path: &Path, content: &str) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| {
        format!("无效文件路径（无父目录）: {}", path.display())
    })?;
    if !parent.is_dir() {
        return Err(format!("配置目录不存在: {}", parent.display()));
    }

    let was_readonly = clear_readonly_if_needed(path)?;

    let tmp_name = format!(
        "{}.{}.tmp",
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("write"),
        std::process::id()
    );
    let tmp_path = parent.join(tmp_name);

    let write_result = (|| -> Result<(), String> {
        {
            let mut file = fs::File::create(&tmp_path).map_err(|e| {
                format!(
                    "创建临时文件失败 {}: {e}（请检查磁盘权限或杀毒软件拦截）",
                    tmp_path.display()
                )
            })?;
            file.write_all(content.as_bytes()).map_err(|e| {
                format!("写入临时文件失败 {}: {e}", tmp_path.display())
            })?;
            file.sync_all().map_err(|e| {
                format!("同步临时文件失败 {}: {e}", tmp_path.display())
            })?;
        }

        // 用 copy 覆盖目标，避免先删原文件导致失败时丢配置
        fs::copy(&tmp_path, path).map_err(|e| {
            format!(
                "写入配置文件失败 {}: {e}（文件可能被占用，请完全退出 Steam/EA 后重试）",
                path.display()
            )
        })?;
        let _ = fs::remove_file(&tmp_path);
        Ok(())
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }

    if was_readonly && path.exists() {
        let _ = set_path_readonly(path, true);
    }

    write_result
}
