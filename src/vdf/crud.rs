use crate::vdf::VdfValue;
use indexmap::IndexMap;

impl VdfValue {
    // 创建一个空的 VDF 对象
    pub fn new_object() -> Self {
        VdfValue::Object(IndexMap::new())
    }

    // 创建一个字符串值
    pub fn new_string(s: impl Into<String>) -> Self {
        VdfValue::String(s.into())
    }

    // 通过路径列表获取嵌套值
    pub fn get_value_by_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&str> {
        match self.get_by_path(path) {
            Some(vdf_value) => match vdf_value.as_string() {
                Some(value) => Some(value),
                None => None,
            },
            None => None,
        }
    }

    // 通过路径列表获取嵌套值
    pub fn get_by_path<T: AsRef<str>>(&self, path: &[T]) -> Option<&VdfValue> {
        if path.is_empty() {
            return Some(self);
        }

        let mut current = self;

        // 手动迭代路径列表
        let mut index = 0;
        while index < path.len() {
            let key = &path[index];

            if let VdfValue::Object(map) = current {
                current = match map.get(key.as_ref()) {
                    Some(value) => value,
                    None => return None,
                };
            } else {
                return None;
            }

            index += 1;
        }

        Some(current)
    }

    pub fn get<T: AsRef<str>>(&self, path: T) -> Option<&VdfValue> {
        self.get_by_path(&[path])
    }
    pub fn get_value<T: AsRef<str>>(&self, path: T) -> Option<&str> {
        self.get_value_by_path(&[path])
    }

    // 通过路径列表获取可变嵌套值
    pub fn get_mut_by_path<T: AsRef<str>>(&mut self, path: &[T]) -> Option<&mut VdfValue> {
        if path.is_empty() {
            return Some(self);
        }

        let mut current = self;

        // 手动迭代路径列表
        let mut index = 0;
        while index < path.len() {
            let key = &path[index];

            if let VdfValue::Object(map) = current {
                current = match map.get_mut(key.as_ref()) {
                    Some(value) => value,
                    None => return None,
                };
            } else {
                return None;
            }

            index += 1;
        }

        Some(current)
    }

    //设置值按路径
    pub fn set_value_by_path<T: AsRef<str>>(&mut self, path: &[T], value: T) -> Result<(), String> {
        self.set_by_path(path, VdfValue::String(value.as_ref().to_string()))
    }

    //设置嵌套对象按路径
    pub fn set_object_by_path<T: AsRef<str>>(
        &mut self,
        path: &[T],
        value: IndexMap<String, Box<VdfValue>>,
    ) -> Result<(), String> {
        self.set_by_path(path, VdfValue::Object(value))
    }

    // 通过路径列表设置值；中间缺失的对象节点会自动创建（例如首次写入某游戏的 LaunchOptions）。
    pub fn set_by_path<T: AsRef<str>>(
        &mut self,
        path: &[T],
        value: VdfValue,
    ) -> Result<(), String> {
        if path.is_empty() {
            *self = value;
            return Ok(());
        }

        let mut current = self;
        for (i, segment) in path.iter().enumerate() {
            let key = segment.as_ref();
            let is_leaf = i + 1 == path.len();

            let VdfValue::Object(map) = current else {
                return Err(format!(
                    "VDF path segment '{}' parent is not an object",
                    key
                ));
            };

            if is_leaf {
                map.insert(key.to_string(), Box::new(value));
                return Ok(());
            }

            if !map.contains_key(key) {
                map.insert(key.to_string(), Box::new(VdfValue::new_object()));
            }

            let child = map.get_mut(key).ok_or_else(|| {
                format!("VDF path segment '{}' missing after insert", key)
            })?;

            if !matches!(child.as_ref(), VdfValue::Object(_)) {
                return Err(format!(
                    "VDF path segment '{}' exists but is not an object",
                    key
                ));
            }

            current = child.as_mut();
        }

        Ok(())
    }

    // 检查路径是否存在
    pub fn path_exists(&self, path: &[&str]) -> bool {
        self.get_by_path(path).is_some()
    }

    // 获取字符串值（如果当前值是字符串）
    pub fn as_string(&self) -> Option<&str> {
        if let VdfValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    // 获取对象值（如果当前值是对象）
    pub fn as_object(&self) -> Option<&IndexMap<String, Box<VdfValue>>> {
        if let VdfValue::Object(map) = self {
            Some(map)
        } else {
            None
        }
    }

    // 插入键值对到对象中
    pub fn insert(&mut self, key: impl Into<String>, value: VdfValue) -> Result<(), String> {
        if let VdfValue::Object(map) = self {
            map.insert(key.into(), Box::new(value));
            Ok(())
        } else {
            Err("Not an object".to_string())
        }
    }
}
