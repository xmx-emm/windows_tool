use std::path::Path;
use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

/// 通过注册表获取Steam路径
/// reg query "HKEY_CURRENT_USER\Software\Valve\Steam" /v "SteamPath"
/// c:/program files (x86)/steam
pub fn get_steam_path_by_registry() -> Option<String> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;
    let subkey = RegKey::predef(HKEY_CURRENT_USER).open_subkey(r"Software\Valve\Steam");
    match subkey {
        Ok(subkey) => match subkey.get_value("SteamPath") {
            Ok(val) => Some(val),
            Err(e) => {
                println!("SteamPath get value error {}", e);
                None
            }
        },
        Err(e) => {
            println!("SteamPath open_subkey error {}", e);
            None
        }
    }
}

/// 通过注册表获取Steam当前用户的id
/// Tips: 只有在运行Steam时有效，退出时Steam 将会把此项修改为0
///计算机\HKEY_CURRENT_USER\SOFTWARE\Valve\Steam\ActiveProcess ActiveUser 16进制
pub fn get_steam_active_user_id_by_registry() -> Option<String> {
    let subkey = RegKey::predef(HKEY_CURRENT_USER);
    let sub_key = subkey.open_subkey(r"SOFTWARE\Valve\Steam\ActiveProcess");
    match sub_key {
        Ok(reg_key) => match reg_key.get_raw_value("ActiveUser") {
            Ok(val) => Some(val.to_string()),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

/// 通过注册表获取Steam当前用户的局部配置
/// C:\Program Files (x86)\Steam\userdata\{ActiveUser ID}\config\localconfig.vdf
pub fn get_active_user_steam_local_config_path_by_registry() -> Option<String> {
    match get_steam_path_by_registry() {
        Some(steam_path) => match get_steam_active_user_id_by_registry() {
            Some(active_user_id) => {
                let path = Path::new(&steam_path)
                    .join("userdata")
                    .join(active_user_id)
                    .join("config")
                    .join("localconfig.vdf");
                if path.exists() {
                    Some(path.as_path().to_str().unwrap().to_string())
                } else {
                    None
                }
            }
            None => None,
        },
        None => None,
    }
}
