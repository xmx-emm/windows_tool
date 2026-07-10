use crate::port_forwarding::PortForwarding;
use crate::port_forwarding::command::get_all_ipv4_to_ipv4_port_proxy_silent;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// 备份端口转发到文件
pub fn backups_to_file<T: AsRef<str>>(file_path: T) -> Result<bool, String> {
    let list = get_all_ipv4_to_ipv4_port_proxy_silent();
    let serialized =
        serde_json::to_string(&list).map_err(|e| format!("序列化端口转发失败: {}", e))?;
    let path = file_path.as_ref();
    let mut file =
        File::create(path).map_err(|e| format!("Failed to create file {}: {}", path, e))?;
    file.write_all(serialized.as_bytes())
        .map_err(|e| format!("Failed to write file {}: {}", path, e))?;
    Ok(Path::new(path).is_file())
}

pub fn load_by_file<T: AsRef<str>>(file_path: T) -> Result<Vec<PortForwarding>, String> {
    let path = file_path.as_ref();
    let mut file = File::open(path).map_err(|e| format!("Failed to open file {}: {}", path, e))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read file {}: {}", path, e))?;
    serde_json::from_str(&content).map_err(|e| format!("端口转发 JSON 解析失败: {}", e))
}
