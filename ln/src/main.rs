mod cli;

use clap::Parser;
use std::fs;

fn main() {
    let args = cli::Args::parse();

    if !args.target.exists() {
        eprintln!("ln: failed to access '{}': No such file or directory", args.target.display());
        std::process::exit(1);
    }

    let target = fs::canonicalize(&args.target).unwrap_or_else(|_| args.target.clone());

    let link_path = if args.link_name.is_dir() {
        args.link_name.join(target.file_name().unwrap())
    } else {
        args.link_name
    };

    if link_path.exists() {
        if args.force {
            if let Err(e) = fs::remove_file(&link_path) {
                eprintln!("ln: cannot remove '{}': {}", link_path.display(), e);
                std::process::exit(1);
            }
        } else {
            eprintln!("ln: failed to create link '{}': File exists", link_path.display());
            std::process::exit(1);
        }
    }

    let result = if args.symbolic {
        create_symlink(&target, &link_path)
    } else {
        fs::hard_link(&target, &link_path)
    };

    match result {
        Ok(_) => (),
        Err(e) => {
            eprintln!("ln: failed to create link '{}': {}", link_path.display(), e);
            std::process::exit(1);
        }
    }
}

#[cfg(windows)]
fn create_symlink(target: &std::path::Path, link_path: &std::path::Path) -> std::io::Result<()> {
    use std::os::windows::fs::symlink_file;
    symlink_file(target, link_path)
}

#[cfg(unix)]
fn create_symlink(target: &std::path::Path, link_path: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::symlink;
    symlink(target, link_path)
}