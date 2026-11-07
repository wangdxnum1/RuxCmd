//! Per-physical-drive I/O counters shared by the sysstat-style tools
//! (`iostat -d`, `sar -d`), read through
//! `IOCTL_DISK_PERFORMANCE` on each `\\.\PhysicalDriveN`.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows_sys::Win32::System::Ioctl::{DISK_PERFORMANCE, IOCTL_DISK_PERFORMANCE};
use windows_sys::Win32::System::IO::DeviceIoControl;

/// One drive's counters at a moment in time.
#[derive(Debug, Clone, PartialEq)]
pub struct DriveStats {
    /// `PhysicalDrive0` style name, matching the \\.\ device opened.
    pub name: String,
    pub read_bytes: u64,
    pub write_bytes: u64,
    pub read_count: u64,
    pub write_count: u64,
}

/// Snapshot of every physical drive that answers IOCTL_DISK_PERFORMANCE.
/// Drives without the performance counters enabled (the ioctl fails) are
/// skipped, mirroring how sysstat only lists devices it can query.
pub fn snapshot() -> Vec<DriveStats> {
    let mut drives = Vec::new();
    for index in 0..16u32 {
        let name = format!("\\\\.\\PhysicalDrive{index}");
        match drive_stats(&name) {
            Some(stats) => drives.push(stats),
            // The drive either does not exist (ENOENT) or has no counters;
            // probing past the last existing drive finds nothing, so stop.
            None if index > 0 && !drive_exists(index) => break,
            None => continue,
        }
    }
    drives
}

fn drive_exists(index: u32) -> bool {
    let name = format!("\\\\.\\PhysicalDrive{index}");
    let wide: Vec<u16> = OsStr::new(&name).encode_wide().chain([0]).collect();
    // SAFETY: `wide` is NUL-terminated; the handle is closed immediately.
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    };
    if handle.is_null() || handle as isize == -2 {
        // INVALID_HANDLE_VALUE (-1) and ERROR_FILE_NOT_FOUND handles.
        return false;
    }
    unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
    true
}

/// Queries one `\\.\PhysicalDriveN` device for its current counters.
fn drive_stats(device: &str) -> Option<DriveStats> {
    let wide: Vec<u16> = OsStr::new(device).encode_wide().chain([0]).collect();
    // SAFETY: `wide` is NUL-terminated; `stats` is a valid out-buffer for the
    // ioctl and `returned` a valid out-pointer.
    unsafe {
        let handle = CreateFileW(
            wide.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        if handle.is_null() || handle as isize == -1 {
            return None;
        }
        let mut stats = DISK_PERFORMANCE::default();
        let mut returned = 0u32;
        let ok = DeviceIoControl(
            handle,
            IOCTL_DISK_PERFORMANCE,
            std::ptr::null(),
            0,
            &mut stats as *mut DISK_PERFORMANCE as *mut core::ffi::c_void,
            std::mem::size_of::<DISK_PERFORMANCE>() as u32,
            &mut returned,
            std::ptr::null_mut(),
        );
        windows_sys::Win32::Foundation::CloseHandle(handle);
        if ok == 0 {
            return None;
        }
        Some(DriveStats {
            // Display the name without the \\.\ prefix, the way iostat lists
            // devices.
            name: device.trim_start_matches("\\\\.\\").to_string(),
            read_bytes: stats.BytesRead as u64,
            write_bytes: stats.BytesWritten as u64,
            read_count: stats.ReadCount as u64,
            write_count: stats.WriteCount as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On a real machine at least one physical drive answers; the names keep
    /// the PhysicalDriveN form.  CI runners have exactly one system drive.
    #[test]
    fn snapshot_lists_physical_drives() {
        let drives = snapshot();
        assert!(!drives.is_empty(), "no physical drives found");
        for drive in &drives {
            assert!(drive.name.starts_with("PhysicalDrive"), "{drive:?}");
        }
        assert_eq!(drives[0].name, "PhysicalDrive0");
    }
}
