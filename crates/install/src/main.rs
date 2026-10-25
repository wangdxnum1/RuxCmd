mod cli;

use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let args = cli::Args::parse();

    let mode = match &args.mode {
        Some(text) => match parse_mode(text) {
            Ok(m) => m,
            Err(()) => {
                eprintln!("install: invalid mode '{}'", text);
                std::process::exit(1);
            }
        },
        // GNU install's default mode is 0755; on Windows only the write bits
        // matter, so 755 means "writable".
        None => 0o755,
    };

    if args.operands.is_empty() {
        eprintln!("install: missing file operand");
        std::process::exit(1);
    }

    let ok = if args.directory {
        create_directories(&args.operands, mode, args.verbose)
    } else {
        copy_sources(&args.operands, mode, args.verbose)
    };

    if !ok {
        std::process::exit(1);
    }
}

/// Parse an octal mode such as 755 or 644. The symbolic-mode forms of GNU
/// install are not supported.
fn parse_mode(text: &str) -> Result<u32, ()> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(());
    }
    let mode = u32::from_str_radix(text, 8).map_err(|_| ())?;
    if mode > 0o7777 {
        return Err(());
    }
    Ok(mode)
}

/// -d: create every operand as a directory, parents included.
fn create_directories(operands: &[PathBuf], mode: u32, verbose: bool) -> bool {
    let mut ok = true;

    for dir in operands {
        let existed = dir.exists();
        if let Err(e) = fs::create_dir_all(dir) {
            eprintln!(
                "install: cannot create directory '{}': {}",
                dir.display(),
                e
            );
            ok = false;
            continue;
        }
        apply_mode(dir, mode);
        if verbose && !existed {
            println!("install -> '{}'", dir.display());
        }
    }

    ok
}

/// Copy mode: one source to DEST (or DEST/ if it is an existing directory),
/// or several sources into a directory.
fn copy_sources(operands: &[PathBuf], mode: u32, verbose: bool) -> bool {
    let (sources, dest) = operands.split_at(operands.len() - 1);
    let dest = &dest[0];

    // With multiple sources the destination must be an existing directory,
    // otherwise every source would overwrite the same target file.
    if sources.len() > 1 && !dest.is_dir() {
        eprintln!("install: target '{}' is not a directory", dest.display());
        return false;
    }

    let mut ok = true;

    for source in sources {
        if let Err(e) = install_file(source, dest, mode, verbose) {
            eprintln!("install: {}", e);
            ok = false;
        }
    }

    ok
}

/// Resolve the destination path for one source and copy, then apply the mode.
fn install_file(source: &Path, dest: &Path, mode: u32, verbose: bool) -> Result<(), String> {
    if !source.is_file() {
        return Err(format!(
            "cannot stat '{}': No such file or directory",
            source.display()
        ));
    }

    let dest_path = if dest.is_dir() {
        match source.file_name() {
            Some(name) => dest.join(name),
            None => {
                return Err(format!(
                    "cannot determine target name for '{}'",
                    source.display()
                ));
            }
        }
    } else {
        dest.to_path_buf()
    };

    // CopyFileEx refuses to overwrite a read-only destination; install (like
    // GNU) overwrites silently, so clear the bit first.
    if dest_path.exists() {
        let _ = ruxcmd_core::fsutil::clear_readonly(&dest_path);
    }

    fs::copy(source, &dest_path).map_err(|e| {
        format!(
            "cannot copy '{}' to '{}': {}",
            source.display(),
            dest_path.display(),
            e
        )
    })?;

    apply_mode(&dest_path, mode);

    if verbose {
        println!("install -> '{}'", dest_path.display());
    }

    Ok(())
}

/// Windows keeps one read-only bit where Unix keeps nine mode bits: any write
/// permission in OCTAL (the 0o222 bits) clears it, no write bit sets it. The
/// default 755 therefore installs writable files.
fn apply_mode(path: &Path, mode: u32) {
    #[cfg(windows)]
    {
        if let Ok(meta) = fs::metadata(path) {
            let mut perms = meta.permissions();
            let readonly = (mode & 0o222) == 0;
            if perms.readonly() != readonly {
                perms.set_readonly(readonly);
                let _ = fs::set_permissions(path, perms);
            }
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
    }
}

#[cfg(test)]
mod tests {
    use super::parse_mode;

    #[test]
    fn parses_octal_modes_only() {
        assert_eq!(parse_mode("755"), Ok(0o755));
        assert_eq!(parse_mode("0"), Ok(0o000));
        assert_eq!(parse_mode("644"), Ok(0o644));
        assert!(parse_mode("").is_err());
        assert!(parse_mode("abc").is_err());
        assert!(parse_mode("77x").is_err());
        assert!(parse_mode("77777").is_err());
    }
}
