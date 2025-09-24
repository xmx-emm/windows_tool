use crate::utils::{now_ymd, now_ymd_hms};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::ptr;
use winapi::ctypes::c_void;
use winapi::um::combaseapi::CoTaskMemFree;
use winapi::um::knownfolders::FOLDERID_Documents;
use winapi::um::shlobj::SHGetKnownFolderPath;
use winapi::um::winnt::PWSTR;

pub fn get_documents_path() -> Option<OsString> {
    unsafe {
        let mut path_ptr: PWSTR = ptr::null_mut();
        let hr = SHGetKnownFolderPath(&FOLDERID_Documents, 0, ptr::null_mut(), &mut path_ptr);

        if hr == 0 && !path_ptr.is_null() {
            // 计算字符串长度
            let mut len = 0;
            while *path_ptr.offset(len) != 0 {
                len += 1;
            }

            // 转换为 OsString
            let wide_slice = std::slice::from_raw_parts(path_ptr, len as usize);
            let path = OsString::from_wide(wide_slice);

            // 释放内存
            CoTaskMemFree(path_ptr as *mut c_void);

            Some(path)
        } else {
            None
        }
    }
}

pub fn backups_folder<T: AsRef<str>>(categorize: T, create_dir: bool) -> Option<PathBuf> {
    match get_documents_path() {
        Some(path) => {
            let folder = Path::new(&path).join(categorize.as_ref()).join("backups");
            if folder.create_dir(create_dir).is_none_or(|i| i == false) {
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

pub trait CreateDir {
    ///创建文件夹
    fn create_dir(&self, create_dir: bool) -> Option<bool>;
}

impl CreateDir for PathBuf {
    fn create_dir(&self, create_dir: bool) -> Option<bool> {
        if !self.exists() {
            if create_dir {
                return match std::fs::create_dir_all(self) {
                    Ok(_) => {
                        println!("create dir finished {}", self.display());
                        Some(self.exists())
                    }
                    Err(_) => {
                        println!("Create Dir error {}", self.display());
                        None
                    }
                };
            }
        }
        Some(true)
    }
}
