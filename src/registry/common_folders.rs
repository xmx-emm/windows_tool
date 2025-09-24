use crate::utils::{Println, run_commands};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use winreg::enums::{HKEY_LOCAL_MACHINE, REG_SZ};
use winreg::{RegKey, RegValue};

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

fn get_3d_objects() -> Result<bool, Box<dyn Error>> {
    let hk = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = RegistryCommonFolders
        .get("3D_OBJECTS")
        .expect("failed to get 3D_OBJECTS");
    Ok(hk.open_subkey(key).is_ok())
}

/// 获取常用文件夹状态
pub fn get_common_folder_state<T: AsRef<str>>(key: T) -> Result<bool, Box<dyn Error>> {
    let key = key.as_ref();
    if key == "3D_OBJECTS" {
        return get_3d_objects();
    }
    let path = RegistryCommonFolders
        .get(key)
        .expect(format!("No Folder for key {}", key).as_str());

    let reg_key = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(path);
    if reg_key.is_err() {
        return Err(format!("open_subkey 错误 {} {}", key, path).into());
    }

    let raw_value = reg_key.unwrap().get_raw_value("ThisPCPolicy");
    if raw_value.is_err() {
        //没有值,在文档上面遇到过
        println!("get_raw_value 错误 {} {}", key, path);
        return Ok(false);
    }
    let value = &raw_value.unwrap();
    if value.vtype == REG_SZ {
        let res = value.to_string();
        return if res == "Show" {
            Ok(true)
        } else if res == "Hide" {
            Ok(false)
        } else {
            Err(format!("emme 这个值两个都没匹配到 {} {}", key, res).into())
        };
    }
    Err(format!("值类型错误 !=RegType::REG_SZ {} {:?}", key, value.vtype).into())
}

pub fn switch_by_api(key: &str, show: bool) -> Result<(), Box<dyn Error>> {
    let path = RegistryCommonFolders
        .get(key)
        .expect(format!("No Folder for key {}", key).as_str());
    let bytes = (if show { "Show" } else { "Hide" }).as_bytes().to_vec();
    match RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(path) {
        Ok(reg_key) => {
            match reg_key.set_raw_value(
                "ThisPCPolicy",
                &RegValue {
                    vtype: REG_SZ,
                    bytes,
                },
            ) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string().into()),
            }
        }
        Err(e) => Err(format!("open_subkey 错误 {} {} {}", key, path, e).into()),
    }
}
fn switch_by_cmd(key: &str, show: bool) {
    let bytes = if !show { "Show" } else { "Hide" };
    let path = RegistryCommonFolders
        .get(key)
        .expect(format!("No Folder for key {}", key).as_str());

    if key == "3D_OBJECTS" {
        let hk = RegKey::predef(HKEY_LOCAL_MACHINE);
        if show {
            match hk.delete_subkey(path) {
                Ok(_) => {}
                Err(e) => {
                    println!("delete_subkey Error: {}", e)
                }
            };
        } else {
            match hk.create_subkey(path) {
                Ok(_) => {}
                Err(e) => {
                    println!("create_subkey Error: {}", e)
                }
            };
        }
    } else {
        run_commands(
            format!(
                "reg add \"HKLM\\{}\" /v ThisPCPolicy /t REG_SZ /d \"{}\" /f",
                path, bytes
            ),
            true,
            false,
        )
        .print_ln();
    }
}

/// 显示某个常用文件夹
///
pub fn show<T: AsRef<str>>(key: T) {
    switch_by_cmd(key.as_ref(), true)
}

/// 隐藏某个常用文件夹
/// reg add "HKEY_LOCAL_MACHINE\SOFTWARE\MyApp" /v SettingName /t REG_SZ /d "ValueData" /f
pub fn hide<T: AsRef<str>>(key: T) {
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
