mod cli;

use clap::Parser;
use std::fs::metadata;
use std::os::windows::ffi::OsStrExt;
use std::path::{Component, Path, Prefix};
use std::time::{SystemTime, UNIX_EPOCH};
use time::{OffsetDateTime, UtcOffset};
use windows_sys::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetDiskFreeSpaceW};

/// Size of the blocks reported by `%b` and by the `-f` block lines. GNU stat
/// counts in 512-byte units regardless of the file system block size.
const GNU_BLOCK_SIZE: u64 = 512;

const HUMAN_TIME_FORMAT: &str =
    "[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour sign:mandatory][offset_minute]";

fn main() {
    let args = cli::Args::parse();

    if args.files.is_empty() {
        eprintln!("stat: missing operand");
        std::process::exit(1);
    }

    let mut exit_code = 0i32;

    for path in &args.files {
        match metadata(path) {
            Ok(meta) => {
                if args.file_system {
                    if !print_fs_status(path) {
                        exit_code = 1;
                    }
                } else if args.terse {
                    print_terse_status(path, &meta);
                } else if let Some(format) = &args.format {
                    print_custom_format(path, &meta, format);
                } else {
                    print_default_status(path, &meta);
                }
            }
            Err(e) => {
                eprintln!("stat: cannot stat '{}': {}", path.display(), e);
                exit_code = 1;
            }
        }
    }

    std::process::exit(exit_code);
}

fn print_default_status(path: &Path, meta: &std::fs::Metadata) {
    let path_str = path.to_string_lossy();
    let file_type = if meta.is_dir() {
        "directory"
    } else {
        "regular file"
    };
    let size = meta.len();
    let created = meta.created().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    let modified = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    let accessed = meta.accessed().unwrap_or(std::time::SystemTime::UNIX_EPOCH);

    println!("  File: {}", path_str);
    println!("  Size: {:>12} bytes", size);
    println!("  Type: {}", file_type);
    println!("Device: {:>12}", 0);
    println!("Inode: {:>12}", 0);
    println!("Links: {:>12}", 1);
    println!("Access: {:?}", meta.permissions());
    println!("Uid: {:>12}", 0);
    println!("Gid: {:>12}", 0);
    println!("Created: {}", format_time(created));
    println!("Modified: {}", format_time(modified));
    println!("Accessed: {}", format_time(accessed));
}

fn print_fs_status(path: &Path) -> bool {
    let root = volume_root(path);
    let root_wide: Vec<u16> = std::ffi::OsStr::new(&root)
        .encode_wide()
        .chain(Some(0))
        .collect();

    let mut sectors_per_cluster: u32 = 0;
    let mut bytes_per_sector: u32 = 0;
    let mut free_clusters: u32 = 0;
    let mut total_clusters: u32 = 0;
    let geometry_ok = unsafe {
        GetDiskFreeSpaceW(
            root_wide.as_ptr(),
            &mut sectors_per_cluster,
            &mut bytes_per_sector,
            &mut free_clusters,
            &mut total_clusters,
        )
    } != 0;

    let mut available_bytes: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut free_bytes: u64 = 0;
    let space_ok = unsafe {
        GetDiskFreeSpaceExW(
            root_wide.as_ptr(),
            &mut available_bytes,
            &mut total_bytes,
            &mut free_bytes,
        )
    } != 0;

    let block_size = u64::from(sectors_per_cluster) * u64::from(bytes_per_sector);
    if !geometry_ok || !space_ok || block_size == 0 {
        eprintln!(
            "stat: cannot read file system status for '{}'",
            path.display()
        );
        return false;
    }

    println!("  File: {}", path.to_string_lossy());
    println!(
        "Block size: {:<10} Fundamental block size: {}",
        block_size, block_size
    );
    println!(
        "Blocks: Total: {}    Free: {}    Available: {}",
        total_bytes / block_size,
        free_bytes / block_size,
        available_bytes / block_size
    );
    true
}

fn print_terse_status(path: &Path, meta: &std::fs::Metadata) {
    let file_type = if meta.is_dir() { 'd' } else { '-' };
    let size = meta.len();
    let modified = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
    let mtime = modified
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    println!(
        "{} {} {} {} {} {} {} {}",
        file_type,
        "rwxrwxrwx",
        1,
        0,
        0,
        size,
        mtime,
        path.to_string_lossy()
    );
}

fn print_custom_format(path: &Path, meta: &std::fs::Metadata, format: &str) {
    let mut result = String::new();
    let chars: Vec<char> = format.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] != '%' {
            result.push(chars[i]);
            i += 1;
            continue;
        }
        if i + 1 >= chars.len() {
            // Trailing lone '%' is emitted literally.
            result.push('%');
            break;
        }

        let directive = chars[i + 1];
        let replacement = match directive {
            '%' => "%".to_string(),
            'n' => path.to_string_lossy().to_string(),
            's' => meta.len().to_string(),
            'b' => meta.len().div_ceil(GNU_BLOCK_SIZE).to_string(),
            'B' => GNU_BLOCK_SIZE.to_string(),
            // Windows has no usable device numbers; 0 is the declared placeholder.
            'd' | 'D' | 't' | 'T' => "0".to_string(),
            'm' => volume_root(path),
            'h' => "1".to_string(),
            'i' => "0".to_string(),
            'u' | 'g' => "0".to_string(),
            'F' => file_type_name(meta).to_string(),
            'A' => permission_string(meta),
            'a' => format!("{:o}", octal_permissions(meta)),
            'f' => raw_mode(meta).to_string(),
            'W' => epoch_string(meta.created()),
            'X' => epoch_string(meta.accessed()),
            'Y' => epoch_string(meta.modified()),
            // Windows exposes no change time; fall back to the modification time.
            'Z' => epoch_string(meta.modified()),
            'x' => format_time_or_unknown(meta.accessed()),
            'y' => format_time_or_unknown(meta.modified()),
            'z' => format_time_or_unknown(meta.modified()),
            _ => {
                eprintln!("stat: invalid directive '%{}'", directive);
                std::process::exit(1);
            }
        };
        result.push_str(&replacement);
        i += 2;
    }

    println!("{result}");
}

fn file_type_name(meta: &std::fs::Metadata) -> &'static str {
    if meta.is_dir() {
        "directory"
    } else if meta.is_symlink() {
        "symbolic link"
    } else {
        "regular file"
    }
}

/// Permission approximation: Windows ACLs are not POSIX bits, so this reports
/// full access for writable entries and read-only access for read-only ones.
fn permission_string(meta: &std::fs::Metadata) -> String {
    let type_char = if meta.is_dir() { 'd' } else { '-' };
    let perms = if meta.permissions().readonly() {
        "r--r--r--"
    } else if meta.is_dir() {
        "rwxrwxrwx"
    } else {
        "rw-rw-rw-"
    };
    format!("{}{}", type_char, perms)
}

fn octal_permissions(meta: &std::fs::Metadata) -> u32 {
    if meta.permissions().readonly() {
        0o444
    } else if meta.is_dir() {
        0o777
    } else {
        0o666
    }
}

/// Raw mode in hex: POSIX type bits (S_IFDIR/S_IFREG) plus the approximated
/// permission bits.
fn raw_mode(meta: &std::fs::Metadata) -> String {
    let type_bits: u32 = if meta.is_dir() { 0o040000 } else { 0o100000 };
    format!("{:x}", type_bits | octal_permissions(meta))
}

fn epoch_string(stamp: std::io::Result<SystemTime>) -> String {
    let secs = stamp
        .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
        .unwrap_or(0);
    secs.to_string()
}

fn format_time_or_unknown(stamp: std::io::Result<SystemTime>) -> String {
    match stamp {
        Ok(time) => format_time(time),
        Err(_) => "Unknown".to_string(),
    }
}

fn format_time(time: SystemTime) -> String {
    let secs = match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as i64,
        Err(_) => return "Unknown".to_string(),
    };
    let datetime = OffsetDateTime::from_unix_timestamp(secs).unwrap_or(OffsetDateTime::UNIX_EPOCH);
    let local = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
    let format = time::format_description::parse_borrowed::<1>(HUMAN_TIME_FORMAT).unwrap();
    datetime
        .to_offset(local)
        .format(&format)
        .unwrap_or("Unknown".to_string())
}

/// Mount point of `path`: the volume root (drive letter root or UNC share),
/// which is the closest Windows equivalent of a mount point.
fn volume_root(path: &Path) -> String {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    };
    match absolute.components().next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => {
                format!("{}:\\", (letter as char).to_ascii_uppercase())
            }
            Prefix::UNC(server, share) | Prefix::VerbatimUNC(server, share) => {
                format!(
                    "\\\\{}\\{}",
                    server.to_string_lossy(),
                    share.to_string_lossy()
                )
            }
            _ => "\\".to_string(),
        },
        _ => "\\".to_string(),
    }
}
