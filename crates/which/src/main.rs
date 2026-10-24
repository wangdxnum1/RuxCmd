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

    let mut exit_code = 0i32;

    for cmd in &args.commands {
        let found = find_command(cmd, &path_dirs, args.all);
        if !found {
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

fn find_command(cmd: &str, path_dirs: &[&str], all: bool) -> bool {
    let mut found = false;

    for dir in path_dirs {
        let dir_path = Path::new(dir);
        if !dir_path.is_dir() {
            continue;
        }

        let extensions = vec!["", ".exe", ".cmd", ".bat", ".com"];

        for ext in &extensions {
            let file_name = format!("{}{}", cmd, ext);
            let full_path = dir_path.join(&file_name);

            if full_path.is_file()
                && let Ok(path) = full_path.canonicalize()
            {
                println!("{}", strip_verbatim(&path));
                found = true;
                if !all {
                    return true;
                }
            }
        }
    }

    found
}

/// Strip the `\\?\` verbatim prefix (mapping `\\?\UNC\` to `//`) and normalize
/// backslashes to forward slashes so the output is directly reusable.
fn strip_verbatim(path: &Path) -> String {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        return format!("//{}", rest.replace('\\', "/"));
    }
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s.replace('\\', "/")
}
