//! Valve KeyValues（VDF）文本格式的解析、遍历与写回文件。
//!
//! 入口类型为 `VdfValue`；从字符串解析使用 `parse_vdf_string`（见本模块 `pub use`）。

use indexmap::IndexMap;

mod crud;
mod file;
mod value;

pub use file::get_steam_local_config_vdf_path_by_user_id;
pub use value::VdfValue;

// 改进的解析函数，处理更多边界情况
fn parse_vdf<I>(chars: &mut std::iter::Peekable<I>) -> Result<VdfValue, String>
where
    I: Iterator<Item = char> + Clone,
{
    let mut result = IndexMap::new();

    skip_whitespace_and_comments(chars);

    while chars.peek().is_some() {
        if let Some(&c) = chars.peek() {
            if c == '}' {
                chars.next(); // 消耗掉 '}'
                break;
            }

            if c == '"' {
                let key = parse_string(chars)?;
                skip_whitespace_and_comments(chars);

                if chars.peek() == Some(&'{') {
                    // 嵌套对象
                    chars.next(); // 跳过 '{'
                    let nested = parse_vdf(chars)?;
                    result.insert(key, Box::new(nested));
                } else if chars.peek() == Some(&'"') {
                    // 字符串值
                    let value = parse_string(chars)?;
                    result.insert(key, Box::from(VdfValue::String(value)));
                } else {
                    // 可能的值没有引号（非标准VDF但有些文件这样）
                    let value = parse_unquoted_string(chars)?;
                    result.insert(key, Box::from(VdfValue::String(value)));
                }

                skip_whitespace_and_comments(chars);
            } else if c == '/' {
                // 处理注释
                skip_comment(chars);
                skip_whitespace_and_comments(chars);
            } else if c.is_alphabetic() || c.is_numeric() || c == '_' {
                // 处理没有引号的键（非标准VDF但有些文件这样）
                let key = parse_unquoted_string(chars)?;
                skip_whitespace_and_comments(chars);

                if chars.peek() == Some(&'{') {
                    chars.next(); // 跳过 '{'
                    let nested = parse_vdf(chars)?;
                    result.insert(key, Box::from(nested));
                } else {
                    let value = parse_unquoted_string(chars)?;
                    result.insert(key, Box::from(VdfValue::String(value)));
                }

                skip_whitespace_and_comments(chars);
            } else {
                return Err(format!(
                    "Unexpected character: '{}' (0x{:02x})",
                    if c.is_control() {
                        format!("\\x{:02x}", c as u8)
                    } else {
                        c.to_string()
                    },
                    c as u8
                ));
            }
        }
    }

    Ok(VdfValue::Object(result))
}

// 合并空白和注释跳过
fn skip_whitespace_and_comments<I>(chars: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = char>,
    I: Clone,
{
    loop {
        skip_whitespace(chars);

        // 检查是否有注释
        if chars.peek() == Some(&'/') {
            let mut clone = chars.clone();
            clone.next(); // 跳过第一个 '/'
            if clone.peek() == Some(&'/') {
                skip_comment(chars);
                continue;
            }
        }

        break;
    }
}

fn parse_string<I>(chars: &mut std::iter::Peekable<I>) -> Result<String, String>
where
    I: Iterator<Item = char>,
{
    skip_whitespace(chars);

    if chars.next() != Some('"') {
        return Err("Expected opening quote".to_string());
    }

    let mut result = String::new();
    let mut escape_next = false;

    while let Some(&c) = chars.peek() {
        if escape_next {
            match c {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                'r' => result.push('\r'),
                '"' => result.push('"'),
                '\\' => result.push('\\'),
                _ => result.push(c), // 未知转义序列，直接添加字符
            }
            chars.next();
            escape_next = false;
        } else if c == '\\' {
            escape_next = true;
            chars.next();
        } else if c == '"' {
            chars.next(); // 跳过 closing quote
            return Ok(result);
        } else {
            result.push(chars.next().unwrap());
        }
    }

    Err("Unterminated string".to_string())
}

// 解析没有引号的字符串
fn parse_unquoted_string<I>(chars: &mut std::iter::Peekable<I>) -> Result<String, String>
where
    I: Iterator<Item = char>,
{
    let mut result = String::new();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() || c == '{' || c == '}' {
            break;
        }
        result.push(chars.next().unwrap());
    }

    if result.is_empty() {
        Err("Expected unquoted string".to_string())
    } else {
        Ok(result)
    }
}

fn skip_whitespace<I>(chars: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = char>,
{
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
}

fn skip_comment<I>(chars: &mut std::iter::Peekable<I>)
where
    I: Iterator<Item = char>,
{
    if chars.next() == Some('/') {
        if chars.peek() == Some(&'/') {
            chars.next(); // 跳过第二个 '/'
            // 跳过直到行尾
            while let Some(&c) = chars.peek() {
                if c == '\n' {
                    break;
                }
                chars.next();
            }
        }
    }
}

pub fn parse_vdf_string(input: &str) -> Result<VdfValue, String> {
    let mut chars = input.chars().peekable();
    parse_vdf(&mut chars)
}

#[cfg(test)]
mod tests_vdf {
    use crate::vdf::value::VdfValue;

    #[test]
    fn test_get_by_path() {
        let mut root = VdfValue::new_object();
        let mut nested = VdfValue::new_object();
        nested.insert("key", VdfValue::new_string("value")).unwrap();
        root.insert("nested", nested).unwrap();

        let path = ["nested", "key"];
        let result = root.get_by_path(&path);
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_string(), Some("value"));
    }

    #[test]
    fn test_path_exists() {
        let mut root = VdfValue::new_object();
        root.insert("key", VdfValue::new_string("value")).unwrap();

        assert!(root.path_exists(&["key"]));
        assert!(!root.path_exists(&["nonexistent"]));
    }

    #[test]
    fn test_set_by_path() {
        let mut root = VdfValue::new_object();
        let mut nested = VdfValue::new_object();
        nested
            .insert("old_key", VdfValue::new_string("old_value"))
            .unwrap();
        root.insert("nested", nested).unwrap();

        let path = ["nested", "new_key"];
        root.set_by_path(&path, VdfValue::new_string("new_value"))
            .unwrap();

        let result = root.get_by_path(&path);
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_string(), Some("new_value"));
    }
    
}
