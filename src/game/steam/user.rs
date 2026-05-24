//! 枚举本机 Steam 用户（`userdata` 下含 `localconfig.vdf` 的目录），并从 VDF 构造 [`SteamUser`]。

use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::registry::steam::get_steam_path_by_registry;
use crate::utils::string::StringExt;
use crate::vdf::VdfValue;

#[derive(Serialize, Deserialize, Debug)]
pub struct SteamUser {
    name: String,
    id: String,
    avatar: String,
    config_path: String,
}

impl SteamUser {
    pub fn from_vdf<T: AsRef<str>>(
        vdf: &VdfValue,
        user_id: &T,
        config_path: &T,
    ) -> Result<SteamUser, String> {
        let name = vdf.get_value_by_path(&vec!["UserLocalConfigStore", "friends", "PersonaName"]);
        let avatar = vdf.get_value_by_path(&vec![
            "UserLocalConfigStore",
            "friends",
            user_id.as_ref(),
            "avatar",
        ]);
        if name.is_none() {
            Err(format!(
                "UserLocalConfigStore name not found {}",
                user_id.as_ref()
            ))
        } else {
            let avatar = if avatar.is_none() {
                "fef49e7fa7e1997310d705b2a6158ff8dc1cdfeb".into() //问号头像
            } else {
                avatar.unwrap().to_string()
            };
            Ok(SteamUser {
                name: name.unwrap().to_string(),
                id: user_id.as_ref().parse().unwrap(),
                avatar,
                config_path: config_path.as_ref().to_string(),
            })
        }
    }
}

/// 通过遍历已安装的Steam 用户数据获取登录用户的id
/// C:\Program Files (x86)\Steam\userdata\{user id}\config
pub fn get_steam_users_id() -> Result<Vec<String>, String> {
    match get_steam_path_by_registry() {
        Some(steam_path) => {
            let userdata = &Path::new(&steam_path).join("userdata");
            match userdata.read_dir() {
                Ok(dir) => {
                    let mut users = Vec::new();
                    for x in dir {
                        if let Ok(dir) = x
                            && let Ok(ft) = dir.file_type()
                            && ft.is_dir()
                        {
                            let user_id = dir.file_name().to_str().unwrap().to_owned();
                            let config_file = userdata
                                .join(&user_id)
                                .join("config")
                                .join("localconfig.vdf");
                            if user_id.is_valid_integer(false) && config_file.is_file() {
                                users.push(user_id);
                            }
                        }
                    }
                    Ok(users)
                }
                Err(err) => Err(format!("Steam userdata path doesn't exist {}", err)),
            }
        }
        None => Err("Steam path doesn't exist".to_string()),
    }
}

/// 获取所有的Steam用户数据
pub fn get_steam_users() -> Result<Vec<SteamUser>, String> {
    match get_steam_users_id() {
        Ok(steam_users) => {
            let mut users = Vec::new();
            let steam_path = get_steam_path_by_registry().unwrap(); //get_steam已经验证过一次了
            for user in steam_users {
                let local_config_path = &Path::new(&steam_path)
                    .join("userdata")
                    .join(&user)
                    .join("config")
                    .join("localconfig.vdf");
                match VdfValue::load_from_file(local_config_path) {
                    Ok(vdf) => {
                        match SteamUser::from_vdf(
                            &vdf,
                            &user,
                            &local_config_path.to_str().unwrap().to_string(),
                        ) {
                            Ok(i) => {
                                users.push(i);
                            }
                            Err(e) => {
                                println!("Steam users not found: {}", user);
                                println!("\t{}", e)
                            }
                        }
                    }
                    Err(e) => {
                        println!("Steam userdata path doesn't exist {}", user);
                        println!("\t{}", e)
                    }
                }
            }
            Ok(users)
        }
        Err(e) => Err(e),
    }
}
