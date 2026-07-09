//! Apex 配置文件解析。

use super::encoding::ApexFileEncoding;
use super::value::{ApexCfgDocument, ApexCfgLine};

pub fn parse_content(content: &str, encoding: ApexFileEncoding) -> Result<ApexCfgDocument, String> {
    let mut doc = ApexCfgDocument::new();
    doc.encoding = encoding;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            doc.lines.push(ApexCfgLine::Raw(line.to_string()));
            continue;
        }
        if trimmed == "{" || trimmed == "}" {
            doc.lines.push(ApexCfgLine::Raw(line.to_string()));
            continue;
        }
        // 根节点名: "VideoConfig"
        if trimmed.starts_with('"')
            && trimmed.ends_with('"')
            && !trimmed.contains('\t')
            && trimmed.len() > 2
        {
            let name = trimmed.trim_matches('"').to_string();
            if doc.root_name.is_none() {
                doc.root_name = Some(name.clone());
            }
            doc.lines.push(ApexCfgLine::Raw(line.to_string()));
            continue;
        }
        // bind 行: bind_US_standard "1" "weaponSelectPrimary0" 0
        if trimmed.starts_with("bind") {
            doc.lines.push(ApexCfgLine::Raw(line.to_string()));
            continue;
        }
        // key "value" 或 "key" "value"
        if let Some(kv) = parse_key_value_line(line) {
            doc.lines.push(kv);
        } else {
            doc.lines.push(ApexCfgLine::Raw(line.to_string()));
        }
    }
    Ok(doc)
}

fn parse_key_value_line(line: &str) -> Option<ApexCfgLine> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed == "{" || trimmed == "}" {
        return None;
    }

    let (prefix, rest) = split_prefix(line);
    let rest = rest.trim_start();
    if rest.is_empty() {
        return None;
    }

    let quoted = extract_quoted_segments(rest);
    if quoted.is_empty() {
        return None;
    }

    let (key, value, separator) = if quoted.len() >= 2 {
        let key = format!("\"{}\"", quoted[0]);
        let value = quoted[1].clone();
        let sep = separator_between_first_two_quotes(rest, &quoted[0], &quoted[1]);
        (key, value, sep)
    } else if let Some(first_quote) = rest.find('"') {
        let key = rest[..first_quote].trim().to_string();
        if key.is_empty() {
            return None;
        }
        let value = quoted[0].clone();
        let sep = rest[first_quote..]
            .find(&format!("\"{}\"", value))
            .map(|i| rest[first_quote..first_quote + i].to_string())
            .unwrap_or_else(|| " ".to_string());
        (key, value, sep)
    } else {
        return None;
    };

    Some(ApexCfgLine::KeyValue {
        key,
        value,
        prefix,
        separator,
    })
}

/// 提取行内所有引号包裹的片段（不含引号）。
fn extract_quoted_segments(s: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'"' {
            i += 1;
            continue;
        }
        let start = i + 1;
        i += 1;
        while i < bytes.len() && bytes[i] != b'"' {
            i += 1;
        }
        if i < bytes.len() {
            segments.push(
                std::str::from_utf8(&bytes[start..i])
                    .unwrap_or_default()
                    .to_string(),
            );
            i += 1;
        } else {
            break;
        }
    }
    segments
}

fn separator_between_first_two_quotes(
    rest: &str,
    first: &str,
    second: &str,
) -> String {
    let first_end = rest
        .find(&format!("\"{first}\""))
        .map(|i| i + first.len() + 2)
        .unwrap_or(0);
    let second_start = rest[first_end..]
        .find(&format!("\"{second}\""))
        .map(|i| first_end + i)
        .unwrap_or(first_end);
    rest[first_end..second_start].to_string()
}

fn split_prefix(line: &str) -> (String, &str) {
    let trimmed = line.trim_start();
    let prefix_len = line.len() - trimmed.len();
    (line[..prefix_len].to_string(), trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_videoconfig_sample() {
        let sample = r#""VideoConfig"
{
	"setting.defaultres"		"1920"
	"setting.defaultresheight"		"1080"
}
"#;
        let doc = parse_content(sample, ApexFileEncoding::Utf8).unwrap();
        assert_eq!(doc.root_name.as_deref(), Some("VideoConfig"));
        assert_eq!(doc.get("setting.defaultres"), Some("1920"));
        assert_eq!(doc.get("setting.defaultresheight"), Some("1080"));
    }

    #[test]
    fn parse_profile_kv() {
        let sample = r#"cl_fovScale "1.55"
mouse_sensitivity "0.996247"
"#;
        let doc = parse_content(sample, ApexFileEncoding::Utf8).unwrap();
        assert_eq!(doc.get("cl_fovScale"), Some("1.55"));
    }
}
