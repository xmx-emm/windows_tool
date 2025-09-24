// 定义一个 trait 来包含我们的扩展方法
pub trait StringExt {
    /// 检查字符串是否全部由 ASCII 数字组成
    fn is_all_digits(&self) -> bool;

    /// 检查字符串是否全部由数字组成（包括 Unicode 数字）
    fn is_all_digits_unicode(&self) -> bool;

    /// 检查字符串是否表示有效的整数（可选支持符号）
    fn is_valid_integer(&self, allow_sign: bool) -> bool;

    /// 检查字符串是否表示有效的浮点数（可选支持符号和小数点）
    fn is_valid_float(&self, allow_sign: bool) -> bool;

    /// 尝试将字符串解析为 i64（如果它是有效的整数）
    fn to_i64(&self) -> Option<i64>;

    /// 尝试将字符串解析为 f64（如果它是有效的浮点数）
    fn to_f64(&self) -> Option<f64>;
}

// 为所有实现了 AsRef<str> 的类型实现这个 trait
impl<T: AsRef<str>> StringExt for T {
    fn is_all_digits(&self) -> bool {
        let s = self.as_ref();

        // 空字符串不算全数字
        if s.is_empty() {
            return false;
        }

        // 检查每个字符是否为 ASCII 数字
        s.chars().all(|c| c.is_ascii_digit())
    }

    fn is_all_digits_unicode(&self) -> bool {
        let s = self.as_ref();

        // 空字符串不算全数字
        if s.is_empty() {
            return false;
        }

        // 检查每个字符是否为数字（包括 Unicode 数字字符）
        s.chars().all(|c| c.is_numeric())
    }

    fn is_valid_integer(&self, allow_sign: bool) -> bool {
        let s = self.as_ref();

        // 空字符串无效
        if s.is_empty() {
            return false;
        }

        let mut chars = s.chars();

        // 处理可选符号
        if allow_sign {
            if let Some(first) = chars.next() {
                if first == '+' || first == '-' {
                    // 如果只有符号，没有数字，则无效
                    if s.len() == 1 {
                        return false;
                    }
                } else if !first.is_ascii_digit() {
                    return false;
                }
            }
        }

        // 检查剩余字符是否都是数字
        chars.all(|c| c.is_ascii_digit())
    }

    fn is_valid_float(&self, allow_sign: bool) -> bool {
        let s = self.as_ref();

        // 空字符串无效
        if s.is_empty() {
            return false;
        }

        let mut chars = s.chars().peekable();
        let mut has_dot = false;

        // 处理可选符号
        if allow_sign {
            if let Some(&first) = chars.peek() {
                if first == '+' || first == '-' {
                    chars.next();
                    // 如果只有符号，没有数字，则无效
                    if chars.peek().is_none() {
                        return false;
                    }
                }
            }
        }

        // 检查每个字符
        while let Some(c) = chars.next() {
            if c == '.' {
                // 只能有一个小数点
                if has_dot {
                    return false;
                }
                has_dot = true;

                // 小数点前后必须有数字
                if chars.peek().is_none() {
                    return false; // 小数点后没有数字
                }
            } else if !c.is_ascii_digit() {
                return false;
            }
        }

        true
    }

    fn to_i64(&self) -> Option<i64> {
        let s = self.as_ref();

        // 检查是否是有效的整数（允许符号）
        if self.is_valid_integer(true) {
            s.parse().ok()
        } else {
            None
        }
    }

    fn to_f64(&self) -> Option<f64> {
        let s = self.as_ref();

        // 检查是否是有效的浮点数（允许符号）
        if self.is_valid_float(true) {
            s.parse().ok()
        } else {
            None
        }
    }
}

// 使用示例
#[test]
fn test_string() {
    // 导入 trait 来启用扩展方法
    use StringExt;

    let test_strings = [
        "12345",
        "0",
        "",
        "123a45",
        "12.34",
        "-123",
        "+456",
        "3.14",
        "-2.718",
    ];

    println!("=== 字符串扩展方法示例 ===");

    for s in &test_strings {
        println!("字符串: '{}'", s);
        println!("  is_all_digits(): {}", s.is_all_digits());
        println!("  is_all_digits_unicode(): {}", s.is_all_digits_unicode());
        println!("  is_valid_integer(true): {}", s.is_valid_integer(true));
        println!("  is_valid_float(true): {}", s.is_valid_float(true));
        println!("  to_i64(): {:?}", s.to_i64());
        println!("  to_f64(): {:?}", s.to_f64());
        println!();
    }

    // 使用字符串字面量直接调用方法
    println!("直接调用示例:");
    println!("'12345'.is_all_digits() = {}", "12345".is_all_digits());
    println!("'12.34'.to_f64() = {:?}", "12.34".to_f64());

    // 使用 String 类型调用方法
    let s = String::from("-42");
    println!("String 类型调用: s.is_valid_integer(true) = {}", s.is_valid_integer(true));
    println!("String 类型调用: s.to_i64() = {:?}", s.to_i64());
}

#[cfg(test)]
mod tests_string_ext {
    use super::*;

    #[test]
    fn test_is_all_digits() {
        assert!("12345".is_all_digits());
        assert!(!"123a45".is_all_digits());
        assert!(!"".is_all_digits());
    }

    #[test]
    fn test_to_i64() {
        assert_eq!("123".to_i64(), Some(123));
        assert_eq!("-456".to_i64(), Some(-456));
        assert_eq!("12.34".to_i64(), None);
        assert_eq!("abc".to_i64(), None);
    }
}