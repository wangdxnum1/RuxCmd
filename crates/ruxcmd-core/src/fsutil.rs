//! Small filesystem helpers shared by the destructive commands.

/// Clears the read-only attribute on `path` (a no-op on other platforms or
/// when the attribute is not set). Used before `cp -f` / `unzip -o`
/// overwrite a read-only destination and before `rm`/`mv` replace it.
#[cfg(windows)]
pub fn clear_readonly(path: &std::path::Path) -> std::io::Result<()> {
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileAttributesW, SetFileAttributesW, FILE_ATTRIBUTE_READONLY,
    };

    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: wide is NUL-terminated; the API only reads it.
    let attrs = unsafe { GetFileAttributesW(wide.as_ptr()) };
    const INVALID_FILE_ATTRIBUTES: u32 = u32::MAX;
    if attrs == INVALID_FILE_ATTRIBUTES {
        return Ok(()); // missing or inaccessible: let the later write fail
    }
    if attrs & FILE_ATTRIBUTE_READONLY == 0 {
        return Ok(());
    }
    let cleared = attrs & !FILE_ATTRIBUTE_READONLY;
    // SAFETY: wide is NUL-terminated; the API only reads it.
    if unsafe { SetFileAttributesW(wide.as_ptr(), cleared) } == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn clear_readonly(path: &std::path::Path) -> std::io::Result<()> {
    let _ = path;
    Ok(())
}
