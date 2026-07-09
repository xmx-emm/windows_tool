use crate::registry::steam::get_steam_path_by_registry;
use crate::utils::filesystem::write_text_file_atomic;
use crate::vdf::{VdfValue, parse_vdf_string};
use std::fs;
use std::path::{Path, PathBuf};

impl VdfValue {
    /// 从文件加载vdf数据
    pub fn load_from_file<P: AsRef<Path>>(file_path: P) -> Result<VdfValue, String> {
        let path = file_path.as_ref();
        if !path.is_file() {
            return Err(format!(
                "VDF 文件不存在: {}（请确认 Steam 用户目录完整）",
                path.display()
            ));
        }
        match fs::read_to_string(path) {
            Ok(content) => match parse_vdf_string(&content) {
                Ok(vdf) => Ok(vdf),
                Err(e) => Err(format!(
                    "解析 VDF 失败 {}: {}",
                    path.display(),
                    e
                )),
            },
            Err(e) => Err(format!("读取 VDF 失败 {}: {}", path.display(), e)),
        }
    }

    /// 写入数据到文件（原子替换，并处理只读属性）
    pub fn write_to_file<P: AsRef<Path>>(&self, file_path: P) -> Result<(), String> {
        write_text_file_atomic(file_path.as_ref(), &self.to_string())
    }

    // 输入用户id从文件获取localconfig vdf数据
    pub fn load_localconfig_vdf_by_user_id(steam_user_id: usize) -> Result<VdfValue, String> {
        match get_steam_local_config_vdf_path_by_user_id(steam_user_id) {
            Some(local_config_path) => match VdfValue::load_from_file(local_config_path) {
                Ok(vdf) => Ok(vdf),
                Err(e) => Err(e),
            },
            None => Err(format!(
                "Not find steam local config vdf path {}",
                steam_user_id
            )),
        }
    }
    // 获取steam库文件夹的路径
    pub fn load_libraryfolders_vdf() -> Result<VdfValue, String> {
        match get_steam_libraryfolders_config_path_by_registry() {
            Some(libraryfolders_config_path) => {
                match VdfValue::load_from_file(libraryfolders_config_path) {
                    Ok(vdf) => Ok(vdf),
                    Err(e) => Err(e),
                }
            }
            None => Err(format!("Could not find steam libraryfolders config path")),
        }
    }
}

///获取Steam localconfig.vdf路径 按steam的用户id
pub fn get_steam_local_config_vdf_path_by_user_id(steam_user_id: usize) -> Option<PathBuf> {
    match get_steam_path_by_registry() {
        Some(steam_path) => {
            let path_buf = Path::new(&steam_path)
                .join("userdata")
                .join(steam_user_id.to_string())
                .join("config")
                .join("localconfig.vdf");
            Some(path_buf)
        }
        None => {
            print!(
                "get_steam_game_launch_options error {}\n\tSteam path doesn't exist",
                steam_user_id
            );
            None
        }
    }
}

/// 通过注册表获取Steam当前用户的局部配置
/// Steam\steamapps\libraryfolders.vdf
pub fn get_steam_libraryfolders_config_path_by_registry() -> Option<String> {
    match get_steam_path_by_registry() {
        Some(steam_path) => {
            let path = Path::new(&steam_path)
                .join("steamapps")
                .join("libraryfolders.vdf");
            if path.exists() {
                Some(path.as_path().to_str().unwrap().to_string())
            } else {
                None
            }
        }
        None => None,
    }
}
