//! Apex Legends（Steam App ID `1172470`）启动项、语音资源路径与 depot 目录等。

use crate::registry::steam::get_steam_path_by_registry;
use crate::game::steam::language::SteamLanguage;
use crate::game::steam::{
    get_steam_game_launch_options, get_steam_game_library_folder_by_game_id,
    set_steam_game_launch_options,
};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;
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

fn get_apex_audio_folder_path_steam() -> Option<PathBuf> {
    get_steam_game_library_folder_by_game_id(1172470)
        .ok()
        .map(|library_folder| {
            PathBuf::from(library_folder)
                .join("steamapps")
                .join("common")
                .join("Apex Legends")
                .join("audio")
                .join("ship")
        })
}

fn get_ea_user_download_in_place_dir(ea_user_id: &str) -> Option<PathBuf> {
    if !ea_user_id.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let local_app_data = std::env::var("LOCALAPPDATA").ok()?;
    let ini_path = PathBuf::from(local_app_data)
        .join("Electronic Arts")
        .join("EA Desktop")
        .join(format!("user_{}.ini", ea_user_id));
    let content = fs::read_to_string(ini_path).ok()?;
    for line in content.lines() {
        let trim_line = line.trim();
        if trim_line.is_empty() || trim_line.starts_with('#') || trim_line.starts_with(';') {
            continue;
        }
        let Some((key, value)) = trim_line.split_once('=') else {
            continue;
        };
        if key.trim().eq_ignore_ascii_case("user.downloadinplacedir") {
            let dir = value.trim().trim_matches('"');
            if !dir.is_empty() {
                return Some(PathBuf::from(dir));
            }
        }
    }
    None
}

fn get_apex_audio_folder_path_ea(ea_user_id: &str) -> Option<PathBuf> {
    get_ea_user_download_in_place_dir(ea_user_id).map(|base| {
        base.join("Apex")
            .join("audio")
            .join("ship")
    })
}

// 获取Apex 配音文件夹路径audio
/*
Steam: D:\SteamLibrary\steamapps\common\Apex Legends\audio\ship
EA: D:\EA\Apex\audio\ship
*/
pub fn get_apex_audio_folder_path() -> Option<PathBuf> {
    get_apex_audio_folder_path_steam()
}

pub fn get_apex_audio_folder_path_by_platform(
    platform: Option<&str>,
    ea_user_id: Option<&str>,
) -> Option<PathBuf> {
    match platform.unwrap_or("steam").to_ascii_lowercase().as_str() {
        "ea" => ea_user_id.and_then(get_apex_audio_folder_path_ea),
        _ => get_apex_audio_folder_path_steam(),
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

pub fn get_apex_download_folder_path_by_platform(
    depot: usize,
    platform: Option<&str>,
    ea_user_id: Option<&str>,
) -> Option<PathBuf> {
    match platform.unwrap_or("steam").to_ascii_lowercase().as_str() {
        // EA 下语音包直接落在安装目录的 Apex/audio/ship
        "ea" => ea_user_id.and_then(get_apex_audio_folder_path_ea),
        _ => get_apex_depot_download_folder_path(depot),
    }
}

/** 检查apex语音文件是否存在
通过两个文件
D:\SteamLibrary\steamapps\common\Apex Legends\audio\general_japanese.mstr
D:\SteamLibrary\steamapps\common\Apex Legends\audio\general_japanese_patch_1.mstr
*/
pub fn check_apex_miles_language<T: AsRef<str>>(language: T) -> Result<bool, String> {
    check_apex_miles_language_by_platform(language, None, None)
}

pub fn check_apex_miles_language_by_platform<T: AsRef<str>>(
    language: T,
    platform: Option<&str>,
    ea_user_id: Option<&str>,
) -> Result<bool, String> {
    let audio_folder = get_apex_audio_folder_path_by_platform(platform, ea_user_id)
        .ok_or_else(|| format!("Apex 音频目录未找到 platform={:?}", platform))?;
    let language = language.as_ref();

    let general_file_path = audio_folder.join(format!("general_{}.mstr", language));
    let patch_file_path = audio_folder.join(format!("general_{}_patch_1.mstr", language));

    let general_file = general_file_path.exists();
    let patch_file = patch_file_path.exists();
    if !general_file {
        println!("{:?}文件不存在", general_file_path)
    }
    if !patch_file {
        println!("{:?}文件不存在", patch_file_path)
    }
    Ok(general_file && patch_file)
}

/// `%USERPROFILE%\\Saved Games\\Respawn\\Apex\\local`
pub fn get_apex_local_folder_path() -> Result<PathBuf, String> {
    let user_profile = std::env::var("USERPROFILE")
        .map_err(|_| "未设置 USERPROFILE，无法定位 Apex local 配置目录".to_string())?;
    Ok(PathBuf::from(user_profile)
        .join("Saved Games")
        .join("Respawn")
        .join("Apex")
        .join("local"))
}

/// `.../local/videoconfig.txt`
pub fn get_apex_videoconfig_path() -> Result<PathBuf, String> {
    Ok(get_apex_local_folder_path()?.join("videoconfig.txt"))
}

fn parse_videoconfig_key_values(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('"') {
            continue;
        }
        let Some((key_part, rest)) = trimmed.split_once('\t') else {
            continue;
        };
        let key = key_part.trim().trim_matches('"');
        if !key.starts_with("setting.") {
            continue;
        }
        let value = rest
            .trim()
            .trim_matches('"')
            .trim_end_matches(',')
            .to_string();
        if !value.is_empty() {
            map.insert(key.to_string(), value);
        }
    }
    map
}

/// 读取 `videoconfig.txt` 全文与 `setting.*` 键值。
pub fn read_apex_videoconfig() -> Result<(String, HashMap<String, String>), String> {
    let path = get_apex_videoconfig_path()?;
    if !path.exists() {
        return Err(format!("未找到画质配置文件: {}", path.display()));
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取画质配置失败 {}: {e}", path.display()))?;
    let parsed = parse_videoconfig_key_values(&content);
    Ok((content, parsed))
}

fn patch_videoconfig_line(content: &str, key: &str, value: &str) -> String {
    let pattern = format!(r#""{key}""#);
    let replacement_line = format!("\t\"{key}\"\t\t\"{value}\"");
    let mut replaced = false;
    let mut out = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&pattern) {
            out.push_str(&replacement_line);
            replaced = true;
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    if !replaced {
        if let Some(pos) = out.rfind('}') {
            out.insert_str(pos, &format!("{replacement_line}\n"));
        } else {
            out.push_str(&replacement_line);
            out.push('\n');
        }
    }
    out
}

/// 仅更新指定 `setting.*` 键，保留文件中其它内容。
pub fn patch_apex_videoconfig(updates: &HashMap<String, String>) -> Result<String, String> {
    if updates.is_empty() {
        return Err("没有需要写入的画质配置项".to_string());
    }
    let path = get_apex_videoconfig_path()?;
    let local_dir = path
        .parent()
        .ok_or_else(|| "无效的画质配置路径".to_string())?;
    fs::create_dir_all(local_dir)
        .map_err(|e| format!("创建目录失败 {}: {e}", local_dir.display()))?;

    let mut content = if path.exists() {
        fs::read_to_string(&path)
            .map_err(|e| format!("读取画质配置失败 {}: {e}", path.display()))?
    } else {
        concat!(
            "\"VideoConfig\"\n",
            "{\n",
            "}\n"
        )
        .to_string()
    };

    for (key, value) in updates {
        if !key.starts_with("setting.") {
            return Err(format!("非法配置键: {key}"));
        }
        content = patch_videoconfig_line(&content, key, value);
    }

    let backup_path = path.with_extension("txt.bak");
    if path.exists() {
        let _ = fs::copy(&path, &backup_path);
    }
    fs::write(&path, &content).map_err(|e| format!("写入画质配置失败 {}: {e}", path.display()))?;
    Ok(content)
}

/// 打开 Apex local 目录（若不存在则创建）。
pub fn ensure_apex_local_folder() -> Result<PathBuf, String> {
    let dir = get_apex_local_folder_path()?;
    fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败 {}: {e}", dir.display()))?;
    Ok(dir)
}

#[cfg(test)]
mod videoconfig_tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_videoconfig(content: &str) -> (PathBuf, PathBuf) {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("apex_vcfg_test_{stamp}"));
        let path = dir.join("videoconfig.txt");
        fs::create_dir_all(&dir).unwrap();
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "{content}").unwrap();
        (dir, path)
    }

    #[test]
    fn parse_videoconfig_keys() {
        let sample = r#""VideoConfig"
{
	"setting.defaultres"		"1920"
	"setting.defaultresheight"		"1080"
}
"#;
        let map = parse_videoconfig_key_values(sample);
        assert_eq!(map.get("setting.defaultres").map(String::as_str), Some("1920"));
        assert_eq!(
            map.get("setting.defaultresheight").map(String::as_str),
            Some("1080")
        );
    }

    #[test]
    fn patch_videoconfig_updates_existing_key() {
        let sample = r#""VideoConfig"
{
	"setting.defaultres"		"1280"
}
"#;
        let patched = patch_videoconfig_line(sample, "setting.defaultres", "1920");
        assert!(patched.contains("\"setting.defaultres\"\t\t\"1920\""));
        assert!(!patched.contains("\"1280\""));
    }
}
