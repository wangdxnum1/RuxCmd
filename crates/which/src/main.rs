mod cli;

use clap::Parser;
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    if args.commands.is_empty() {
        eprintln!("which: no arguments");
        std::process::exit(1);
    }

    let path_env = std::env::var("PATH").unwrap_or_default();
    let path_dirs: Vec<&str> = path_env.split(';').collect();
    let extensions = build_extensions();

    let mut exit_code = 0i32;
    for cmd in &args.commands {
        let found = find_command(cmd, &path_dirs, &extensions, args.all);
        if !found {
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

/// Windows extension probing: the exact name first, then the standard
/// executable extensions, then any extra PATHEXT entries in their declared
/// order (duplicates removed, case-insensitive).
fn build_extensions() -> Vec<String> {
    let mut exts: Vec<String> = vec![String::new()];
    let push = |raw: &str, exts: &mut Vec<String>| {
        let e = raw.trim().to_ascii_lowercase();
        if !e.is_empty() && !exts.iter().any(|x| x == &e) {
            exts.push(e);
        }
    };
    for d in [".com", ".exe", ".bat", ".cmd"] {
        push(d, &mut exts);
    }
    if let Ok(pat) = std::env::var("PATHEXT") {
        for e in pat.split(';') {
            push(e, &mut exts);
        }
    }
    exts
}

fn find_command(cmd: &str, path_dirs: &[&str], extensions: &[String], all: bool) -> bool {
    let mut found = false;

    for dir in path_dirs {
        let dir_path = Path::new(dir);
        if !dir_path.is_dir() {
            continue;
        }

        for ext in extensions {
            let file_name = format!("{}{}", cmd, ext);
            let full_path = dir_path.join(&file_name);

            if full_path.is_file() {
                // Print the path as written (no symlink resolution), like
                // GNU which and Windows where.exe.
                println!("{}", ruxcmd_core::paths::display(&full_path));
                found = true;
                if !all {
                    return true;
                }
            }
        }
    }

    found
}
