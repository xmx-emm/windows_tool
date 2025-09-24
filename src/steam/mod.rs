pub mod language;
pub mod user;

use crate::registry::steam::get_steam_active_user_id_by_registry;
use crate::utils::hex::extract_last_field;
use crate::vdf::{VdfValue, get_steam_local_config_vdf_path_by_user_id};

// 通过注册表检测steam是否在运行
/// steam在运行时将会同步到注册表中
/// Tips: 如果强行关闭Steam那么在注册表中的数据不会被更改
pub fn steam_is_running_state_by_registry() -> bool {
    match get_steam_active_user_id_by_registry() {
        Some(user_id) => user_id != "0",
        None => false,
    }
}

// 获取启动选项路径按游戏id
/// UserLocalConfigStore -> Software -> Valve -> Steam -> apps -> steam_game_id -> LaunchOptions
pub fn get_launch_options_vdf_paths_by_game_id(steam_game_id: usize) -> Vec<String> {
    let paths = vec![
        "UserLocalConfigStore".to_string(),
        "Software".to_string(),
        "Valve".to_string(),
        "Steam".to_string(),
        "apps".to_string(),
        steam_game_id.to_string(),
        "LaunchOptions".to_string(),
    ];
    paths
}

// 获取userappconfi路径按游戏id
/// UserLocalConfigStore -> UserAppConfig -> steam_game_id
pub fn get_userappconfig_vdf_paths_by_game_id(steam_game_id: usize) -> Vec<String> {
    let paths = vec![
        "UserLocalConfigStore".to_string(),
        "UserAppConfig".to_string(),
        steam_game_id.to_string(),
    ];
    paths
}

// 获取Steam游戏启动选项 按游戏id和steam用户id
/// 从localconfig.vdf加载设置
pub fn get_steam_game_launch_options(
    steam_user_id: usize,
    steam_game_id: usize,
) -> Result<String, String> {
    match VdfValue::load_localconfig_vdf_by_user_id(steam_user_id) {
        Ok(vdf) => match vdf.get_by_path(&get_launch_options_vdf_paths_by_game_id(steam_game_id)) {
            Some(value) => match value.as_string() {
                Some(config_store) => Ok(config_store.to_string()),
                None => Err(format!(
                    "Steam game launch options not a string user:{} \tgame_id:{}",
                    steam_user_id, steam_game_id,
                )),
            },
            None => Err(format!(
                "launch_options Steam game config store get value user:{} \tgame_id:{} ",
                steam_user_id, steam_game_id,
            )),
        },
        Err(e) => Err(format!("Steam game launch options doesn't exist {}", e)),
    }
}

//设置Steam游戏启动选项 按游戏id
pub fn set_steam_game_launch_options<T: AsRef<str>>(
    steam_user_id: usize,
    steam_game_id: usize,
    launch_options: T,
) -> Result<(), String> {
    match get_steam_local_config_vdf_path_by_user_id(steam_user_id) {
        Some(local_config_path) => match VdfValue::load_from_file(&local_config_path) {
            Ok(mut vdf) => {
                let paths = get_launch_options_vdf_paths_by_game_id(steam_game_id);
                match vdf.set_value_by_path(&paths, launch_options.as_ref().to_string()) {
                    Ok(()) => match vdf.write_to_file(local_config_path) {
                        Ok(_) => Ok(()),
                        Err(_) => Err(format!(
                            "Write game launch options failed user_id:{} game_id:{}",
                            steam_user_id, steam_game_id
                        )),
                    },
                    Err(e) => Err(format!(
                        "set_value_by_path Steam game launch options doesn't exist {}",
                        e
                    )),
                }
            }
            Err(e) => Err(format!(
                "set_steam_game_launch_options Steam game launch options doesn't exist {}",
                e
            )),
        },
        None => Err(format!(
            "set_steam_game_launch_options user_id:{} not find",
            steam_user_id
        )),
    }
}

//获取Steam游戏语言 按游戏id,从localconfig.vdf里面
/*
https://partner.steamgames.com/doc/store/localization/languages
*/
pub fn get_steam_game_language(
    steam_user_id: usize,
    steam_game_id: usize,
) -> Result<String, String> {
    match VdfValue::load_localconfig_vdf_by_user_id(steam_user_id) {
        Ok(vdf) => match vdf.get_by_path(&get_userappconfig_vdf_paths_by_game_id(steam_game_id)) {
            Some(value) => match value.as_string() {
                Some(config_store) => {
                    let bytes = hex::decode(&config_store).unwrap();
                    match extract_last_field(&bytes) {
                        Some(last_field) => Ok(last_field.to_string()),
                        None => Err("extract_last_field not find".to_string()),
                    }
                }
                None => Err(format!(
                    "Steam game language not a string user:{} \tgame_id:{}",
                    steam_user_id, steam_game_id,
                )),
            },
            None => Err(format!(
                "language Steam game config store get value user:{} \tgame_id:{}",
                steam_user_id, steam_game_id,
            )),
        },
        Err(e) => Err(format!("Steam game language doesn't exist {}", e)),
    }
}

//获取Steam游戏库路径 按游戏id
/*
Steam\steamapps\libraryfolders.vdf
"libraryfolders"
{
    "0"
    {
        "path"		"D:\\SteamLibrary"
        "label"		"这是一个库"
        "contentid"		"8184390354682506976"
        "totalsize"		"2048406843392"
        "update_clean_bytes_tally"		"2149010076"
        "time_last_update_verified"		"1765757561"
        "apps"
        {
            "365670"		"956090679"
            "578080"		"46739942167"
            "678960"		"36958397200"
            "1172470"		"97031306833"
        }
    }
}
 */
pub fn get_steam_game_library_folder_by_game_id(steam_game_id: usize) -> Result<String, String> {
    let steam_game_string = steam_game_id.to_string();
    match VdfValue::load_libraryfolders_vdf() {
        Ok(vdf) => match vdf.get_by_path(&vec!["libraryfolders"]) {
            Some(library_folders) => {
                match library_folders.as_object() {
                    Some(library) => {
                        //这里拿到单个库的内容
                        for (_, value) in library {
                            match value.get("apps") {
                                //app列表
                                Some(apps) => {
                                    match apps.as_object() {
                                        Some(apps) => {
                                            // 游戏id : 游戏大小
                                            for (app_id, _) in apps {
                                                //匹配游戏id
                                                if *app_id == steam_game_string {
                                                    return if let Some(path_value) =
                                                        value.get_value("path")
                                                    {
                                                        Ok(path_value.to_string())
                                                    } else {
                                                        Err("Steam game libraryfolders.vdf path 'libraryfolders' doesn't exist".to_string())
                                                    };
                                                }
                                            }
                                        }
                                        None => {
                                            println!("apps 信息错拉!!??\t{:?}", apps)
                                        }
                                    }
                                }
                                None => {}
                            }
                        }
                        Err(format!("folders not find {}", steam_game_id))
                    }
                    None => Err(
                        "Steam game libraryfolders.vdf path 'libraryfolders' doesn't exist"
                            .to_string(),
                    ),
                }
            }
            None => Err("Steam game libraryfolders.vdf doesn't exist".to_string()),
        },
        Err(e) => Err(format!("Steam game library folder doesn't exist {}", e)),
    }
}

#[cfg(test)]
mod tests_steam {
    use crate::steam::get_steam_game_library_folder_by_game_id;

    #[test]
    fn test_library_folder() {
        let pa = get_steam_game_library_folder_by_game_id(114);
        println!("{:?}", pa);
    }
}
