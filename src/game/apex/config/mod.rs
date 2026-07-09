//! Apex 本地配置文件解析与读写（`videoconfig.txt`、`settings.cfg`、`profile.cfg`）。
//!
//! 设计参考 [`crate::vdf`]：提供文档模型、路径访问、CRUD 与文件加载/写回。

mod crud;
mod encoding;
mod file;
mod parse;
mod value;

pub use encoding::{decode_bytes, read_text_file, write_text_file, ApexFileEncoding};
pub use file::{
    ensure_apex_local_folder, get_apex_config_path, get_apex_local_folder_path,
    get_apex_profile_folder_path, get_apex_saved_games_root, is_videoconfig_readonly,
    patch_apex_videoconfig, read_apex_videoconfig_map, set_videoconfig_readonly,
    ApexConfigFileKind,
};
pub use value::{ApexCfgDocument, ApexCfgLine};
