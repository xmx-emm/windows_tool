//! PUBG（Steam App ID [`PUBG_APP_ID`]）启动项读写。
//!
//! 额外：支持通过“重命名 Content/Movies 目录”来禁用开场动画（可恢复）。

use crate::game::steam::{
    get_steam_game_library_folder_by_game_id, get_steam_game_launch_options,
    set_steam_game_launch_options,
};
use crate::registry::steam::get_steam_path_by_registry;
use std::fs;
use std::path::PathBuf;

/// PUBG 的 Steam App ID。
pub const PUBG_APP_ID: usize = 578080;

// 获取 PUBG Steam 启动选项
pub fn get_pubg_launch_options_by_steam_user_id(steam_user_id: usize) -> Result<String, String> {
    get_steam_game_launch_options(steam_user_id, PUBG_APP_ID)
}

// 设置 PUBG Steam 启动选项
pub fn set_pubg_launch_options_by_steam_user_id<T: AsRef<str>>(
    steam_user_id: usize,
    launch_options: T,
) -> Result<(), String> {
    set_steam_game_launch_options(steam_user_id, PUBG_APP_ID, launch_options)
}

fn normalize_steam_library_folder(library_folder: String) -> Result<PathBuf, String> {
    let pb = PathBuf::from(library_folder.trim());
    if pb.is_absolute() {
        return Ok(pb);
    }
    // 有些情况下 Steam 的 libraryfolders.vdf 可能是相对路径。
    // 这种情况下用注册表中的 SteamPath 当作基准拼接。
    if let Some(steam_path) = get_steam_path_by_registry() {
        return Ok(PathBuf::from(steam_path).join(pb));
    }
    Err("Steam library folder path is relative but SteamPath is unavailable".to_string())
}

fn pubg_movies_content_dir() -> Result<PathBuf, String> {
    let library_folder = get_steam_game_library_folder_by_game_id(PUBG_APP_ID)
        .map_err(|e| format!("未找到 PUBG 在 Steam 库中的安装目录: {e}"))?;
    let library_folder = normalize_steam_library_folder(library_folder)?;

    Ok(library_folder
        .join("steamapps")
        .join("common")
        .join("PUBG")
        .join("TslGame")
        .join("Content"))
}

fn pubg_movies_paths() -> Result<(PathBuf, PathBuf), String> {
    let content_dir = pubg_movies_content_dir()?;
    let movies = content_dir.join("Movies");
    let movies_disabled = content_dir.join("Movies_disabled");
    Ok((movies, movies_disabled))
}

/// 检查“禁用开场动画”是否已生效（即 Movies 目录是否已被重命名为 Movies_disabled）。
pub fn check_pubg_skip_intro_movies_disabled() -> Result<bool, String> {
    let (_, movies_disabled) = pubg_movies_paths()?;
    Ok(movies_disabled.exists())
}

/// 设置“禁用开场动画”状态（通过 Movies <-> Movies_disabled 可逆重命名）。
///
/// disabled=true: Movies -> Movies_disabled
/// disabled=false: Movies_disabled -> Movies
pub fn set_pubg_skip_intro_movies_disabled(disabled: bool) -> Result<(), String> {
    let (movies, movies_disabled) = pubg_movies_paths()?;

    // 两个目录同时存在一般表示之前操作异常或用户手动处理过。
    if movies.exists() && movies_disabled.exists() {
        return Err(format!(
            "Movies 与 Movies_disabled 同时存在，无法确定状态: movies={} movies_disabled={}",
            movies.display(),
            movies_disabled.display()
        ));
    }

    if disabled {
        if movies_disabled.exists() {
            return Ok(()); // 已禁用
        }
        if !movies.exists() {
            return Err(format!(
                "未找到 Movies 目录，无法禁用开场动画: {}",
                movies.display()
            ));
        }
        fs::rename(&movies, &movies_disabled)
            .map_err(|e| format!("重命名失败 Movies -> Movies_disabled: {e}"))?;
        Ok(())
    } else {
        if movies.exists() {
            return Ok(()); // 已恢复
        }
        if !movies_disabled.exists() {
            return Err(format!(
                "未找到 Movies_disabled 目录，无法恢复开场动画: {}",
                movies_disabled.display()
            ));
        }
        fs::rename(&movies_disabled, &movies)
            .map_err(|e| format!("重命名失败 Movies_disabled -> Movies: {e}"))?;
        Ok(())
    }
}

#[allow(dead_code)]
fn _debug_pubg_movies_paths() -> Result<(String, String), String> {
    let (movies, movies_disabled) = pubg_movies_paths()?;
    Ok((movies.display().to_string(), movies_disabled.display().to_string()))
}

