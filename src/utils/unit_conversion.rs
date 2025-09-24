use std::fmt;

// 定义转换标准枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ByteConversionStandard {
    Binary,  // 二进制标准 (IEC) - 1 GB = 1024^3 字节
    Decimal, // 十进制标准 (SI) - 1 GB = 1000^3 字节
}

// 为 u64 类型实现字节转换方法
pub trait ByteToGB {
    fn to_gb(&self, standard: ByteConversionStandard) -> f64;
    fn to_gb_binary(&self) -> f64;
    fn to_gb_decimal(&self) -> f64;
    fn format_bytes(&self) -> String;
}

impl ByteToGB for u64 {
    /// 根据指定标准将字节转换为 GB
    fn to_gb(&self, standard: ByteConversionStandard) -> f64 {
        match standard {
            ByteConversionStandard::Binary => self.to_gb_binary(),
            ByteConversionStandard::Decimal => self.to_gb_decimal(),
        }
    }

    /// 使用二进制标准转换 (1 GB = 1024^3 字节)
    fn to_gb_binary(&self) -> f64 {
        *self as f64 / 1024.0 / 1024.0 / 1024.0
    }

    /// 使用十进制标准转换 (1 GB = 1000^3 字节)
    fn to_gb_decimal(&self) -> f64 {
        *self as f64 / 1000.0 / 1000.0 / 1000.0
    }

    /// 格式化字节显示（自动选择合适的单位）
    fn format_bytes(&self) -> String {
        const UNITS: [&str; 7] = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
        let mut value = *self as f64;
        let mut unit_index = 0;

        while value >= 1024.0 && unit_index < UNITS.len() - 1 {
            value /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", value, UNITS[unit_index])
    }
}

// 为转换标准实现 Display trait
impl fmt::Display for ByteConversionStandard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ByteConversionStandard::Binary => write!(f, "二进制 (IEC)"),
            ByteConversionStandard::Decimal => write!(f, "十进制 (SI)"),
        }
    }
}

// 单元测试
///
///```
/// use windows_tool::utils::unit_conversion::{ByteConversionStandard, ByteToGB};
/// let bytes: u64 = 15_000_000_000; // 150亿字节
///
/// println!("原始字节: {}", bytes.format_bytes());
/// println!("二进制标准 (IEC): {:.6} GB", bytes.to_gb_binary());
/// println!("十进制标准 (SI): {:.6} GB", bytes.to_gb_decimal());
///
/// // 使用枚举指定标准
/// println!(
///     "使用二进制标准: {:.6} GB",
///     bytes.to_gb(ByteConversionStandard::Binary)
/// );
/// println!(
///     "使用十进制标准: {:.6} GB",
///     bytes.to_gb(ByteConversionStandard::Decimal)
/// );
///
/// // 比较两种标准的差异
/// let binary_gb = bytes.to_gb_binary();
/// let decimal_gb = bytes.to_gb_decimal();
/// let difference = (binary_gb - decimal_gb).abs();
/// let percent_diff = (difference / decimal_gb) * 100.0;
///
/// println!("\n转换差异:");
/// println!("二进制: {:.6} GB", binary_gb);
/// println!("十进制: {:.6} GB", decimal_gb);
/// println!("绝对值差: {:.6} GB", difference);
/// println!("百分比差: {:.2}%", percent_diff);
/// ```
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_conversion() {
        let bytes: u64 = 1_073_741_824; // 1 GB in binary
        assert!((bytes.to_gb_binary() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_decimal_conversion() {
        let bytes: u64 = 1_000_000_000; // 1 GB in decimal
        assert!((bytes.to_gb_decimal() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(1024.format_bytes(), "1.00 KB");
        assert_eq!(1_500_000.format_bytes(), "1.43 MB");
        assert_eq!(15_000_000_000.format_bytes(), "13.97 GB");
    }
}
