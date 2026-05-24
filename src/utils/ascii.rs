pub trait Str {
    fn is_ascii_digit(&self) -> bool;
}

impl Str for String {
    /// 是 ascii 数字
    fn is_ascii_digit(&self) -> bool {
        for c in self.chars() {
            if !c.is_ascii_digit() {
                return false;
            }
        }
        true
    }
}
