// 提取最后一个字段
pub fn extract_last_field(bytes: &[u8]) -> Option<&str> {
    // 按空字节(0x00)分割，取最后一个非空字段
    let fields: Vec<&[u8]> = bytes.split(|&b| b == 0).collect();

    // 从后往前找第一个非空字段
    for field in fields.iter().rev() {
        if !field.is_empty() {
            // 尝试转换为字符串，跳过控制字符
            if let Ok(s) = std::str::from_utf8(field) {
                // 检查是否包含控制字符（ASCII < 0x20）
                if !s
                    .chars()
                    .any(|c| c.is_control() && c != '\n' && c != '\t' && c != '\r')
                {
                    return Some(s);
                }
            }
        }
    }
    None
}
