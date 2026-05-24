///
/// ```
/// println!("test check_ipv4_by_string");
/// use windows_tool::utils::check_ipv4_by_string;
/// assert!(check_ipv4_by_string(&"127.0.0.1".to_string()));
/// assert!(check_ipv4_by_string(&"127.0.0.15".to_string()));
/// assert!(!check_ipv4_by_string(&"asef.asef.0.1".to_string()));
/// assert!(!check_ipv4_by_string(&"12.0.0".to_string()));
/// ```
pub fn check_ipv4_by_string<T: AsRef<str>>(address: T) -> bool {
    let split = address.as_ref().split('.').collect::<Vec<&str>>();
    for x in &split {
        if x.parse::<u8>().is_err() {
            return false;
        }
    }
    split.len() == 4
}
