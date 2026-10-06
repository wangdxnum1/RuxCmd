mod cli;

use clap::Parser;
use std::fs::metadata;

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
                    print_fs_status(path, &meta);
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

fn print_default_status(path: &std::path::Path, meta: &std::fs::Metadata) {
    let path_str = path.to_string_lossy();
    let file_type = if meta.is_dir() {
        "directory"
    } else {
        "regular file"
    };
    let size = meta.len();
    let created = meta
        .created()
        .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH);
    let modified = meta
        .modified()
        .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH);
    let accessed = meta
        .accessed()
        .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH);

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

fn print_fs_status(path: &std::path::Path, _meta: &std::fs::Metadata) {
    let path_str = path.to_string_lossy();
    println!("  File: {}", path_str);
    println!("  ID: unknown");
    println!("  Type: unknown");
    println!("Block size: unknown");
    println!("Fundamental block size: unknown");
    println!("Blocks: unknown");
    println!("Free blocks: unknown");
    println!("Available blocks: unknown");
    println!("Inodes: unknown");
    println!("Free inodes: unknown");
    println!("UID: unknown");
    println!("GID: unknown");
    println!("Mount flags: unknown");
}

fn print_terse_status(path: &std::path::Path, meta: &std::fs::Metadata) {
    let file_type = if meta.is_dir() { 'd' } else { '-' };
    let size = meta.len();
    let modified = meta
        .modified()
        .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH);
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

fn print_custom_format(path: &std::path::Path, meta: &std::fs::Metadata, format: &str) {
    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = format.chars().collect();

    while i < chars.len() {
        if chars[i] == '%' && i + 1 < chars.len() {
            let next_char = chars[i + 1];
            let replacement = match next_char {
                '%' => "%".to_string(),
                'n' => path.to_string_lossy().to_string(),
                's' => meta.len().to_string(),
                't' => (if meta.is_dir() { 'd' } else { '-' }).to_string(),
                'b' => meta.len().to_string(),
                'f' => "0".to_string(),
                'g' => "0".to_string(),
                'u' => "0".to_string(),
                'i' => "0".to_string(),
                'h' => "1".to_string(),
                'D' => "0".to_string(),
                'm' => format_time(
                    meta.modified()
                        .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH),
                ),
                'c' => format_time(
                    meta.created()
                        .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH),
                ),
                'a' => format_time(
                    meta.accessed()
                        .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH),
                ),
                _ => format!("%{}", next_char),
            };
            result.push_str(&replacement);
            i += 2;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    println!("{}", result);
}

fn format_time(time: std::time::SystemTime) -> String {
    match time.duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs() as i64;
            let tm = time::OffsetDateTime::from_unix_timestamp(secs)
                .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);
            let format = time::format_description::parse_borrowed::<1>(
                "[year]-[month]-[day] [hour]:[minute]:[second]",
            )
            .unwrap();
            tm.format(&format).unwrap_or_else(|_| "Unknown".to_string())
        }
        Err(_) => "Unknown".to_string(),
    }
}
