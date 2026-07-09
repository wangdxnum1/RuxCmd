mod cli;

use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args = cli::Args::parse();

    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths
    };

    let mut grand_total: u64 = 0;

    for path in &paths {
        let size = du(path, !args.summarize);
        grand_total += size;
        println!("{} {}", format_size(size, args.human_readable), path.display());
    }

    if args.total && paths.len() > 1 {
        println!("{} total", format_size(grand_total, args.human_readable));
    }
}

fn du(path: &Path, recursive: bool) -> u64 {
    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("du: {}: {}", path.display(), e);
            return 0;
        }
    };

    let mut size = metadata.len();

    if metadata.is_dir() && recursive {
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    size += du(&entry.path(), true);
                }
            }
            Err(e) => eprintln!("du: {}: {}", path.display(), e),
        }
    }

    size
}

fn format_size(bytes: u64, human_readable: bool) -> String {
    if !human_readable {
        return bytes.to_string();
    }

    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < units.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{}", bytes)
    } else {
        format!("{:.1}{}", size, units[unit_idx])
    }
}