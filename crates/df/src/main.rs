mod cli;

use clap::Parser;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetLogicalDriveStringsW};

fn main() {
    let args = cli::Args::parse();

    let mount_points = if args.paths.is_empty() {
        get_all_mount_points()
    } else {
        args.paths
    };

    println!("Filesystem\tSize\tUsed\tAvailable\tUse%\tMounted on");

    for path in mount_points {
        if let Some(stats) = get_disk_stats(&path) {
            let percent = if stats.total > 0 {
                (stats.used as f64 / stats.total as f64 * 100.0) as u64
            } else {
                0
            };

            println!(
                "{}\t{}\t{}\t{}\t{}%\t{}",
                stats.fs_type,
                format_size(stats.total, args.human_readable),
                format_size(stats.used, args.human_readable),
                format_size(stats.available, args.human_readable),
                percent,
                path.display()
            );
        }
    }
}

struct DiskStats {
    fs_type: String,
    total: u64,
    used: u64,
    available: u64,
}

#[cfg(windows)]
fn get_disk_stats(path: &Path) -> Option<DiskStats> {
    let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();

    let mut free_bytes_available: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free_bytes: u64 = 0;

    let result = unsafe {
        GetDiskFreeSpaceExW(
            path_wide.as_ptr(),
            &mut free_bytes_available,
            &mut total_bytes,
            &mut total_free_bytes,
        )
    };

    if result == 0 {
        return None;
    }

    let used = total_bytes.saturating_sub(total_free_bytes);

    Some(DiskStats {
        fs_type: "NTFS".to_string(),
        total: total_bytes,
        used,
        available: free_bytes_available,
    })
}

#[cfg(unix)]
fn get_disk_stats(path: &Path) -> Option<DiskStats> {
    None
}

#[cfg(windows)]
fn get_all_mount_points() -> Vec<PathBuf> {
    use std::mem;

    let mut buf: [u16; 1024] = unsafe { mem::zeroed() };
    let len = unsafe { GetLogicalDriveStringsW(buf.len() as u32 - 1, buf.as_mut_ptr()) };

    let mut drives = Vec::new();
    let mut i = 0;

    while i < len as usize {
        let mut end = i;
        while end < len as usize && buf[end] != 0 {
            end += 1;
        }

        if end > i {
            let wide: Vec<u16> = buf[i..end].to_vec();
            if let Ok(s) = String::from_utf16(&wide) {
                drives.push(PathBuf::from(s));
            }
        }

        i = end + 1;
        if buf[i] == 0 {
            break;
        }
    }

    drives
}

#[cfg(unix)]
fn get_all_mount_points() -> Vec<PathBuf> {
    vec![PathBuf::from("/")]
}

fn format_size(bytes: u64, human_readable: bool) -> String {
    if !human_readable {
        return (bytes / 1024).to_string();
    }

    let units = ["KB", "MB", "GB", "TB"];
    let mut size = bytes as f64 / 1024.0;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < units.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{:.0}", size)
    } else {
        format!("{:.1}{}", size, units[unit_idx])
    }
}
