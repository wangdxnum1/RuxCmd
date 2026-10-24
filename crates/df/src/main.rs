mod cli;

use clap::Parser;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetLogicalDriveStringsW, GetVolumeInformationW,
};

fn main() {
    let args = cli::Args::parse();

    let mount_points = if args.paths.is_empty() {
        get_all_mount_points()
    } else {
        args.paths.clone()
    };

    if args.print_type {
        println!("Filesystem	Type	Size	Used	Available	Use%	Mounted on");
    } else {
        println!("Filesystem	Size	Used	Available	Use%	Mounted on");
    }

    let mut failed = false;
    for path in &mount_points {
        match get_disk_stats(path) {
            Some(stats) => {
                let percent = if stats.total > 0 {
                    // POSIX rounds usage up, so a 1-byte file on an empty disk
                    // already reports 1%.
                    ((stats.used as f64 / stats.total as f64) * 100.0).ceil() as u64
                } else {
                    0
                };
                let device = path_to_device(path);
                if args.print_type {
                    println!(
                        "{}	{}	{}	{}	{}	{}%	{}",
                        device,
                        stats.fs_type,
                        format_size(stats.total, args.human_readable),
                        format_size(stats.used, args.human_readable),
                        format_size(stats.available, args.human_readable),
                        percent,
                        path.display()
                    );
                } else {
                    println!(
                        "{}	{}	{}	{}	{}%	{}",
                        device,
                        format_size(stats.total, args.human_readable),
                        format_size(stats.used, args.human_readable),
                        format_size(stats.available, args.human_readable),
                        percent,
                        path.display()
                    );
                }
            }
            None => {
                failed = true;
                eprintln!("df: {}: cannot get filesystem statistics", path.display());
            }
        }
    }
    if failed {
        std::process::exit(1);
    }
}

/// Display form of the mount point: `C:\` stays `C:`, UNC keeps `\server\share`.
fn path_to_device(path: &Path) -> String {
    let text = path.to_string_lossy();
    text.trim_end_matches(['\\', '/']).to_string()
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
    let fs_type = volume_file_system(path).unwrap_or_else(|| "unknown".to_string());

    Some(DiskStats {
        fs_type,
        total: total_bytes,
        used,
        available: free_bytes_available,
    })
}

/// Real file-system name via GetVolumeInformationW (NTFS, FAT32, exFAT, ...).
#[cfg(windows)]
fn volume_file_system(path: &Path) -> Option<String> {
    let text = path.to_string_lossy().to_string();
    // GetVolumeInformationW needs a trailing slash on drive roots ("C:\").
    let root = if text.len() == 2
        && text.as_bytes()[1] == b':'
        && text.as_bytes()[0].is_ascii_alphabetic()
    {
        format!("{}\\", text)
    } else {
        text
    };
    let wide: Vec<u16> = root.encode_utf16().chain(Some(0)).collect();
    let mut fs_buf = [0u16; 32];
    let ok = unsafe {
        GetVolumeInformationW(
            wide.as_ptr(),
            std::ptr::null_mut(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            fs_buf.as_mut_ptr(),
            32,
        )
    };
    if ok == 0 {
        return None;
    }
    let len = fs_buf.iter().position(|&c| c == 0).unwrap_or(fs_buf.len());
    if len == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&fs_buf[..len]))
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
