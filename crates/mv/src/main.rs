mod cli;

use clap::Parser;
use ruxcmd_core::fsutil::clear_readonly;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    if args.sources.is_empty() {
        eprintln!("mv: missing file operand");
        std::process::exit(1);
    }

    let mut exit_code = 0i32;

    for source in &args.sources {
        if let Err(e) = move_file(source, &args.destination, &args) {
            eprintln!("mv: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

fn move_file(source: &Path, dest: &Path, args: &cli::Args) -> Result<(), String> {
    if !source.exists() {
        return Err(format!(
            "cannot stat '{}': No such file or directory",
            source.display()
        ));
    }

    // `mv src/. dst` (also bare `.`/`..`): merge the contents of `src` into
    // the existing directory `dst`, mirroring cp's dot-component handling.
    // (There is no usable file_name() for a trailing dot component.)
    if ends_in_dot_component(source) {
        if !dest.is_dir() {
            return Err(format!("target '{}' is not a directory", dest.display()));
        }
        let entries = fs::read_dir(source)
            .map_err(|e| format!("cannot read directory '{}': {}", source.display(), e))?;
        let mut first_error = None;
        for entry in entries {
            let entry =
                entry.map_err(|e| format!("cannot read entry in '{}': {}", source.display(), e))?;
            if let Err(e) = move_file(&entry.path(), dest, args)
                && first_error.is_none()
            {
                first_error = Some(e);
            }
        }
        return match first_error {
            Some(e) => Err(e),
            None => Ok(()),
        };
    }

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

    // Use symlink_metadata so a symlink to a directory is renamed as a link
    // (GNU mv does not dereference source symlinks).
    let source_is_dir = fs::symlink_metadata(source)
        .map(|m| m.is_dir())
        .unwrap_or(false);

    if same_file(source, &dest_path) {
        // GNU reports identical operands instead of replacing the entry; -n
        // turns the same collision into a silent skip.
        if args.no_clobber {
            return Ok(());
        }
        return Err(format!(
            "'{}' and '{}' are the same file",
            source.display(),
            dest_path.display()
        ));
    }

    if source_is_dir && dest.exists() && dest.is_dir() && same_file(source, dest) {
        // `mv dir dir` would land inside itself; GNU rejects it before the
        // rename, and -n does not suppress this diagnostic.
        return Err(format!(
            "cannot move '{}' to a subdirectory of itself, '{}'",
            source.display(),
            dest_path.display()
        ));
    }

    if dest_path.exists() {
        let dest_is_dir = dest_path.is_dir();

        if dest_is_dir && !source_is_dir {
            // Never delete the destination tree to make room for a file.
            return Err(format!(
                "cannot overwrite directory '{}' with non-directory",
                dest_path.display()
            ));
        }
        if !dest_is_dir && source_is_dir {
            return Err(format!(
                "cannot overwrite non-directory '{}' with directory",
                dest_path.display()
            ));
        }

        // GNU overwrites an existing destination by default (no prompt); -n
        // keeps both operands untouched and still exits successfully.
        if args.no_clobber {
            return Ok(());
        }

        if !args.force && args.interactive {
            let prompt = format!("mv: overwrite '{}'? ", dest_path.display());
            if !prompt_yes_no(&prompt) {
                return Ok(());
            }
        }

        if !dest_is_dir {
            // Windows refuses to delete a read-only file, but GNU mv replaces
            // the destination outright (unlink only needs write access to the
            // containing directory, not to the file itself). GetFileAttributesW
            // reports the link itself, so a symlink target keeps its attributes.
            let _ = clear_readonly(&dest_path);
            fs::remove_file(&dest_path)
                .map_err(|e| format!("cannot remove '{}': {}", dest_path.display(), e))?;
        }
        // Both operands are directories: like GNU mv we keep the existing
        // directory and let the rename fail when it cannot be replaced
        // (non-empty) — never remove_dir_all the destination.
    }

    match fs::rename(source, &dest_path) {
        Ok(()) => {}
        Err(_) => {
            // Typically a cross-volume move: fall back to copy + delete.
            move_by_copy(source, &dest_path).map_err(|e| {
                format!(
                    "cannot move '{}' to '{}': {}",
                    source.display(),
                    dest_path.display(),
                    e
                )
            })?;
        }
    }

    if args.verbose {
        println!("'{}' -> '{}'", source.display(), dest_path.display());
    }

    Ok(())
}

/// Fallback for `fs::rename` failures (typically cross-volume moves): copy
/// the file or the whole tree, then delete the source.
fn move_by_copy(source: &Path, dest: &Path) -> io::Result<()> {
    let metadata = fs::metadata(source)?;
    if metadata.is_dir() {
        // Like GNU mv, an existing non-empty destination directory is an
        // error rather than something to merge into or wipe.
        if dest.exists() && fs::read_dir(dest)?.next().is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "destination directory is not empty",
            ));
        }
        copy_dir_recursive(source, dest)?;
        // Windows refuses to delete read-only files and read-only directories,
        // so clear the attributes across the tree before std removes it.
        clear_readonly_recursive(source)?;
        fs::remove_dir_all(source)?;
    } else {
        fs::copy(source, dest)?;
        // The source may be read-only: clear the attribute before unlinking
        // (a symlink's own attributes are reported, its target is untouched).
        let _ = clear_readonly(source);
        fs::remove_file(source)?;
    }
    Ok(())
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_dest = dest.join(entry.file_name());
        if fs::metadata(entry.path())?.is_dir() {
            copy_dir_recursive(&entry.path(), &entry_dest)?;
        } else {
            fs::copy(entry.path(), &entry_dest)?;
        }
    }
    Ok(())
}

/// Clears the read-only attribute on `dir` and every entry below it so the
/// tree can be deleted. Symlinks are skipped: they never block removal, and
/// their targets must keep their attributes.
fn clear_readonly_recursive(dir: &Path) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        let _ = clear_readonly(&path);
        if file_type.is_dir() {
            clear_readonly_recursive(&path)?;
        }
    }
    let _ = clear_readonly(dir);
    Ok(())
}

/// True when both paths resolve to the same file, matching GNU's
/// "are the same file" guard. Windows paths compare case-insensitively.
/// Hard links are not detected: std exposes no stable file id, and canonical
/// paths cannot tell two names for one file apart.
fn same_file(a: &Path, b: &Path) -> bool {
    let (Ok(a), Ok(b)) = (fs::canonicalize(a), fs::canonicalize(b)) else {
        return false;
    };
    #[cfg(windows)]
    {
        a.as_os_str()
            .to_string_lossy()
            .eq_ignore_ascii_case(&b.as_os_str().to_string_lossy())
    }
    #[cfg(not(windows))]
    {
        a == b
    }
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
    // GNU writes the prompt to stderr, keeping stdout clean for scripts.
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
    use super::*;

    fn set_readonly(path: &Path) {
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(path, perms).unwrap();
    }

    /// Cross-volume rename fails; the copy+delete fallback must still move
    /// the file. (The test environment has a single volume, so the fallback
    /// is exercised directly.)
    #[test]
    fn cross_volume_fallback_moves_file_by_copy_and_delete() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("f.txt");
        fs::write(&src, "bytes").unwrap();
        let dst = tmp.path().join("f2.txt");

        move_by_copy(&src, &dst).unwrap();
        assert!(!src.exists(), "source must be deleted after the copy");
        assert_eq!(fs::read_to_string(&dst).unwrap(), "bytes");
    }

    #[test]
    fn cross_volume_fallback_moves_directory_tree_by_copy_and_delete() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("tree");
        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("a.txt"), "A").unwrap();
        fs::write(src.join("sub").join("b.txt"), "B").unwrap();
        let dst = tmp.path().join("tree2");

        move_by_copy(&src, &dst).unwrap();
        assert!(!src.exists(), "source tree must be deleted after the copy");
        assert_eq!(fs::read_to_string(dst.join("a.txt")).unwrap(), "A");
        assert_eq!(
            fs::read_to_string(dst.join("sub").join("b.txt")).unwrap(),
            "B"
        );
    }

    #[test]
    fn cross_volume_fallback_deletes_readonly_source_file() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("ro.txt");
        fs::write(&src, "bytes").unwrap();
        set_readonly(&src);
        let dst = tmp.path().join("moved.txt");

        move_by_copy(&src, &dst).unwrap();
        assert!(!src.exists(), "read-only source must be deleted");
        assert_eq!(fs::read_to_string(&dst).unwrap(), "bytes");
    }

    #[test]
    fn cross_volume_fallback_deletes_tree_with_readonly_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("tree");
        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("a.txt"), "A").unwrap();
        fs::write(src.join("sub").join("b.txt"), "B").unwrap();
        set_readonly(&src.join("a.txt"));
        set_readonly(&src.join("sub"));
        set_readonly(&src);
        let dst = tmp.path().join("tree2");

        move_by_copy(&src, &dst).unwrap();
        assert!(!src.exists(), "read-only tree must be deleted");
        assert_eq!(fs::read_to_string(dst.join("a.txt")).unwrap(), "A");
        assert_eq!(
            fs::read_to_string(dst.join("sub").join("b.txt")).unwrap(),
            "B"
        );
    }

    #[test]
    fn cross_volume_fallback_refuses_nonempty_destination_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("tree");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("a.txt"), "A").unwrap();
        let dst = tmp.path().join("dst");
        fs::create_dir(&dst).unwrap();
        fs::write(dst.join("old.txt"), "OLD").unwrap();

        let err = move_by_copy(&src, &dst).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(dst.join("old.txt")).unwrap(), "OLD");
        assert!(
            src.join("a.txt").exists(),
            "source must survive the failure"
        );
    }

    #[test]
    fn dot_source_merges_contents_into_existing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(src.join("nested")).unwrap();
        fs::write(src.join("a.txt"), "alpha").unwrap();
        fs::write(src.join("nested").join("b.txt"), "beta").unwrap();

        let dst = tmp.path().join("dst");
        fs::create_dir(&dst).unwrap();
        fs::write(dst.join("keep.txt"), "keep").unwrap();

        let args = cli::Args::parse_from(["mv", "x", "y"]);
        // The dot-component form must be absolute here so the merge reads
        // the fixture directory, not the test process's working directory.
        move_file(&src.join("."), &dst, &args).unwrap();

        assert_eq!(fs::read_to_string(dst.join("a.txt")).unwrap(), "alpha");
        assert_eq!(
            fs::read_to_string(dst.join("nested").join("b.txt")).unwrap(),
            "beta"
        );
        assert_eq!(fs::read_to_string(dst.join("keep.txt")).unwrap(), "keep");
        // Contents merged: no dst/src or dst/dst subdirectory may appear.
        assert!(!dst.join("src").exists());
        assert!(!dst.join("dst").exists());
    }
}
