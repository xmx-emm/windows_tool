//! Steam 客户端数据访问：注册表状态、`localconfig.vdf`、`libraryfolders.vdf`、启动项与游戏语言等。
//!
//! **约定**：[`usize`] 类型的 `steam_user_id` 与 `steam_game_id` 分别对应 `userdata` 目录名与 Steam App ID。

pub mod language;
pub mod user;

use crate::registry::steam::get_steam_active_user_id_by_registry;
use crate::utils::{CommandHiddenWindowExt, Println};
use crate::utils::hex::extract_last_field;
use crate::vdf::{VdfValue, get_steam_local_config_vdf_path_by_user_id};
use std::process::Command;

// 通过注册表检测steam是否在运行
/// steam在运行时将会同步到注册表中
/// Tips: 如果强行关闭Steam那么在注册表中的数据不会被更改
pub fn steam_is_running_state_by_registry() -> bool {
    match get_steam_active_user_id_by_registry() {
        Some(user_id) => user_id != "0",
        None => false,
    }
}

/// 通过 `tasklist` 检测 Steam 进程是否存在。
pub fn steam_is_running_by_tasklist() -> Result<bool, String> {
    let output = Command::new("tasklist")
        .with_hidden_window()
        .args(["/FI", "IMAGENAME eq steam.exe", "/FO", "CSV"])
        .output()
        .map_err(|e| e.to_string())?;
    output.print_ln();
    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.contains("steam.exe"))
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
/// 从localconfig.vdf加载设置。
/// 若尚未写入过该游戏的 `LaunchOptions`（路径不存在），返回空字符串而非错误。
pub fn get_steam_game_launch_options(
    steam_user_id: usize,
    steam_game_id: usize,
) -> Result<String, String> {
    let vdf = VdfValue::load_localconfig_vdf_by_user_id(steam_user_id).map_err(|e| {
        format!(
            "读取 Steam localconfig.vdf 失败 user:{} game_id:{}: {}",
            steam_user_id, steam_game_id, e
        )
    })?;
    match vdf.get_by_path(&get_launch_options_vdf_paths_by_game_id(steam_game_id)) {
        Some(value) => match value.as_string() {
            Some(config_store) => Ok(config_store.to_string()),
            None => Err(format!(
                "Steam 启动项节点不是字符串 user:{} game_id:{}",
                steam_user_id, steam_game_id,
            )),
        },
        // 从未玩过 / 从未设置过启动项：视为空，便于首次应用创建节点。
        None => Ok(String::new()),
    }
}

//设置Steam游戏启动选项 按游戏id
pub fn set_steam_game_launch_options<T: AsRef<str>>(
    steam_user_id: usize,
    steam_game_id: usize,
    launch_options: T,
) -> Result<(), String> {
    let launch_options = launch_options.as_ref();
    let local_config_path = get_steam_local_config_vdf_path_by_user_id(steam_user_id)
        .ok_or_else(|| {
            format!(
                "未找到 Steam 安装路径或用户目录 user_id:{}（请确认已安装 Steam 并登录过该账户）",
                steam_user_id
            )
        })?;

    if !local_config_path.is_file() {
        return Err(format!(
            "未找到 Steam 用户配置文件: {}（请先在 Steam 登录该账户至少一次）",
            local_config_path.display()
        ));
    }

    let mut vdf = VdfValue::load_from_file(&local_config_path).map_err(|e| {
        format!(
            "加载 Steam localconfig.vdf 失败 user_id:{}: {}",
            steam_user_id, e
        )
    })?;

    let paths = get_launch_options_vdf_paths_by_game_id(steam_game_id);
    vdf.set_value_by_path(&paths, launch_options.to_string())
        .map_err(|e| {
            format!(
                "写入 Steam 启动项节点失败 user_id:{} game_id:{}: {}",
                steam_user_id, steam_game_id, e
            )
        })?;

    vdf.write_to_file(&local_config_path).map_err(|e| {
        format!(
            "保存 Steam localconfig.vdf 失败 user_id:{} game_id:{}: {}",
            steam_user_id, steam_game_id, e
        )
    })?;

    println!(
        "write launch_options success user_id:{} game_id:{} launch_options:\"{}\"",
        steam_user_id, steam_game_id, launch_options
    );
    Ok(())
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
                    let bytes = hex::decode(config_store).map_err(|e| {
                        format!(
                            "Steam game language hex decode failed user:{} game_id:{}: {:?}",
                            steam_user_id, steam_game_id, e
                        )
                    })?;
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
    use crate::game::steam::get_steam_game_library_folder_by_game_id;

    #[test]
    fn test_library_folder() {
        let pa = get_steam_game_library_folder_by_game_id(114);
        println!("{:?}", pa);
    }
}
