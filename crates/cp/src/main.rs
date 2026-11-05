mod cli;

use clap::Parser;
use std::fs;
use std::io::{self, Write};
#[cfg(windows)]
use std::os::windows::fs::symlink_file;
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    if args.sources.is_empty() {
        eprintln!("cp: missing file operand");
        std::process::exit(1);
    }

    // With multiple sources the destination must be an existing directory,
    // otherwise each source would silently overwrite the same target file.
    if args.sources.len() > 1 && !(args.destination.exists() && args.destination.is_dir()) {
        eprintln!(
            "cp: target '{}' is not a directory",
            args.destination.display()
        );
        std::process::exit(1);
    }

    let mut exit_code = 0i32;
    let mut had_error = false;

    for source in &args.sources {
        if let Err(e) = copy_file(source, &args.destination, &args, &mut had_error) {
            eprintln!("cp: {}", e);
            exit_code = 1;
        }
    }

    if had_error {
        exit_code = 1;
    }

    std::process::exit(exit_code);
}

/// How an existing destination is treated. GNU cp overwrites by default; -n
/// and -i share one interactive setting and the last one on the command line
/// wins, while -f is an independent switch that does not settle that race.
#[derive(Clone, Copy)]
enum Overwrite {
    Allow,
    Prompt,
    Skip,
}

fn overwrite_decision(args: &cli::Args) -> Overwrite {
    if args.no_clobber {
        Overwrite::Skip
    } else if args.interactive {
        Overwrite::Prompt
    } else {
        Overwrite::Allow
    }
}

/// GNU cp can only replace an unwritable destination when -f is in effect;
/// without it the copy fails with "Permission denied". On Windows the
/// equivalent of "unwritable" is the read-only attribute.
fn ensure_replaceable(path: &Path, force: bool) -> Result<(), String> {
    let metadata =
        fs::metadata(path).map_err(|e| format!("cannot stat '{}': {}", path.display(), e))?;
    if !metadata.permissions().readonly() {
        return Ok(());
    }
    if !force {
        return Err(format!(
            "cannot create regular file '{}': Permission denied",
            path.display()
        ));
    }
    let _ = ruxcmd_core::fsutil::clear_readonly(path);
    Ok(())
}

fn copy_file(
    source: &Path,
    dest: &Path,
    args: &cli::Args,
    had_error: &mut bool,
) -> Result<(), String> {
    if !source.exists() {
        return Err(format!(
            "cannot stat '{}': No such file or directory",
            source.display()
        ));
    }

    let source_metadata =
        fs::metadata(source).map_err(|e| format!("cannot stat '{}': {}", source.display(), e))?;

    if source_metadata.is_dir() {
        if !args.recursive {
            return Err(format!("omitting directory '{}'", source.display()));
        }
        copy_directory(source, dest, args, had_error)
    } else {
        let dest_path = if dest.exists() && dest.is_dir() {
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

        if dest_path.exists() {
            match overwrite_decision(args) {
                Overwrite::Skip => return Ok(()),
                Overwrite::Prompt => {
                    if !prompt_yes_no(&format!("cp: overwrite '{}'? ", dest_path.display())) {
                        return Ok(());
                    }
                }
                Overwrite::Allow => {}
            }
        }

        if dest_path.exists() {
            ensure_replaceable(&dest_path, args.force)?;
        }

        if args.symbolic_link {
            #[cfg(windows)]
            symlink_file(source, &dest_path).map_err(|e| {
                format!(
                    "cannot create symbolic link '{}': {}",
                    dest_path.display(),
                    e
                )
            })?;
            #[cfg(not(windows))]
            fs::symlink_file(source, &dest_path).map_err(|e| {
                format!(
                    "cannot create symbolic link '{}': {}",
                    dest_path.display(),
                    e
                )
            })?;
        } else {
            fs::copy(source, &dest_path).map_err(|e| {
                format!(
                    "cannot copy '{}' to '{}': {}",
                    source.display(),
                    dest_path.display(),
                    e
                )
            })?;
        }

        if args.verbose {
            println!("'{}' -> '{}'", source.display(), dest_path.display());
        }

        Ok(())
    }
}

/// Copy one file entry from inside a directory tree, applying the same
/// overwrite and read-only rules as a top-level copy.
fn copy_entry_file(source: &Path, dest_path: &Path, args: &cli::Args) -> Result<(), String> {
    if dest_path.exists() {
        match overwrite_decision(args) {
            Overwrite::Skip => return Ok(()),
            Overwrite::Prompt => {
                if !prompt_yes_no(&format!("cp: overwrite '{}'? ", dest_path.display())) {
                    return Ok(());
                }
            }
            Overwrite::Allow => {}
        }
        ensure_replaceable(dest_path, args.force)?;
    }

    fs::copy(source, dest_path).map_err(|e| {
        format!(
            "cannot copy '{}' to '{}': {}",
            source.display(),
            dest_path.display(),
            e
        )
    })?;

    if args.verbose {
        println!("'{}' -> '{}'", source.display(), dest_path.display());
    }

    Ok(())
}

fn copy_directory(
    source: &Path,
    dest: &Path,
    args: &cli::Args,
    had_error: &mut bool,
) -> Result<(), String> {
    let dest_path = if ends_in_dot_component(source) {
        // `cp -r src/. dest` (also bare `.`/`..`): copy the contents of
        // source into dest itself. Dest must already be an existing
        // directory; unlike GNU cp we do not create it on demand.
        if !dest.is_dir() {
            return Err(format!("target '{}' is not a directory", dest.display()));
        }
        dest.to_path_buf()
    } else if dest.exists() && dest.is_dir() {
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

    if !dest_path.exists() {
        fs::create_dir_all(&dest_path)
            .map_err(|e| format!("cannot create directory '{}': {}", dest_path.display(), e))?;
    }

    let entries = fs::read_dir(source)
        .map_err(|e| format!("cannot read directory '{}': {}", source.display(), e))?;

    for entry in entries {
        let entry =
            entry.map_err(|e| format!("cannot read entry in '{}': {}", source.display(), e))?;
        let entry_path = entry.path();
        let dest_entry_path = dest_path.join(entry.file_name());

        let entry_metadata = fs::metadata(&entry_path)
            .map_err(|e| format!("cannot stat '{}': {}", entry_path.display(), e))?;

        if entry_metadata.is_dir() {
            // A failing entry must not abort the remaining tree, but it does
            // make the overall cp fail (GNU exits 1 after reporting it).
            if let Err(e) = copy_directory(&entry_path, &dest_entry_path, args, had_error) {
                eprintln!("cp: {}", e);
                *had_error = true;
            }
        } else if let Err(e) = copy_entry_file(&entry_path, &dest_entry_path, args) {
            eprintln!("cp: {}", e);
            *had_error = true;
        }
    }

    Ok(())
}

/// True when the path's final component is `.` or `..` (e.g. `.`, `./`,
/// `src/.`, `src/..`), so the path carries no usable trailing name.
fn ends_in_dot_component(path: &Path) -> bool {
    use std::path::Component;
    match path.components().next_back() {
        Some(Component::CurDir) | Some(Component::ParentDir) => return true,
        _ => {}
    }
    // `components()` normalizes away interior/trailing `.` segments, so also
    // inspect the raw text for a trailing dot component such as `src/.`.
    let text = path.as_os_str().to_string_lossy();
    let text = text.trim_end_matches(['/', '\\']);
    matches!(text.rsplit(['/', '\\']).next(), Some(".") | Some(".."))
}

fn prompt_yes_no(prompt: &str) -> bool {
    // GNU cp writes its prompts to stderr, keeping stdout clean for scripts.
    eprint!("{}", prompt);
    let _ = io::stderr().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    let input = input.trim().to_lowercase();
    input == "y" || input == "yes"
}

#[cfg(test)]
mod tests {
    use super::ends_in_dot_component;
    use std::path::Path;

    #[test]
    fn detects_trailing_dot_components() {
        for p in [
            ".", "./", "./.", "..", "../", "src/.", "src/..", "src/../", "a\\b\\..",
        ] {
            assert!(
                ends_in_dot_component(Path::new(p)),
                "expected dot tail: {p}"
            );
        }
        for p in [
            "src",
            "src/",
            "src/./sub",
            "src/..\\sub",
            "/",
            "\\",
            "C:\\",
            "C:\\dir",
            "",
        ] {
            assert!(
                !ends_in_dot_component(Path::new(p)),
                "expected no dot tail: {p}"
            );
        }
    }
}
