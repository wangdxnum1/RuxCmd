mod cli;

use clap::Parser;
use std::fs;
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    if args.files.is_empty() {
        eprintln!("chmod: missing file operand");
        std::process::exit(1);
    }

    let mode = parse_mode(&args.mode).unwrap_or_else(|e| {
        eprintln!("chmod: {}", e);
        std::process::exit(1);
    });

    let mut exit_code = 0i32;

    for file in &args.files {
        if let Err(e) = chmod_file(file, mode, &args) {
            eprintln!("chmod: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

fn parse_mode(mode_str: &str) -> Result<u32, String> {
    let mode =
        u32::from_str_radix(mode_str, 8).map_err(|_| format!("invalid mode: '{}'", mode_str))?;

    if mode > 0o7777 {
        return Err(format!("mode '{}' out of range", mode_str));
    }

    Ok(mode)
}

fn chmod_file(path: &Path, mode: u32, args: &cli::Args) -> Result<(), String> {
    if !path.exists() {
        return Err(format!(
            "cannot access '{}': No such file or directory",
            path.display()
        ));
    }

    if path.is_dir() && args.recursive {
        chmod_recursive(path, mode, args)?;
    }

    set_permissions(path, mode)
        .map_err(|e| format!("cannot change permissions of '{}': {}", path.display(), e))?;

    if args.verbose {
        println!("mode of '{}' changed to {:04o}", path.display(), mode);
    }

    Ok(())
}

fn chmod_recursive(dir: &Path, mode: u32, args: &cli::Args) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("cannot read directory '{}': {}", dir.display(), e))?;

    for entry in entries {
        let entry =
            entry.map_err(|e| format!("cannot read entry in '{}': {}", dir.display(), e))?;

        let entry_path = entry.path();

        if entry_path.is_dir() {
            chmod_recursive(&entry_path, mode, args)?;
        }

        set_permissions(&entry_path, mode).map_err(|e| {
            format!(
                "cannot change permissions of '{}': {}",
                entry_path.display(),
                e
            )
        })?;

        if args.verbose {
            println!("mode of '{}' changed to {:04o}", entry_path.display(), mode);
        }
    }

    Ok(())
}

#[cfg(windows)]
fn set_permissions(path: &Path, mode: u32) -> std::io::Result<()> {
    let mut perms = fs::metadata(path)?.permissions();

    let readonly = (mode & 0o222) == 0;
    perms.set_readonly(readonly);

    fs::set_permissions(path, perms)
}

#[cfg(unix)]
fn set_permissions(path: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let perms = fs::Permissions::from_mode(mode);
    fs::set_permissions(path, perms)
}
