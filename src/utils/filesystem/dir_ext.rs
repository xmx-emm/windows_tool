use std::path::PathBuf;

pub trait CreateDir {
    /// 创建文件夹
    fn create_dir(&self, create_dir: bool) -> Option<bool>;
}

impl CreateDir for PathBuf {
    fn create_dir(&self, create_dir: bool) -> Option<bool> {
        if !self.exists() {
            if create_dir {
                return match std::fs::create_dir_all(self) {
                    Ok(_) => {
                        println!("create dir finished {}", self.display());
                        Some(self.exists())
                    }
                    Err(_) => {
                        println!("Create Dir error {}", self.display());
                        None
                    }
                };
            }
            return None;
        }
        Some(true)
    }
}
