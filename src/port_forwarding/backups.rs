use crate::port_forwarding::PortForwarding;
use crate::port_forwarding::command::get_all_ipv4_to_ipv4_port_proxy_silent;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// 备份资源管理器注册表
pub fn backups_to_file<T: AsRef<str>>(file_path: T) -> bool {
    let list = get_all_ipv4_to_ipv4_port_proxy_silent();
    let serialized = serde_json::to_string(&list).unwrap();
    let path = file_path.as_ref();
    match File::create(path) {
        Ok(mut file) => match file.write_all(serialized.as_bytes()) {
            Ok(_) => Path::new(path).is_file(),
            Err(_) => {
                println!("Failed to write file {}", path);
                false
            }
        },
        Err(_) => {
            println!("Failed to create file {}", path);
            false
        }
    }
}

pub fn load_by_file<T: AsRef<str>>(file_path: T) -> Option<Vec<PortForwarding>> {
    let path = file_path.as_ref();
    match File::open(path) {
        Ok(mut file) => {
            let mut content = String::new();
            match file.read_to_string(&mut content) {
                Ok(_) => {
                    let deserialized: Vec<PortForwarding> = serde_json::from_str(&content).unwrap();
                    return Some(deserialized);
                }
                Err(_) => {
                    println!("Failed to read file {}", path);
                }
            }
        }
        Err(_) => {
            println!("Failed to open file {}", path);
        }
    }
    None
}
