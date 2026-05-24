use indexmap::IndexMap;

/// VDF 树中的节点：字符串叶子，或键到子节点的对象（顺序由 [`indexmap::IndexMap`] 保留）。
#[derive(Debug, PartialEq, Clone)]
pub enum VdfValue {
    String(String),
    Object(IndexMap<String, Box<VdfValue>>),
}

impl VdfValue {
    //将Vdf数据反回为字符串
    pub fn to_string(&self) -> String {
        self.vdf_to_string(0)
    }

    // 辅助函数：将 VDF 值转换为字符串表示
    fn vdf_to_string(&self, indent: usize) -> String {
        let mut result = String::new();

        let indent_str = "\t".repeat(indent);
        match self {
            VdfValue::String(s) => {
                result.push_str(&format!("\"{}\"", s));
            }
            VdfValue::Object(map) => {
                // let mut sorted: Vec<(&String, &Box<VdfValue>)> = map.iter().collect();
                // sorted.sort_by(|a, b| a.0.cmp(b.0));
                for (key, val) in map {
                    result.push_str(&format!("{}\"{}\"", indent_str, key));
                    match &**val {
                        VdfValue::String(s) => {
                            result.push_str(&format!("\t\t\"{}\"\n", escape_string(s)));
                        }
                        VdfValue::Object(_) => {
                            result.push_str(&format!("\n{}{{\n", indent_str));
                            result.push_str(&val.vdf_to_string(indent + 1).as_str());
                            result.push_str(&format!("{}}}\n", indent_str));
                        }
                    }
                }
            }
        }
        result
    }
}

// 转义字符串中的特殊字符
fn escape_string(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),  // 转义双引号
            '\\' => result.push_str("\\\\"), // 转义反斜杠
            '\n' => result.push_str("\\n"),  // 转义换行符
            '\r' => result.push_str("\\r"),  // 转义回车符
            '\t' => result.push_str("\\t"),  // 转义制表符
            _ => result.push(c),             // 其他字符直接添加
        }
    }
    result
}
