use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE, REG_EXPAND_SZ, REG_SZ};
use winreg::RegKey;

lazy_static! {
  /// HKEY_LOCAL_MACHINE
  /// ("DESKTOP",vec!("B4BFCC3A-DB2C-424C-B029-7FE99A87C641")),//桌面文件夹
  /// ("DOCUMENTS",vec!("f42ee2d3-909f-4907-8871-4c22fc0bf756","A8CDFF1C-4878-43be-B5FD-F8091C1C60D0")),//文档文件夹
  /// ("VIDEOS",vec!("35286a68-3c57-41a1-bbb1-0eae73d76c95","A0953C92-50DC-43bf-BE83-3742FED03C9C","f86fa3ab-70d2-4fc7-9c99-fcbf05467f3a")),//视频文件夹
  /// ("PICTURES",vec!("0ddd015d-b06c-45d5-8c4c-f59713854639","24ad3ad4-a569-4530-98e1-ab02f9417aa8","3ADD1653-EB32-4cb0-BBD7-DFA0ABB5ACCA")),//图片文件夹
  /// ("DOWNLOADS",vec!("7d83ee9b-2244-4e70-b1f5-5393042af1e4","374DE290-123F-4565-9164-39C4925E467B","088e3905-0323-4b02-9826-5d99428e115f")),//下载文件夹
  /// ("MUSIC",vec!("a0c69a99-21c8-4671-8703-7934162fcf1d","1CF1260C-4DD0-4ebb-811F-33C572699FDE","3dfdf296-dbec-4fb4-81d1-6a3438bcf4de")),//音乐文件夹
  /// ("PUBLIC",vec!("4336a54d-038b-4685-ab02-99bb52d3fb8b")),//通用文件夹
  /// ("FONTS",vec!("BD84B380-8CA2-1069-AB1D-08000948F534")),//字体文件夹
  static ref RegistryCommonFolders:HashMap<String, String> = {
    let mycomputer = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\MyComputer\NameSpace\{%s}";
    let folderdescriptions = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\FolderDescriptions\{%s}\PropertyBag";
    let mut sub_key: HashMap<String, String> = HashMap::new();
    sub_key.insert("3D_OBJECTS".to_string(),mycomputer.replace("%s","0DB7E03F-FC29-4DC6-9020-FF41B59E513A"));
    [
        ("PICTURES","0ddd015d-b06c-45d5-8c4c-f59713854639"),//图片
        ("VIDEOS","35286a68-3c57-41a1-bbb1-0eae73d76c95"),//视频
        ("DOWNLOADS","7d83ee9b-2244-4e70-b1f5-5393042af1e4"),//下载
        ("MUSIC","a0c69a99-21c8-4671-8703-7934162fcf1d"),//音乐
        ("DESKTOP","B4BFCC3A-DB2C-424C-B029-7FE99A87C641"),//桌面
        ("DOCUMENTS","f42ee2d3-909f-4907-8871-4c22fc0bf756")//文档
    ].iter().for_each(|(k,v)| {
        sub_key.insert(k.to_string(), folderdescriptions.replace("%s",v));
    });
    sub_key
  };
}

/// 已知常用文件夹键名（与 `RegistryCommonFolders` 一致）。
pub fn known_common_folder_keys() -> Vec<&'static str> {
    RegistryCommonFolders.keys().map(|k| k.as_str()).collect()
}

pub fn is_known_common_folder_key(key: &str) -> bool {
    RegistryCommonFolders.contains_key(key)
}

fn folder_path(key: &str) -> Result<&'static str, Box<dyn Error>> {
    RegistryCommonFolders
        .get(key)
        .map(|s| s.as_str())
        .ok_or_else(|| format!("未知常用文件夹 key: {}", key).into())
}

fn get_3d_objects() -> Result<bool, Box<dyn Error>> {
    let hk = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = folder_path("3D_OBJECTS")?;
    Ok(hk.open_subkey(key).is_ok())
}

/// 获取常用文件夹状态
pub fn get_common_folder_state<T: AsRef<str>>(key: T) -> Result<bool, Box<dyn Error>> {
    let key = key.as_ref();
    if key == "3D_OBJECTS" {
        return get_3d_objects();
    }
    let path = folder_path(key)?;

    let reg_key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(path);
    if reg_key.is_err() {
        return Err(format!("open_subkey 错误 {} {}", key, path).into());
    }

    let raw_value = match reg_key?.get_raw_value("ThisPCPolicy") {
        Ok(v) => v,
        Err(_) => {
            //没有值,在文档上面遇到过
            println!("get_raw_value 错误 {} {}", key, path);
            return Ok(false);
        }
    };
    let res = match raw_value.vtype {
        REG_SZ | REG_EXPAND_SZ => raw_value.to_string(),
        _ => {
            return Err(format!(
                "ThisPCPolicy 类型非 REG_SZ/REG_EXPAND_SZ: {} {:?}",
                key, raw_value.vtype
            )
            .into());
        }
    };
    let res_trim = res.trim_end_matches('\0').trim();
    // 与 Explorer 一致：英文 Show/Hide；大小写不敏感以兼容异常写入
    match res_trim.to_ascii_lowercase().as_str() {
        "show" => Ok(true),
        "hide" => Ok(false),
        _ => {
            // 误把 UTF-8 当 REG_SZ 写入时读回会变成乱码；仍返回 Ok 以免 get_all_state 丢项，用户切换一次即可修复
            println!(
                "ThisPCPolicy 非 Show/Hide（可能已损坏），暂按「显示」处理，请点一次切换以重写: {} = {:?}",
                key, res_trim
            );
            Ok(true)
        }
    }
}

pub fn switch_by_api(key: &str, show: bool) -> Result<(), Box<dyn Error>> {
    let path = folder_path(key)?;
    // 必须用 `set_value` / `ToRegValue`：REG_SZ 在注册表里是 UTF-16LE，不能 `.as_bytes()` 当 REG_SZ 写
    let policy: &str = if show { "Show" } else { "Hide" };
    match RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(path, KEY_READ | KEY_WRITE)
    {
        Ok(reg_key) => reg_key
            .set_value("ThisPCPolicy", &policy)
            .map_err(|e| e.to_string().into()),
        Err(e) => Err(format!("open_subkey 错误 {} {} {}", key, path, e).into()),
    }
}

fn switch_by_cmd(key: &str, show: bool) -> Result<(), Box<dyn Error>> {
    // 与 [`switch_by_api`] 一致：显示 → ThisPCPolicy=Show，隐藏 → Hide
    let path = folder_path(key)?;

    if key == "3D_OBJECTS" {
        // 「此电脑」中 3D 对象由 MyComputer\NameSpace\{GUID} 是否存在控制：存在则显示
        let hk = RegKey::predef(HKEY_LOCAL_MACHINE);
        if show {
            hk.create_subkey(path)
                .map(|_| ())
                .map_err(|e| format!("create_subkey Error: {} {}", key, e).into())
        } else {
            hk.delete_subkey(path)
                .map_err(|e| format!("delete_subkey Error: {} {}", key, e).into())
        }
    } else {
        switch_by_api(key, show)
    }
}

/// 显示某个常用文件夹
pub fn show<T: AsRef<str>>(key: T) -> Result<(), Box<dyn Error>> {
    switch_by_cmd(key.as_ref(), true)
}

/// 隐藏某个常用文件夹
pub fn hide<T: AsRef<str>>(key: T) -> Result<(), Box<dyn Error>> {
    switch_by_cmd(key.as_ref(), false)
}

///获取所有的常用文件夹开启情况
pub fn get_all_state() -> HashMap<String, bool> {
    let mut hm: HashMap<String, bool> = HashMap::new();
    RegistryCommonFolders
        .keys()
        .for_each(|f| match get_common_folder_state(f) {
            Ok(state) => {
                hm.insert(f.to_string(), state);
            }
            Err(e) => println!("{}", e),
        });
    hm
}
