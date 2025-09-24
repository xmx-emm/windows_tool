use crate::registry::steam::get_steam_path_by_registry;
use crate::vdf::{VdfValue, parse_vdf_string};
use std::fs;
use std::path::{Path, PathBuf};

impl VdfValue {
    /// 从文件加载vdf数据
    pub fn load_from_file<P: AsRef<Path>>(file_path: P) -> Result<VdfValue, String> {
        match fs::read_to_string(&file_path.as_ref()) {
            Ok(content) => match parse_vdf_string(&content) {
                Ok(vdf) => Ok(vdf),
                Err(e) => Err(format!(
                    "Failed to parse VDF file {} \n\t{}",
                    file_path.as_ref().to_str().unwrap(),
                    e
                )),
            },
            Err(e) => Err(format!("Could not read from file: {}", e)),
        }
    }

    /// 写入数据到文件
    pub fn write_to_file<P: AsRef<Path>>(&self, file_path: P) -> Result<(), String> {
        let path = file_path.as_ref();
        match fs::write(path, self.to_string().as_bytes()) {
            Ok(_) => match fs::exists(path) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!(
                    "Cannot write to file {} {}",
                    path.to_str().unwrap(),
                    e
                )),
            },
            Err(e) => Err(format!("write_to_file error {:?} {}", path, e)),
        }
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
