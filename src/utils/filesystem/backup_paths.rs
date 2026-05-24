use crate::utils::filesystem::{CreateDir, get_documents_path};
use crate::utils::{now_ymd, now_ymd_hms};
use std::path::{Path, PathBuf};

pub fn backups_folder<T: AsRef<str>>(categorize: T, create_dir: bool) -> Option<PathBuf> {
    match get_documents_path() {
        Some(path) => {
            let folder = Path::new(&path).join(categorize.as_ref()).join("backups");
            if folder.create_dir(create_dir).is_none_or(|i| !i) {
                println!("creating folder {}", folder.display());
                return None;
            }
            Some(folder)
        }
        None => {
            println!("get_documents_path path not found in path");
            None
        }
    }
}

pub fn backups_explorer_registry_path<T: AsRef<str>>(
    categorize: T,
    create_dir: bool,
) -> Option<String> {
    match backups_folder(categorize, create_dir) {
        Some(folder) => match folder.join(format!("explorer {}.reg", now_ymd())).to_str() {
            Some(registry_path) => Some(registry_path.to_string()),
            None => {
                println!("join path error {}", folder.display());
                None
            }
        },
        None => None,
    }
}

pub fn backups_port_forwarding_json_path<T: AsRef<str>>(
    categorize: T,
    create_dir: bool,
) -> Option<String> {
    match backups_folder(categorize, create_dir) {
        Some(folder) => match folder
            .join(format!("port_forwarding {}.json", now_ymd_hms()))
            .to_str()
        {
            Some(registry_path) => Some(registry_path.to_string()),
            None => {
                println!("join path error {}", folder.display());
                None
            }
        },
        None => None,
    }
}
