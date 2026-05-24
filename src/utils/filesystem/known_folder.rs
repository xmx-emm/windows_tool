use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::ptr;
use winapi::ctypes::c_void;
use winapi::um::combaseapi::CoTaskMemFree;
use winapi::um::knownfolders::FOLDERID_Documents;
use winapi::um::shlobj::SHGetKnownFolderPath;
use winapi::um::winnt::PWSTR;

pub fn get_documents_path() -> Option<OsString> {
    unsafe {
        let mut path_ptr: PWSTR = ptr::null_mut();
        let hr = SHGetKnownFolderPath(&FOLDERID_Documents, 0, ptr::null_mut(), &mut path_ptr);

        if hr == 0 && !path_ptr.is_null() {
            let mut len = 0;
            while *path_ptr.offset(len) != 0 {
                len += 1;
            }

            let wide_slice = std::slice::from_raw_parts(path_ptr, len as usize);
            let path = OsString::from_wide(wide_slice);

            CoTaskMemFree(path_ptr as *mut c_void);
            Some(path)
        } else {
            None
        }
    }
}
