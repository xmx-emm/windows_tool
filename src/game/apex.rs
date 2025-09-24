use crate::registry::steam::get_steam_path_by_registry;
use crate::steam::language::SteamLanguage;
use crate::steam::{
    get_steam_game_launch_options, get_steam_game_library_folder_by_game_id,
    set_steam_game_launch_options,
};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::path::PathBuf;

lazy_static! {
    /// apex depots的语言表
    /// 在切换语音时获取
    /// 1172471 本体
    /// https://steamdb.info/app/1172470/depots/
    pub static ref APEX_LANGUAGES_DEPOTS: HashMap<SteamLanguage,i32> = {
        let mut map: HashMap<SteamLanguage,i32> = HashMap::new();
        map.insert(SteamLanguage::FRENCH,1172472);//法语
        map.insert(SteamLanguage::GERMAN,1172473);//德语
        map.insert(SteamLanguage::ITALIAN,1172474);//意大利语
        map.insert(SteamLanguage::JAPANESE,1172475);//日语
        map.insert(SteamLanguage::KOREAN,1172476);//韩语
        map.insert(SteamLanguage::SIMPLIFIED_CHINESE ,1172477);//普通话 Mandarin
        map.insert(SteamLanguage::POLISH ,1172478);//波兰语
        map.insert(SteamLanguage::RUSSIAN ,1172479);//俄语
        map.insert(SteamLanguage::SPANISH ,1311105);//西班牙语 - 西班牙
        // map.insert(SteamLanguage::ARABIC ,1311105);//阿拉伯语 Tips:还没支持到
        map
    };
}

//获取Steam Apex 启动选项
pub fn get_apex_launch_options_by_steam_user_id(steam_user_id: usize) -> Result<String, String> {
    match get_steam_game_launch_options(steam_user_id, 1172470) {
        Ok(value) => Ok(value),
        Err(e) => Err(e),
    }
}

//设置Steam Apex 启动选项
pub fn set_apex_launch_options_by_steam_user_id<T: AsRef<str>>(
    steam_user_id: usize,
    launch_options: T,
) -> Result<(), String> {
    match set_steam_game_launch_options(steam_user_id, 1172470, launch_options) {
        Ok(value) => Ok(value),
        Err(e) => Err(e),
    }
}

// 获取Apex 配音文件夹路径audio
/*
D:\SteamLibrary\steamapps\common\Apex Legends\audio\ship
*/
pub fn get_apex_audio_folder_path() -> Option<PathBuf> {
    if let Ok(library_folder) = get_steam_game_library_folder_by_game_id(1172470) {
        let audio_folder = PathBuf::from(library_folder)
            .join("steamapps")
            .join("common")
            .join("Apex Legends")
            .join("audio")
            .join("ship")
            // .canonicalize()
            ;
        Some(audio_folder)
        // match audio_folder {
        //     Ok(value) => Some(value),
        //     Err(_) => None,
        // }
    } else {
        None
    }
}

//获取Apex 配音depot下载地址
/*
C:\Program Files (x86)\Steam\steamapps\content\app_1172470\depot_1172475\audio\ship
*/
pub fn get_apex_depot_download_folder_path(depot: usize) -> Option<PathBuf> {
    if let Some(steam_folder) = get_steam_path_by_registry() {
        let download_folder = PathBuf::from(steam_folder)
            .join("steamapps")
            .join("content")
            .join("app_1172470")
            .join(format!("depot_{}", depot))
            .join("audio")
            .join("ship");
        Some(download_folder)
        // match download_folder {
        //     Ok(value) => Some(value),
        //     Err(_) => None,
        // }
    } else {
        None
    }
}

//检查apex语音文件是否存在
/*
通过两个文件
D:\SteamLibrary\steamapps\common\Apex Legends\audio\general_japanese.mstr
D:\SteamLibrary\steamapps\common\Apex Legends\audio\general_japanese_patch_1.mstr
*/
pub fn check_apex_miles_language<T: AsRef<str>>(language: T) -> Result<bool, String> {
    if let Some(audio_folder) = get_apex_audio_folder_path() {
        let language = language.as_ref();

        let general_file_path = audio_folder
            .join(format!("general_{}.mstr", language))
            .canonicalize()
            .map_err(|e| format!("general_file_path err {}", e.to_string()))?;
        let patch_file_path = audio_folder
            .join(format!("general_{}_patch_1.mstr", language))
            .canonicalize()
            .map_err(|e| format!("patch_file_path err {}", e.to_string()))?;

        let general_file = general_file_path.exists();
        let patch_file = patch_file_path.exists();
        if !general_file {
            println!("{:?}文件不存在", general_file_path)
        }
        if !patch_file {
            println!("{:?}文件不存在", patch_file_path)
        }
        Ok(general_file && patch_file)
    } else {
        Err("SteamLibrary not found".to_string())
    }
}
