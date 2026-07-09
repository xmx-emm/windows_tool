//! Apex 配置文件读写时的编码探测与转换。

use encoding_rs::UTF_16LE;
use std::fs;
use std::path::Path;

/// 写入文件时使用的编码。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApexFileEncoding {
    Utf8,
    Utf16Le,
}

impl ApexFileEncoding {
    pub fn encode(&self, content: &str) -> Vec<u8> {
        match self {
            ApexFileEncoding::Utf8 => content.as_bytes().to_vec(),
            ApexFileEncoding::Utf16Le => {
                let mut bytes = Vec::with_capacity(2 + content.len() * 2);
                bytes.extend_from_slice(&[0xFF, 0xFE]);
                for unit in content.encode_utf16() {
                    bytes.extend_from_slice(&unit.to_le_bytes());
                }
                bytes
            }
        }
    }
}

/// 从磁盘读取文本，自动识别 UTF-8 / UTF-16 LE BOM / 无 BOM UTF-8。
pub fn read_text_file<P: AsRef<Path>>(path: P) -> Result<(String, ApexFileEncoding), String> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(|e| format!("无法读取文件 {}: {e}", path.display()))?;
    decode_bytes(&bytes)
}

/// 移除 Apex 配置文本中的 NUL，避免 EA 启动校验失败。
pub fn sanitize_config_text(text: &str) -> String {
    text.chars().filter(|c| *c != '\0').collect()
}

pub fn decode_bytes(bytes: &[u8]) -> Result<(String, ApexFileEncoding), String> {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let (cow, _, _) = UTF_16LE.decode(&bytes[2..]);
        return Ok((sanitize_config_text(&cow), ApexFileEncoding::Utf16Le));
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return Err("暂不支持 UTF-16 BE 编码的 Apex 配置文件".to_string());
    }
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let s = std::str::from_utf8(&bytes[3..])
            .map_err(|e| format!("UTF-8 BOM 文件解码失败: {e}"))?;
        return Ok((sanitize_config_text(s), ApexFileEncoding::Utf8));
    }
    if let Ok(s) = std::str::from_utf8(bytes) {
        return Ok((sanitize_config_text(s), ApexFileEncoding::Utf8));
    }
    // 无 BOM：尝试 UTF-16 LE（偶数字节且大量 0x00）
    if bytes.len() >= 2 && bytes.len() % 2 == 0 {
        let zero_count = bytes.iter().skip(1).step_by(2).filter(|&&b| b == 0).count();
        if zero_count > bytes.len() / 4 {
            let (cow, _, _) = UTF_16LE.decode(bytes);
            return Ok((sanitize_config_text(&cow), ApexFileEncoding::Utf16Le));
        }
    }
    // 回退 GBK（中文 Windows 常见）
    use encoding_rs::GBK;
    let (cow, _, _) = GBK.decode(bytes);
    Ok((sanitize_config_text(&cow), ApexFileEncoding::Utf8))
}

pub fn write_text_file<P: AsRef<Path>>(
    path: P,
    content: &str,
    encoding: ApexFileEncoding,
) -> Result<(), String> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败 {}: {e}", parent.display()))?;
    }
    fs::write(path, encoding.encode(content))
        .map_err(|e| format!("写入文件失败 {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_utf8_content() {
        let bytes = b"\"VideoConfig\"\n{\n}\n";
        let (text, enc) = decode_bytes(bytes).unwrap();
        assert_eq!(enc, ApexFileEncoding::Utf8);
        assert!(text.contains("VideoConfig"));
    }

    #[test]
    fn decode_strips_nul_bytes() {
        let bytes = b"\"VideoConfig\"\n{\n}\n\0\n";
        let (text, _) = decode_bytes(bytes).unwrap();
        assert!(!text.contains('\0'));
        assert!(text.contains("VideoConfig"));
    }
}
