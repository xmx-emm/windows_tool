//! Apex 配置文件路径与磁盘读写。

use super::encoding::{read_text_file, write_text_file};
use super::value::ApexCfgDocument;
use std::path::{Path, PathBuf};

/// Apex 配置文件类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApexConfigFileKind {
    /// `%USERPROFILE%/Saved Games/Respawn/Apex/local/videoconfig.txt`
    VideoConfig,
    /// `%USERPROFILE%/Saved Games/Respawn/Apex/local/settings.cfg`
    Settings,
    /// `%USERPROFILE%/Saved Games/Respawn/Apex/profile/profile.cfg`
    Profile,
}

impl ApexConfigFileKind {
    pub fn file_name(&self) -> &'static str {
        match self {
            ApexConfigFileKind::VideoConfig => "videoconfig.txt",
            ApexConfigFileKind::Settings => "settings.cfg",
            ApexConfigFileKind::Profile => "profile.cfg",
        }
    }

    pub fn subdir(&self) -> &'static str {
        match self {
            ApexConfigFileKind::VideoConfig | ApexConfigFileKind::Settings => "local",
            ApexConfigFileKind::Profile => "profile",
        }
    }
}

/// `%USERPROFILE%\Saved Games\Respawn\Apex`
pub fn get_apex_saved_games_root() -> Result<PathBuf, String> {
    let user_profile = std::env::var("USERPROFILE")
        .map_err(|_| "未设置 USERPROFILE，无法定位 Apex 配置目录".to_string())?;
    Ok(PathBuf::from(user_profile)
        .join("Saved Games")
        .join("Respawn")
        .join("Apex"))
}

pub fn get_apex_local_folder_path() -> Result<PathBuf, String> {
    Ok(get_apex_saved_games_root()?.join("local"))
}

pub fn get_apex_profile_folder_path() -> Result<PathBuf, String> {
    Ok(get_apex_saved_games_root()?.join("profile"))
}

pub fn get_apex_config_path(kind: ApexConfigFileKind) -> Result<PathBuf, String> {
    Ok(get_apex_saved_games_root()?
        .join(kind.subdir())
        .join(kind.file_name()))
}

/// 查询 `videoconfig.txt` 是否为只读。
pub fn is_videoconfig_readonly() -> Result<bool, String> {
    let path = get_apex_config_path(ApexConfigFileKind::VideoConfig)?;
    if !path.exists() {
        return Ok(false);
    }
    let meta = std::fs::metadata(&path)
        .map_err(|e| format!("读取文件属性失败 {}: {e}", path.display()))?;
    Ok(meta.permissions().readonly())
}

/// 设置/取消 `videoconfig.txt` 只读属性。
pub fn set_videoconfig_readonly(locked: bool) -> Result<(), String> {
    let path = get_apex_config_path(ApexConfigFileKind::VideoConfig)?;
    if !path.exists() {
        return Err(format!("未找到画面配置文件: {}", path.display()));
    }
    set_path_readonly(&path, locked)
}

fn set_path_readonly(path: &Path, locked: bool) -> Result<(), String> {
    let meta = std::fs::metadata(path)
        .map_err(|e| format!("读取文件属性失败 {}: {e}", path.display()))?;
    let mut perms = meta.permissions();
    if perms.readonly() == locked {
        return Ok(());
    }
    perms.set_readonly(locked);
    std::fs::set_permissions(path, perms)
        .map_err(|e| format!("设置只读属性失败 {}: {e}", path.display()))
}

impl ApexCfgDocument {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path = path.as_ref();
        let (content, encoding) = read_text_file(path)?;
        let mut doc = Self::from_content(&content, encoding)?;
        doc.encoding = encoding;
        Ok(doc)
    }

    pub fn load(kind: ApexConfigFileKind) -> Result<Self, String> {
        let path = get_apex_config_path(kind)?;
        if !path.exists() {
            return Err(format!("未找到配置文件: {}", path.display()));
        }
        Self::load_from_file(path)
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败 {}: {e}", parent.display()))?;
        }
        let content = self.to_string();
        write_text_file(path, &content, self.encoding)
    }

    pub fn save(&self, kind: ApexConfigFileKind) -> Result<(), String> {
        let path = get_apex_config_path(kind)?;
        if path.exists() {
            let backup = path.with_extension(match kind {
                ApexConfigFileKind::VideoConfig => "txt.bak",
                _ => "cfg.bak",
            });
            let _ = std::fs::copy(&path, &backup);
        }
        self.write_to_file(&path)
    }

    pub fn save_with_backup(&self, kind: ApexConfigFileKind) -> Result<PathBuf, String> {
        self.save(kind)?;
        get_apex_config_path(kind)
    }
}

/// 读取 videoconfig 键值映射（兼容旧 API）。
pub fn read_apex_videoconfig_map() -> Result<std::collections::HashMap<String, String>, String> {
    let doc = ApexCfgDocument::load(ApexConfigFileKind::VideoConfig)?;
    Ok(doc.key_values().into_iter().collect())
}

/// 批量更新 videoconfig 中的键。
pub fn patch_apex_videoconfig(
    updates: &std::collections::HashMap<String, String>,
) -> Result<String, String> {
    if updates.is_empty() {
        return Err("没有需要写入的画质配置项".to_string());
    }
    let path = get_apex_config_path(ApexConfigFileKind::VideoConfig)?;

    // 若文件被设为只读，先临时解锁，写入后再恢复（防止 Apex 启动还原配置）。
    let was_readonly = path.exists()
        && std::fs::metadata(&path)
            .map(|m| m.permissions().readonly())
            .unwrap_or(false);
    if was_readonly {
        set_path_readonly(&path, false)?;
    }

    let result = (|| -> Result<String, String> {
        let mut doc = if path.exists() {
            ApexCfgDocument::load_from_file(&path)?
        } else {
            let mut doc = ApexCfgDocument::new();
            doc.root_name = Some("VideoConfig".to_string());
            doc.lines.push(super::value::ApexCfgLine::Raw("\"VideoConfig\"".to_string()));
            doc.lines.push(super::value::ApexCfgLine::Raw("{".to_string()));
            doc.lines.push(super::value::ApexCfgLine::Raw("}".to_string()));
            doc
        };

        for (key, value) in updates {
            if !key.starts_with("setting.") && !key.starts_with('"') {
                return Err(format!("非法配置键: {key}"));
            }
            let normalized = key.trim_matches('"');
            doc.set(normalized, value)?;
        }
        doc.save(ApexConfigFileKind::VideoConfig)?;
        Ok(doc.to_string())
    })();

    if was_readonly {
        // 无论写入成功与否，都恢复只读属性。
        let _ = set_path_readonly(&path, true);
    }

    result
}

pub fn ensure_apex_local_folder() -> Result<PathBuf, String> {
    let dir = get_apex_local_folder_path()?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败 {}: {e}", dir.display()))?;
    Ok(dir)
}
