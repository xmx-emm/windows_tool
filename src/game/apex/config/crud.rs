//! Apex 配置文件 CRUD。

use super::value::{ApexCfgDocument, ApexCfgLine};

impl ApexCfgDocument {
    /// 设置或更新键值；不存在则在 `}` 前插入。
    pub fn set(&mut self, key: &str, value: impl Into<String>) -> Result<(), String> {
        let value = value.into();
        let quoted_key = normalize_key(key);

        for line in self.lines.iter_mut() {
            if let ApexCfgLine::KeyValue { key: k, value: v, .. } = line {
                if keys_equal(k, &quoted_key) {
                    *v = value;
                    return Ok(());
                }
            }
        }

        self.insert_before_close_brace(quoted_key, value)
    }

    pub fn set_many(&mut self, updates: &[(String, String)]) -> Result<(), String> {
        for (k, v) in updates {
            self.set(k, v.clone())?;
        }
        Ok(())
    }

    pub fn remove(&mut self, key: &str) -> Result<bool, String> {
        let quoted_key = normalize_key(key);
        let before = self.lines.len();
        self.lines.retain(|line| {
            !matches!(
                line,
                ApexCfgLine::KeyValue { key: k, .. } if keys_equal(k, &quoted_key)
            )
        });
        Ok(self.lines.len() < before)
    }

    pub fn insert(&mut self, key: &str, value: impl Into<String>) -> Result<(), String> {
        let quoted_key = normalize_key(key);
        let value = value.into();
        if self.path_exists(key) {
            return self.set(key, value);
        }
        self.insert_before_close_brace(quoted_key, value)
    }

    fn insert_before_close_brace(&mut self, key: String, value: String) -> Result<(), String> {
        let new_line = ApexCfgLine::KeyValue {
            key,
            value,
            prefix: "\t".to_string(),
            separator: "\t\t".to_string(),
        };

        if let Some(idx) = self.lines.iter().rposition(|l| {
            matches!(l, ApexCfgLine::Raw(s) if s.trim() == "}")
        }) {
            self.lines.insert(idx, new_line);
        } else {
            self.lines.push(new_line);
        }
        Ok(())
    }
}

fn normalize_key(key: &str) -> String {
    if key.starts_with('"') {
        key.to_string()
    } else {
        format!("\"{key}\"")
    }
}

fn keys_equal(a: &str, b: &str) -> bool {
    a.trim_matches('"') == b.trim_matches('"')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::apex::config::encoding::ApexFileEncoding;

    #[test]
    fn set_updates_existing() {
        let sample = r#""VideoConfig"
{
	"setting.defaultres"		"1280"
}
"#;
        let mut doc = ApexCfgDocument::from_content(sample, ApexFileEncoding::Utf8).unwrap();
        doc.set("setting.defaultres", "1920").unwrap();
        assert_eq!(doc.get("setting.defaultres"), Some("1920"));
        let out = doc.to_string();
        assert!(out.contains("\"1920\""));
        assert!(!out.contains("\"1280\""));
    }

    #[test]
    fn remove_key() {
        let sample = r#""VideoConfig"
{
	"setting.defaultres"		"1920"
}
"#;
        let mut doc = ApexCfgDocument::from_content(sample, ApexFileEncoding::Utf8).unwrap();
        assert!(doc.remove("setting.defaultres").unwrap());
        assert!(doc.get("setting.defaultres").is_none());
    }
}
