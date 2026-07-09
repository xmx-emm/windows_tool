//! Apex Legends（Steam App ID `1172470`）启动项、语音资源路径、本地配置与 depot 目录等。

pub mod config;

use crate::registry::steam::get_steam_path_by_registry;
use crate::game::steam::language::SteamLanguage;
use crate::game::steam::{
    get_steam_game_launch_options, get_steam_game_library_folder_by_game_id,
    set_steam_game_launch_options,
};
use crate::utils::CommandHiddenWindowExt;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub use config::{
    ensure_apex_local_folder, get_apex_config_path, get_apex_local_folder_path,
    get_apex_profile_folder_path, get_apex_saved_games_root, is_videoconfig_readonly,
    patch_apex_videoconfig, read_apex_videoconfig_map, set_videoconfig_readonly, ApexCfgDocument,
    ApexConfigFileKind,
};

lazy_static! {
    /// apex depots的语言表
    pub static ref APEX_LANGUAGES_DEPOTS: HashMap<SteamLanguage,i32> = {
        let mut map: HashMap<SteamLanguage,i32> = HashMap::new();
        map.insert(SteamLanguage::FRENCH,1172472);
        map.insert(SteamLanguage::GERMAN,1172473);
        map.insert(SteamLanguage::ITALIAN,1172474);
        map.insert(SteamLanguage::JAPANESE,1172475);
        map.insert(SteamLanguage::KOREAN,1172476);
        map.insert(SteamLanguage::SIMPLIFIED_CHINESE ,1172477);
        map.insert(SteamLanguage::POLISH ,1172478);
        map.insert(SteamLanguage::RUSSIAN ,1172479);
        map.insert(SteamLanguage::SPANISH ,1311105);
        map
    };
}

pub fn get_apex_launch_options_by_steam_user_id(steam_user_id: usize) -> Result<String, String> {
    get_steam_game_launch_options(steam_user_id, 1172470)
}

pub fn set_apex_launch_options_by_steam_user_id<T: AsRef<str>>(
    steam_user_id: usize,
    launch_options: T,
) -> Result<(), String> {
    set_steam_game_launch_options(steam_user_id, 1172470, launch_options)
}

/// 通过 `tasklist` 检测 Apex 游戏进程是否正在运行。
pub fn apex_is_running_by_tasklist() -> Result<bool, String> {
    const PROCESS_NAMES: [&str; 2] = ["r5apex.exe", "r5apex_dx12.exe"];
    for name in PROCESS_NAMES {
        let output = Command::new("tasklist")
            .with_hidden_window()
            .args(["/FI", &format!("IMAGENAME eq {name}"), "/FO", "CSV"])
            .output()
            .map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.to_ascii_lowercase().contains(name) {
            return Ok(true);
        }
    }
    Ok(false)
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
        base.join("Apex").join("audio").join("ship")
    })
}

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
        "ea" => ea_user_id.and_then(get_apex_audio_folder_path_ea),
        _ => get_apex_depot_download_folder_path(depot),
    }
}

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

/// 读取 `videoconfig.txt` 全文与 `setting.*` 键值。
pub fn read_apex_videoconfig() -> Result<(String, HashMap<String, String>), String> {
    let path = get_apex_config_path(ApexConfigFileKind::VideoConfig)?;
    let doc = ApexCfgDocument::load_from_file(&path)?;
    let map = doc.key_values().into_iter().collect();
    Ok((doc.to_string(), map))
}

/// 加载任意 Apex 配置文件为文档。
pub fn load_apex_config(kind: ApexConfigFileKind) -> Result<ApexCfgDocument, String> {
    ApexCfgDocument::load(kind)
}

/// 保存 Apex 配置文件文档。
pub fn save_apex_config(doc: &ApexCfgDocument, kind: ApexConfigFileKind) -> Result<(), String> {
    doc.save(kind)
}

/// `.../local/videoconfig.txt`
pub fn get_apex_videoconfig_path() -> Result<PathBuf, String> {
    get_apex_config_path(ApexConfigFileKind::VideoConfig)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_videoconfig_keys() {
        let sample = r#""VideoConfig"
{
	"setting.defaultres"		"1920"
	"setting.defaultresheight"		"1080"
}
"#;
        let doc = ApexCfgDocument::from_content(sample, config::ApexFileEncoding::Utf8).unwrap();
        assert_eq!(doc.get("setting.defaultres"), Some("1920"));
    }

    #[test]
    fn patch_videoconfig_updates_existing_key() {
        let sample = r#""VideoConfig"
{
	"setting.defaultres"		"1280"
}
"#;
        let mut doc =
            ApexCfgDocument::from_content(sample, config::ApexFileEncoding::Utf8).unwrap();
        doc.set("setting.defaultres", "1920").unwrap();
        let out = doc.to_string();
        assert!(out.contains("\"1920\""));
        assert!(!out.contains("\"1280\""));
    }
}
