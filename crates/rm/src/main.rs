mod cli;

use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    // If -f with no files, that's okay (ignore missing operands)
    if args.files.is_empty() && args.force {
        return;
    }

    // If -i is set, force is irrelevant
    let interactive_always = args.interactive; // -i
    let interactive_once = args.interactive_once; // -I
    let force = args.force && !interactive_always && !interactive_once;

    let mut exit_code = 0i32;

    // Determine interactive-once threshold
    let has_many_files = args.files.len() > 3 || args.recursive;

    if interactive_once && has_many_files {
        if !prompt_yes_no(
            &format!(
                "rm: remove {} argument(s){}? ",
                args.files.len(),
                if args.recursive { " recursively" } else { "" }
            ),
            false,
        ) {
            return;
        }
    }

    for path in &args.files {
        match remove_path(
            path,
            force,
            interactive_always,
            false, // interactive_once already handled above
            args.recursive,
            args.remove_empty_dirs,
            args.verbose,
            args.one_file_system,
            args.no_preserve_root,
        ) {
            Ok(()) => {}
            Err(e) => {
                if !force {
                    eprintln!("rm: {}", e);
                }
                exit_code = 1;
            }
        }
    }

    std::process::exit(exit_code);
}

/// Check if path should be protected (root filesystem protection).
fn is_protected_root(path: &Path, no_preserve_root: bool) -> bool {
    if no_preserve_root {
        return false;
    }

    // On Windows, protect drive roots like C:\, D:\
    if let Some(path_str) = path.to_str() {
        // Match patterns like "C:\" or "C:" or "/" or "\"
        let path_str = path_str.trim_end_matches(['/', '\\']);
        if path_str.len() == 2 && path_str.as_bytes()[1] == b':' {
            return true;
        }
        if path_str.is_empty() || path_str == "/" || path_str == "\\" {
            return true;
        }
    }

    // Also check canonical form
    if let Ok(canonical) = path.canonicalize() {
        if let Some(canon_str) = canonical.to_str() {
            let canon_str = canon_str.trim_end_matches(['/', '\\']);
            if canon_str.len() == 2 && canon_str.as_bytes()[1] == b':' {
                return true;
            }
        }
    }

    false
}

/// Get the device ID for a path (for one-file-system check).
fn get_device_id(path: &Path) -> Option<u64> {
    #[cfg(windows)]
    {
        // On Windows, use the drive letter as a device identifier
        let path_str = path.to_string_lossy();
        if path_str.len() >= 2 && path_str.as_bytes().get(1) == Some(&b':') {
            Some(path_str.as_bytes()[0].to_ascii_uppercase() as u64)
        } else {
            Some(0)
        }
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::MetadataExt;
        fs::metadata(path).ok().map(|m| m.dev())
    }
}

/// Prompt user with yes/no question.
fn prompt_yes_no(prompt: &str, default_no: bool) -> bool {
    let suffix = if default_no { " [y/N] " } else { " [y/n] " };
    print!("{}{}", prompt, suffix);
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    let input = input.trim().to_lowercase();
    if default_no {
        input == "y" || input == "yes"
    } else {
        input != "n" && input != "no"
    }
}

/// Core removal logic.
fn remove_path(
    path: &Path,
    force: bool,
    interactive: bool,
    _interactive_once_handled: bool,
    recursive: bool,
    remove_empty_dirs: bool,
    verbose: bool,
    one_file_system: bool,
    no_preserve_root: bool,
) -> Result<(), String> {
    // Check if path exists (symlink handling: use metadata for existence check)
    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) => {
            if force {
                return Ok(());
            }
            return Err(format!("cannot remove '{}': {}", path.display(), e));
        }
    };

    // Protect root
    if is_protected_root(path, no_preserve_root) {
        return Err(format!(
            "refusing to remove '{}', skipping (use --no-preserve-root to override)",
            path.display()
        ));
    }

    let file_type = metadata.file_type();

    if file_type.is_dir() && !file_type.is_symlink() {
        // It's a directory
        if !recursive && !remove_empty_dirs {
            return Err(format!(
                "cannot remove '{}': Is a directory",
                path.display()
            ));
        }

        if remove_empty_dirs && !recursive {
            // -d: only remove empty directories
            match fs::remove_dir(path) {
                Ok(()) => {
                    if verbose {
                        eprintln!("removed directory '{}'", path.display());
                    }
                    return Ok(());
                }
                Err(e) => {
                    return Err(format!("cannot remove '{}': {}", path.display(), e));
                }
            }
        }

        if recursive {
            // Check one-file-system
            let root_device = if one_file_system {
                get_device_id(path)
            } else {
                None
            };

            // Recursively remove directory contents
            match remove_dir_recursive(
                path,
                force,
                interactive,
                verbose,
                one_file_system,
                root_device,
            ) {
                Ok(()) => {
                    if verbose {
                        eprintln!("removed directory '{}'", path.display());
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        } else {
            Err(format!(
                "cannot remove '{}': Is a directory",
                path.display()
            ))
        }
    } else {
        // It's a file or symlink
        if interactive {
            if !prompt_yes_no(
                &format!("rm: remove regular file '{}'? ", path.display()),
                false,
            ) {
                return Ok(());
            }
        }

        match fs::remove_file(path) {
            Ok(()) => {
                if verbose {
                    eprintln!("removed '{}'", path.display());
                }
                Ok(())
            }
            Err(e) => {
                if force {
                    if verbose {
                        eprintln!(
                            "rm: cannot remove '{}': {} (ignored due to --force)",
                            path.display(),
                            e
                        );
                    }
                    return Ok(());
                }
                Err(format!("cannot remove '{}': {}", path.display(), e))
            }
        }
    }
}

/// Recursively remove directory contents, then the directory itself.
fn remove_dir_recursive(
    dir: &Path,
    force: bool,
    interactive: bool,
    verbose: bool,
    one_file_system: bool,
    root_device: Option<u64>,
) -> Result<(), String> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            return Err(format!("cannot read directory '{}': {}", dir.display(), e));
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                if !force {
                    return Err(format!("cannot read entry in '{}': {}", dir.display(), e));
                }
                continue;
            }
        };

        let path = entry.path();
        let metadata = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                if !force {
                    return Err(format!("cannot stat '{}': {}", path.display(), e));
                }
                continue;
            }
        };

        // Check one-file-system
        if one_file_system {
            if let Some(ref root_dev) = root_device {
                let entry_device = get_device_id(&path);
                if entry_device.is_some() && entry_device != Some(*root_dev) {
                    if verbose {
                        eprintln!("rm: skipping '{}' (different file system)", path.display());
                    }
                    continue;
                }
            }
        }

        if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
            remove_dir_recursive(
                &path,
                force,
                interactive,
                verbose,
                one_file_system,
                root_device,
            )?;
        } else {
            // File or symlink
            if interactive {
                if !prompt_yes_no(
                    &format!("rm: remove regular file '{}'? ", path.display()),
                    false,
                ) {
                    continue;
                }
            }

            if let Err(e) = fs::remove_file(&path) {
                if !force {
                    return Err(format!("cannot remove '{}': {}", path.display(), e));
                }
            } else if verbose {
                eprintln!("removed '{}'", path.display());
            }
        }
    }

    // Remove the top-level directory itself
    fs::remove_dir(dir).map_err(|e| format!("cannot remove '{}': {}", dir.display(), e))
}
