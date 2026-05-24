//! EA Desktop：`%LOCALAPPDATA%\Electronic Arts\EA Desktop`
//!
//! - **帐户名 / 头像**：INI 一般没有 Persona 与头像；由 `EADesktop.log` 的 `setUserId`+`nuchash` 配对 `user_*.ini`，再在 `EADesktopVerbose(.log/.bak)` 里按 `nuchash` 收集 `GetProfileResponse` 与 `userAvatar` URL（`.log` 覆盖 `.bak`）。
//! - **Apex 启动项**：`user.gamecommandline.origin.ofr.50.0002694`（键名大小写不敏感）。

use crate::utils::{CommandHiddenWindowExt, Println};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;

const APEX_CMD_KEY_LOWER: &str = "user.gamecommandline.origin.ofr.50.0002694";
const VERBOSE_TAIL_BYTES: u64 = 12 * 1024 * 1024;
const VERBOSE_PREAMBLE_BYTES: u64 = 1024 * 1024;
const NUCHASH_LOOKAHEAD: usize = 120;

const INI_DISPLAY_NAME_KEYS: &[&str] = &[
    "user.eadisplayname",
    "user.maynickname",
    "user.displayname",
    "user.personaname",
    "user.nickname",
    "user.username",
];
const INI_AVATAR_KEYS: &[&str] = &[
    "user.avataruri",
    "user.avatarurl",
    "user.pictureurl",
    "user.portraiturl",
    "user.profileimageurl",
];

#[derive(Clone, Debug, Default)]
struct NuchashProfile {
    persona: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EaDesktopUser {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub config_path: String,
    pub user_userid: String,
    pub nu_hash: String,
}

pub fn ea_desktop_dir() -> Result<PathBuf, String> {
    let local = std::env::var("LOCALAPPDATA")
        .map_err(|_| "未设置 LOCALAPPDATA（仅 Windows 支持 EA Desktop）".to_string())?;
    Ok(PathBuf::from(local).join("Electronic Arts").join("EA Desktop"))
}

fn user_ini_path(ea_user_id: &str) -> Result<PathBuf, String> {
    if !ea_user_id.chars().all(|c| c.is_ascii_digit()) {
        return Err("无效的 EA 用户 id".to_string());
    }
    let dir = ea_desktop_dir()?;
    let p = dir.join(format!("user_{}.ini", ea_user_id));
    if !p.is_file() {
        return Err(format!("未找到 EA 用户配置文件: {:?}", p));
    }
    Ok(p)
}

fn parse_ini_lines(content: &str) -> Vec<(String, String)> {
    content
        .lines()
        .filter_map(|line| {
            let t = line.trim().trim_end();
            if t.is_empty() || t.starts_with('#') || t.starts_with(';') {
                return None;
            }
            t.split_once('=')
                .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        })
        .collect()
}

fn ini_lower(lines: &[(String, String)]) -> HashMap<String, String> {
    lines
        .iter()
        .map(|(k, v)| (k.to_lowercase(), v.clone()))
        .collect()
}

fn first_value(lower: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|k| {
        lower
            .get(*k)
            .map(String::as_str)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    })
}

fn display_name_from_ini(lower: &HashMap<String, String>, file_id: &str) -> String {
    first_value(lower, INI_DISPLAY_NAME_KEYS).unwrap_or_else(|| format!("EA {}", file_id))
}

fn avatar_from_ini(lower: &HashMap<String, String>) -> String {
    first_value(lower, INI_AVATAR_KEYS).unwrap_or_default()
}

fn apex_cmd_line_index(lines: &[(String, String)]) -> Option<usize> {
    lines.iter().position(|(k, _)| k.eq_ignore_ascii_case(APEX_CMD_KEY_LOWER))
}

fn read_user_ini(path: &Path) -> Result<Vec<(String, String)>, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("读取 {:?}: {}", path, e))?;
    Ok(parse_ini_lines(&content))
}

fn write_user_ini(path: &Path, lines: &[(String, String)]) -> Result<(), String> {
    let out: String = lines
        .iter()
        .map(|(k, v)| format!("{}={}\n", k, v))
        .collect();
    fs::write(path, out).map_err(|e| format!("写入 {:?}: {}", path, e))
}

fn nuchash_from_line(line: &str) -> Option<String> {
    let key = "nuchash=[";
    let start = line.find(key)? + key.len();
    let rest = line.get(start..)?;
    let end = rest.find(']')?;
    let h = rest.get(..end)?.trim();
    if h.is_empty() || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(h.to_ascii_lowercase())
}

fn user_ini_id_from_line(line: &str) -> Option<String> {
    for (idx, _) in line.match_indices("user_") {
        let tail = line.get(idx + "user_".len()..)?;
        let id: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !id.is_empty() && tail[id.len()..].starts_with(".ini") {
            return Some(id);
        }
    }
    None
}

/// `setUserId` 之后：优先「Unable to open file」行内的 `user_*.ini`，否则窗口内第一个 `user_*.ini`。
fn user_ini_after_setuser(lines: &[String], i: usize, j_end: usize) -> Option<String> {
    let mut fallback = None;
    for line in lines.get(i + 1..j_end)? {
        if line.contains("setUserId") && line.contains("nuchash=[") {
            break;
        }
        let id = user_ini_id_from_line(line);
        let Some(id) = id else { continue };
        if line.contains("Unable to open file") {
            return Some(id);
        }
        if fallback.is_none() {
            fallback = Some(id);
        }
    }
    fallback
}

fn nuchash_to_config_id_from_log(path: &Path) -> HashMap<String, String> {
    let Ok(f) = fs::File::open(path) else {
        return HashMap::new();
    };
    let lines: Vec<String> = BufReader::new(f).lines().filter_map(Result::ok).collect();
    let n = lines.len();
    let mut out = HashMap::new();
    for i in 0..n {
        let line = &lines[i];
        if !(line.contains("setUserId") && line.contains("nuchash=[")) {
            continue;
        }
        let Some(hash) = nuchash_from_line(line) else {
            continue;
        };
        let j_end = (i + 1 + NUCHASH_LOOKAHEAD).min(n);
        if let Some(id) = user_ini_after_setuser(&lines, i, j_end) {
            out.insert(hash, id);
        }
    }
    out
}

fn persona_from_get_profile_line(line: &str) -> Option<String> {
    if !(line.contains("GetProfileResponse") && line.contains("Persona=\"")) {
        return None;
    }
    let key = "Persona=\"";
    let start = line.find(key)? + key.len();
    let rest = line.get(start..)?;
    let end = rest.find('"')?;
    let p = rest.get(..end)?;
    (!p.is_empty()).then_some(p.to_string())
}

fn user_avatar_url_from_line(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    let pos = lower.find("useravatar/")?;
    let before = line.get(..pos)?;
    let https_start = before.rfind("https://")?;
    let rest = line.get(https_start..)?;
    let end = rest
        .find(|c: char| {
            c.is_whitespace() || matches!(c, '"' | '\'' | '<' | ']')
        })
        .unwrap_or(rest.len());
    let mut url = rest
        .get(..end)?
        .trim_end_matches([')', ',', ';', '"', '\'', ']']);
    while url.ends_with('"') || url.ends_with('\'') || url.ends_with(']') {
        url = url.trim_end_matches(['"', '\'', ']']);
    }
    Some(url.to_string())
}

fn normalize_ea_avatar_url(url: &str) -> String {
    if url.is_empty() {
        return String::new();
    }
    const NEW_SIZE: &str = "208x208";
    url.replace("40X40", NEW_SIZE)
        .replace("40x40", NEW_SIZE)
        .replace("128X128", NEW_SIZE)
        .replace("128x128", NEW_SIZE)
}

fn read_verbose_tail(path: &Path, tail: u64, preamble: u64) -> Result<String, String> {
    let mut f = fs::File::open(path).map_err(|e| format!("打开 {:?}: {}", path, e))?;
    let len = f.metadata().map_err(|e| e.to_string())?.len();
    let want = tail.saturating_add(preamble).min(len);
    let start = len.saturating_sub(want);
    f.seek(SeekFrom::Start(start))
        .map_err(|e| format!("定位 {:?}: {}", path, e))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)
        .map_err(|e| format!("读取 {:?}: {}", path, e))?;
    let mut s = String::from_utf8_lossy(&buf).into_owned();
    if start > 0 {
        if let Some(i) = s.find('\n') {
            s.drain(..=i);
        }
    }
    Ok(s)
}

fn nuchash_profiles_from_verbose(text: &str) -> HashMap<String, NuchashProfile> {
    let mut map = HashMap::<String, NuchashProfile>::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        if line.contains("nuchash=[") {
            current = nuchash_from_line(line);
            continue;
        }
        let Some(ch) = current.as_ref() else {
            continue;
        };
        let e = map.entry(ch.clone()).or_default();
        if let Some(p) = persona_from_get_profile_line(line) {
            e.persona = Some(p);
        }
        if let Some(u) = user_avatar_url_from_line(line) {
            e.avatar_url = Some(u);
        }
    }
    map
}

fn tail_from_last_nuchash<'a>(text: &'a str, hash: &str) -> Option<&'a str> {
    let needle = format!("nuchash=[{}]", hash.to_lowercase());
    let mut last = None::<usize>;
    let mut pos = 0;
    while let Some(i) = text.get(pos..)?.find(&needle) {
        last = Some(pos + i);
        pos += i + needle.len();
    }
    last.and_then(|s| text.get(s..))
}

fn first_avatar_after_nuchash(text: &str, hash: &str) -> Option<String> {
    let tail = tail_from_last_nuchash(text, hash)?;
    tail
        .lines()
        .find_map(user_avatar_url_from_line)
}

fn first_persona_after_nuchash(text: &str, hash: &str) -> Option<String> {
    let tail = tail_from_last_nuchash(text, hash)?;
    tail.lines().find_map(persona_from_get_profile_line)
}

fn enrich_profiles(chunk: &str, profiles: &mut HashMap<String, NuchashProfile>, hashes: &[String]) {
    for h in hashes {
        if h.is_empty() {
            continue;
        }
        let e = profiles.entry(h.clone()).or_default();
        if e.avatar_url.is_none() {
            e.avatar_url = first_avatar_after_nuchash(chunk, h);
        }
        if e.persona.is_none() {
            e.persona = first_persona_after_nuchash(chunk, h);
        }
    }
}

fn last_avatar_in_text(text: &str) -> Option<String> {
    text.lines().filter_map(user_avatar_url_from_line).last()
}

fn merge_profiles(
    mut base: HashMap<String, NuchashProfile>,
    overlay: HashMap<String, NuchashProfile>,
) -> HashMap<String, NuchashProfile> {
    for (k, v) in overlay {
        let e = base.entry(k).or_default();
        if v.persona.is_some() {
            e.persona = v.persona;
        }
        if v.avatar_url.is_some() {
            e.avatar_url = v.avatar_url;
        }
    }
    base
}

fn invert_nuchash_map(m: &HashMap<String, String>) -> HashMap<String, String> {
    m.iter().map(|(h, id)| (id.clone(), h.clone())).collect()
}

fn ingest_verbose_file(
    path: &Path,
    profiles: &mut HashMap<String, NuchashProfile>,
    hashes: &[String],
    last_chunk: &mut Option<String>,
) {
    let Ok(text) = read_verbose_tail(path, VERBOSE_TAIL_BYTES, VERBOSE_PREAMBLE_BYTES) else {
        return;
    };
    *last_chunk = Some(text.clone());
    let mut chunk_profiles = nuchash_profiles_from_verbose(&text);
    enrich_profiles(&text, &mut chunk_profiles, hashes);
    *profiles = merge_profiles(std::mem::take(profiles), chunk_profiles);
}

pub fn ea_desktop_is_running_by_tasklist() -> Result<bool, String> {
    const NAMES: &[&str] = &[
        "EADesktop.exe",
        "EALauncher.exe",
        "EABackgroundAgent.exe",
        "EASteamProxy.exe",
    ];
    for name in NAMES {
        // 直接调用 tasklist 并传递参数，避免 cmd /C 的引号解析问题。
        let output = Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {}", name), "/FO", "CSV"])
            .with_hidden_window()
            .output()
            .map_err(|e| e.to_string())?;
        output.print_ln();
        let out = String::from_utf8_lossy(&output.stdout);
        if out.to_lowercase().contains(&name.to_ascii_lowercase()) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn get_ea_desktop_users() -> Result<Vec<EaDesktopUser>, String> {
    let dir = ea_desktop_dir()?;
    if !dir.is_dir() {
        return Ok(vec![]);
    }

    let hash_to_ini = nuchash_to_config_id_from_log(&dir.join("Logs").join("EADesktop.log"));
    let config_to_hash = invert_nuchash_map(&hash_to_ini);

    let mut hashes: Vec<String> = hash_to_ini.keys().cloned().collect();
    hashes.sort();
    hashes.dedup();

    let mut profiles = HashMap::new();
    let mut last_verbose_chunk = None;

    let bak = dir.join("Logs").join("EADesktopVerbose.bak");
    if bak.is_file() {
        ingest_verbose_file(&bak, &mut profiles, &hashes, &mut last_verbose_chunk);
    }
    let vlog = dir.join("Logs").join("EADesktopVerbose.log");
    if vlog.is_file() {
        ingest_verbose_file(&vlog, &mut profiles, &hashes, &mut last_verbose_chunk);
    }

    let mut users = Vec::new();
    for ent in fs::read_dir(&dir)
        .map_err(|e| format!("读取 EA Desktop 目录失败: {}", e))?
            .flatten()
    {
        let fname = ent.file_name().to_string_lossy().into_owned();
        let Some(ini_id) = fname
            .strip_prefix("user_")
            .and_then(|s| s.strip_suffix(".ini"))
            .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
        else {
            continue;
        };
        let path = ent.path();
        let Ok(lines) = read_user_ini(&path) else {
            continue;
        };
        let lower = ini_lower(&lines);
        let user_userid = first_value(&lower, &["user.userid"]).unwrap_or_default();
        let nu_hash = config_to_hash.get(ini_id).cloned().unwrap_or_default();

        let prof = (!nu_hash.is_empty())
            .then(|| profiles.get(&nu_hash).cloned())
            .flatten()
            .unwrap_or_default();

        let display_name = prof
            .persona
            .clone()
            .unwrap_or_else(|| display_name_from_ini(&lower, ini_id));
        let avatar_src = prof
            .avatar_url
            .clone()
            .unwrap_or_else(|| avatar_from_ini(&lower));
        let avatar = normalize_ea_avatar_url(&avatar_src);

        users.push(EaDesktopUser {
            id: ini_id.to_string(),
            name: display_name,
            avatar,
            config_path: path.to_string_lossy().into_owned(),
            user_userid,
            nu_hash,
        });
    }

    users.sort_by(|a, b| a.id.cmp(&b.id));

    if users.len() == 1 && users[0].avatar.is_empty() {
        if let Some(chunk) = last_verbose_chunk.as_ref() {
            if let Some(url) = last_avatar_in_text(chunk) {
                users[0].avatar = normalize_ea_avatar_url(&url);
            }
        }
    }
    Ok(users)
}

pub fn get_apex_launch_option_ea(ea_user_id: &str) -> Result<String, String> {
    let path = user_ini_path(ea_user_id)?;
    let lines = read_user_ini(&path)?;
    Ok(apex_cmd_line_index(&lines)
        .map(|i| lines[i].1.clone())
        .unwrap_or_default())
}

pub fn set_apex_launch_option_ea(ea_user_id: &str, launch_option: &str) -> Result<(), String> {
    let path = user_ini_path(ea_user_id)?;
    let mut lines = read_user_ini(&path)?;
    let key = "user.gamecommandline.origin.ofr.50.0002694";
    match apex_cmd_line_index(&lines) {
        Some(i) => lines[i].1 = launch_option.to_string(),
        None => lines.push((key.to_string(), launch_option.to_string())),
    }
    write_user_ini(&path, &lines)
}
