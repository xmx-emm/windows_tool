use std::fs;
use std::path::Path;

//复制文件夹内所有内容
/*
 * Copy a directory recursively
 */
pub fn copy_dir_all<T: AsRef<Path>>(src: T, dst: T) -> Result<(), Box<dyn std::error::Error>> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
