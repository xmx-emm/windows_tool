//! Apex 配置文件文档模型（行级保留，支持写回）。

use super::encoding::ApexFileEncoding;
use indexmap::IndexMap;

/// 单行类型：键值对、bind 行、原始行（注释/空行/花括号等）。
#[derive(Debug, Clone, PartialEq)]
pub enum ApexCfgLine {
    /// 原始文本行（含换行符前的内容，不含 `\n`）。
    Raw(String),
    /// `"key" "value"` 或 `key "value"` 形式。
    KeyValue {
        key: String,
        value: String,
        /// 行首缩进（制表符/空格）。
        prefix: String,
        /// key 与 value 之间的分隔（通常为 `\t\t`）。
        separator: String,
    },
}

impl ApexCfgLine {
    pub fn key(&self) -> Option<&str> {
        match self {
            ApexCfgLine::KeyValue { key, .. } => Some(key.as_str()),
            _ => None,
        }
    }

    pub fn value(&self) -> Option<&str> {
        match self {
            ApexCfgLine::KeyValue { value, .. } => Some(value.as_str()),
            _ => None,
        }
    }

    pub fn to_line_string(&self) -> String {
        match self {
            ApexCfgLine::Raw(s) => s.clone(),
            ApexCfgLine::KeyValue {
                key,
                value,
                prefix,
                separator,
            } => {
                if key.starts_with('"') {
                    format!("{prefix}{key}{separator}\"{value}\"")
                } else {
                    format!("{prefix}{key}{separator}\"{value}\"")
                }
            }
        }
    }
}

/// 解析后的 Apex 配置文件文档。
#[derive(Debug, Clone, PartialEq)]
pub struct ApexCfgDocument {
    pub lines: Vec<ApexCfgLine>,
    pub root_name: Option<String>,
    pub encoding: ApexFileEncoding,
}

impl ApexCfgDocument {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            root_name: None,
            encoding: ApexFileEncoding::Utf8,
        }
    }

    pub fn from_content(content: &str, encoding: ApexFileEncoding) -> Result<Self, String> {
        super::parse::parse_content(content, encoding)
    }

    pub fn to_string(&self) -> String {
        let mut out = String::new();
        for (i, line) in self.lines.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(&line.to_line_string());
        }
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out
    }

    /// 所有键值对的快照（后出现的同 key 覆盖前者）。
    pub fn key_values(&self) -> IndexMap<String, String> {
        let mut map = IndexMap::new();
        for line in &self.lines {
            if let ApexCfgLine::KeyValue { key, value, .. } = line {
                map.insert(key.clone(), value.clone());
            }
        }
        map
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.lines
            .iter()
            .rev()
            .find_map(|line| {
                if let ApexCfgLine::KeyValue { key: k, value, .. } = line {
                    if k == key || k.trim_matches('"') == key {
                        return Some(value.as_str());
                    }
                }
                None
            })
    }

    pub fn path_exists(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

impl Default for ApexCfgDocument {
    fn default() -> Self {
        Self::new()
    }
}
