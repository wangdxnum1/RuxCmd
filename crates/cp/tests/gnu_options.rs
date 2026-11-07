//! GNU-alignment tests for cp 0.4.6 (GNU coreutils 8.32 reference):
//! - `-a/--archive` implies `-p` (mappable subset) and `-R`
//! - `-u/--update` copies only when the source is strictly newer
//! - `-t/--target-directory` turns every operand into a source
//! - `-T/--no-target-directory` always treats the destination as a file

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::PredicateBooleanExt;
use std::fs;
use std::time::{Duration, SystemTime};

fn stamp(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

/// std cannot open a directory for writing, so the test uses the same
/// FILE_WRITE_ATTRIBUTES + FILE_FLAG_BACKUP_SEMANTICS route as cp itself.
#[cfg(windows)]
fn set_dir_modified(path: &std::path::Path, when: SystemTime) {
    use std::os::windows::ffi::OsStrExt;
    use std::time::UNIX_EPOCH;
    use windows::Win32::Foundation::{CloseHandle, FILETIME};
    use windows::Win32::Storage::FileSystem::{
        CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_READ, FILE_SHARE_WRITE,
        FILE_WRITE_ATTRIBUTES, OPEN_EXISTING, SetFileTime,
    };

    let ticks = when.duration_since(UNIX_EPOCH).unwrap().as_nanos() / 100 + 116_444_736_000_000_000;
    let stamp = FILETIME {
        dwLowDateTime: (ticks & 0xFFFF_FFFF) as u32,
        dwHighDateTime: (ticks >> 32) as u32,
    };
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    unsafe {
        let handle = CreateFileW(
            windows::core::PCWSTR(wide.as_ptr()),
            FILE_WRITE_ATTRIBUTES.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )
        .unwrap();
        SetFileTime(handle, None, None, Some(&stamp as *const FILETIME)).unwrap();
        let _ = CloseHandle(handle);
    }
}

fn set_file_mtime(path: &std::path::Path, when: SystemTime) {
    fs::OpenOptions::new()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(when))
        .unwrap();
}

/// -a must imply -R (directory copied without -r) and -p (mtimes preserved
/// for files and for every directory level).
#[test]
fn archive_implies_recursive_and_preserves_timestamps() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("srcdir");
    fs::create_dir_all(src.join("sub")).unwrap();
    fs::write(src.join("sub").join("f.txt"), "DEEP").unwrap();

    let dir_when = stamp(1_600_000_000);
    let sub_when = stamp(1_650_000_000);
    let file_when = stamp(1_700_000_000);
    set_file_mtime(&src.join("sub").join("f.txt"), file_when);
    set_dir_modified(&src.join("sub"), sub_when);
    set_dir_modified(&src, dir_when);

    let dst = tmp.path().join("copy");
    // No -r/-R on the command line: -a must supply the recursion.
    cargo_bin_cmd!("cp")
        .arg("-a")
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();

    let copied = dst.join("sub").join("f.txt");
    assert_eq!(fs::read_to_string(&copied).unwrap(), "DEEP");
    assert_eq!(
        fs::metadata(&copied).unwrap().modified().unwrap(),
        file_when,
        "-a must preserve file mtime"
    );
    #[cfg(windows)]
    {
        assert_eq!(
            fs::metadata(&dst).unwrap().modified().unwrap(),
            dir_when,
            "-a must preserve top-level directory mtime"
        );
        assert_eq!(
            fs::metadata(dst.join("sub")).unwrap().modified().unwrap(),
            sub_when,
            "-a must preserve nested directory mtime"
        );
    }
}

/// -a combined with explicit -p/-r stays valid (duplicated flags are fine).
#[test]
fn archive_accepts_duplicate_p_and_r() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("f.txt");
    fs::write(&src, "X").unwrap();
    set_file_mtime(&src, stamp(1_600_000_000));

    cargo_bin_cmd!("cp")
        .args(["-a", "-p", "-r"])
        .arg(&src)
        .arg(tmp.path().join("g.txt"))
        .assert()
        .success();
    assert_eq!(
        fs::metadata(tmp.path().join("g.txt"))
            .unwrap()
            .modified()
            .unwrap(),
        stamp(1_600_000_000)
    );
}

/// Source newer than destination: -u copies.
#[test]
fn update_copies_when_source_is_newer() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("dst.txt");
    fs::write(&dst, "OLD").unwrap();
    set_file_mtime(&dst, stamp(1_600_000_000));
    fs::write(&src, "NEW").unwrap();
    set_file_mtime(&src, stamp(1_700_000_000));

    cargo_bin_cmd!("cp")
        .arg("-u")
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&dst).unwrap(), "NEW");
}

/// Source older than destination: -u skips silently and exits 0.
#[test]
fn update_skips_when_source_is_older() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("dst.txt");
    fs::write(&dst, "KEEP").unwrap();
    set_file_mtime(&dst, stamp(1_700_000_000));
    fs::write(&src, "STALE").unwrap();
    set_file_mtime(&src, stamp(1_600_000_000));

    cargo_bin_cmd!("cp")
        .arg("-u")
        .arg("-v")
        .arg(&src)
        .arg(&dst)
        .assert()
        .success()
        .stdout(predicates::str::is_empty());
    assert_eq!(fs::read_to_string(&dst).unwrap(), "KEEP");
}

/// Equal mtimes count as "not newer" (GNU 8.32 copies only on strictly
/// newer sources).
#[test]
fn update_treats_equal_mtime_as_no_copy() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("dst.txt");
    fs::write(&src, "SRC").unwrap();
    set_file_mtime(&src, stamp(1_650_000_000));
    fs::write(&dst, "DST").unwrap();
    set_file_mtime(&dst, stamp(1_650_000_000));

    cargo_bin_cmd!("cp")
        .arg("-u")
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&dst).unwrap(), "DST");
}

/// A missing destination is always copied, regardless of -u.
#[test]
fn update_copies_when_destination_is_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src.txt");
    fs::write(&src, "FRESH").unwrap();
    set_file_mtime(&src, stamp(1_600_000_000));

    cargo_bin_cmd!("cp")
        .arg("-u")
        .arg(&src)
        .arg(tmp.path().join("dst.txt"))
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(tmp.path().join("dst.txt")).unwrap(),
        "FRESH"
    );
}

/// The -u check runs before -i: an older source is skipped without a prompt.
#[test]
fn update_skips_before_interactive_prompt() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("dst.txt");
    fs::write(&dst, "KEEP").unwrap();
    set_file_mtime(&dst, stamp(1_700_000_000));
    fs::write(&src, "STALE").unwrap();
    set_file_mtime(&src, stamp(1_600_000_000));

    cargo_bin_cmd!("cp")
        .args(["-u", "-i"])
        .arg(&src)
        .arg(&dst)
        .write_stdin("y\n")
        .assert()
        .success()
        .stderr(predicates::str::contains("overwrite").not());
    assert_eq!(fs::read_to_string(&dst).unwrap(), "KEEP");
}

/// -n still suppresses the copy even when the source is newer.
#[test]
fn update_defers_to_no_clobber() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("dst.txt");
    fs::write(&dst, "KEEP").unwrap();
    set_file_mtime(&dst, stamp(1_600_000_000));
    fs::write(&src, "NEW").unwrap();
    set_file_mtime(&src, stamp(1_700_000_000));

    cargo_bin_cmd!("cp")
        .args(["-u", "-n"])
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&dst).unwrap(), "KEEP");
}

/// -f does not override -u: an older source is still skipped.
#[test]
fn update_wins_over_force_for_older_source() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src.txt");
    let dst = tmp.path().join("dst.txt");
    fs::write(&dst, "KEEP").unwrap();
    set_file_mtime(&dst, stamp(1_700_000_000));
    fs::write(&src, "STALE").unwrap();
    set_file_mtime(&src, stamp(1_600_000_000));

    cargo_bin_cmd!("cp")
        .args(["-u", "-f"])
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&dst).unwrap(), "KEEP");
}

/// -t DIR copies every operand into DIR; -v reports the resolved target.
#[test]
fn target_directory_accepts_multiple_sources() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    let b = tmp.path().join("b.txt");
    fs::write(&a, "A").unwrap();
    fs::write(&b, "B").unwrap();
    let out = tmp.path().join("out");
    fs::create_dir(&out).unwrap();

    cargo_bin_cmd!("cp")
        .arg("-t")
        .arg(&out)
        .arg(&a)
        .arg(&b)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(out.join("a.txt")).unwrap(), "A");
    assert_eq!(fs::read_to_string(out.join("b.txt")).unwrap(), "B");

    // GNU also accepts the option after the operands.
    let out2 = tmp.path().join("out2");
    fs::create_dir(&out2).unwrap();
    cargo_bin_cmd!("cp")
        .arg(&a)
        .arg(&b)
        .arg("-t")
        .arg(&out2)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(out2.join("a.txt")).unwrap(), "A");
    assert_eq!(fs::read_to_string(out2.join("b.txt")).unwrap(), "B");
}

/// A missing -t target fails like GNU ("failed to access").
#[test]
fn target_directory_missing_target_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    fs::write(&a, "A").unwrap();

    cargo_bin_cmd!("cp")
        .args(["-t"])
        .arg(tmp.path().join("nodir"))
        .arg(&a)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("failed to access"));
}

/// An existing non-directory -t target fails like GNU.
#[test]
fn target_directory_rejects_file_target() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    let file = tmp.path().join("plain.txt");
    fs::write(&a, "A").unwrap();
    fs::write(&file, "P").unwrap();

    cargo_bin_cmd!("cp")
        .args(["-t"])
        .arg(&file)
        .arg(&a)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("is not a directory"));
    assert_eq!(fs::read_to_string(&file).unwrap(), "P");
}

/// -t and -T are mutually exclusive (GNU: "cannot combine").
#[test]
fn target_directory_conflicts_with_no_target_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    fs::write(&a, "A").unwrap();
    let out = tmp.path().join("out");
    fs::create_dir(&out).unwrap();

    cargo_bin_cmd!("cp")
        .arg("-t")
        .arg(&out)
        .arg("-T")
        .arg(&a)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("cannot combine"));
}

/// -T never looks inside an existing directory destination.
#[test]
fn no_target_directory_rejects_directory_destination() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    fs::write(&a, "A").unwrap();
    let dir = tmp.path().join("dir");
    fs::create_dir(&dir).unwrap();

    cargo_bin_cmd!("cp")
        .arg("-T")
        .arg(&a)
        .arg(&dir)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("cannot overwrite directory"));
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
}

/// -T with a fresh destination creates a plain file.
#[test]
fn no_target_directory_creates_plain_file() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    fs::write(&a, "A").unwrap();

    cargo_bin_cmd!("cp")
        .arg("-T")
        .arg(&a)
        .arg(tmp.path().join("newfile"))
        .assert()
        .success();
    assert_eq!(fs::read_to_string(tmp.path().join("newfile")).unwrap(), "A");
}

/// -T with more than two operands is a usage error and copies nothing.
#[test]
fn no_target_directory_rejects_extra_operands() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    let b = tmp.path().join("b.txt");
    fs::write(&a, "A").unwrap();
    fs::write(&b, "B").unwrap();

    cargo_bin_cmd!("cp")
        .arg("-T")
        .arg(&a)
        .arg(&b)
        .arg(tmp.path().join("c.txt"))
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("extra operand"));
    assert_eq!(fs::read_to_string(&b).unwrap(), "B", "b must survive");
    assert!(!tmp.path().join("c.txt").exists());
}

/// -T with a recursive directory source copies the contents into the
/// destination itself (no source-name level in between).
#[test]
fn no_target_directory_copies_dir_contents_into_dest() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("srcdir");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("f.txt"), "F").unwrap();
    let dst = tmp.path().join("dstdir");
    fs::create_dir(&dst).unwrap();

    cargo_bin_cmd!("cp")
        .args(["-rT"])
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(dst.join("f.txt")).unwrap(), "F");
    assert!(!dst.join("srcdir").exists(), "no extra name level");
}

/// One operand without -t is a GNU usage error, not a clap error.
#[test]
fn missing_destination_operand_reports_gnu_error() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a.txt");
    fs::write(&a, "A").unwrap();

    cargo_bin_cmd!("cp")
        .arg(&a)
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "missing destination file operand after",
        ));
}

/// -u also applies to files inside recursively copied directories.
#[test]
fn update_applies_per_entry_in_recursive_copy() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src");
    let dst = tmp.path().join("dst");
    fs::create_dir_all(src.join("sub")).unwrap();
    // cp -ru src dst with an existing directory dst lands as dst/src/...
    fs::create_dir_all(dst.join("src").join("sub")).unwrap();

    fs::write(src.join("sub").join("f.txt"), "STALE").unwrap();
    set_file_mtime(&src.join("sub").join("f.txt"), stamp(1_600_000_000));
    fs::write(dst.join("src").join("sub").join("f.txt"), "KEEP").unwrap();
    set_file_mtime(
        &dst.join("src").join("sub").join("f.txt"),
        stamp(1_700_000_000),
    );

    fs::write(src.join("g.txt"), "NEW").unwrap();
    set_file_mtime(&src.join("g.txt"), stamp(1_750_000_000));
    fs::write(dst.join("src").join("g.txt"), "OLD").unwrap();
    set_file_mtime(&dst.join("src").join("g.txt"), stamp(1_500_000_000));

    cargo_bin_cmd!("cp")
        .args(["-ru"])
        .arg(&src)
        .arg(&dst)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(dst.join("src").join("sub").join("f.txt")).unwrap(),
        "KEEP",
        "older source entry must be skipped"
    );
    assert_eq!(
        fs::read_to_string(dst.join("src").join("g.txt")).unwrap(),
        "NEW",
        "newer source entry must be copied"
    );
}

// --- hard links (-l/--link) -------------------------------------------------

/// `cp -l` creates a hard link: writing through the link changes the source.
#[test]
fn link_shares_content_with_source() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dest = dir.path().join("dest.txt");
    fs::write(&src, "original").unwrap();
    cargo_bin_cmd!("cp")
        .args(["-l"])
        .arg(&src)
        .arg(&dest)
        .assert()
        .success();
    fs::write(&dest, "modified").unwrap();
    assert_eq!(fs::read_to_string(&src).unwrap(), "modified");
}

/// Without -f an existing destination rejects the hard link like GNU.
#[test]
fn link_to_existing_destination_errors() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dest = dir.path().join("dest.txt");
    fs::write(&src, "new").unwrap();
    fs::write(&dest, "old").unwrap();
    cargo_bin_cmd!("cp")
        .args(["-l"])
        .arg(&src)
        .arg(&dest)
        .assert()
        .failure()
        .stderr(predicates::str::contains("File exists"))
        .stderr(predicates::str::contains("cannot create hard link"));
    assert_eq!(fs::read_to_string(&dest).unwrap(), "old");
}

/// -f unlinks the existing destination and links anyway.
#[test]
fn link_force_replaces_existing_destination() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dest = dir.path().join("dest.txt");
    fs::write(&src, "new").unwrap();
    fs::write(&dest, "old").unwrap();
    cargo_bin_cmd!("cp")
        .args(["-lf"])
        .arg(&src)
        .arg(&dest)
        .assert()
        .success();
    assert_eq!(fs::read_to_string(&dest).unwrap(), "new");
    fs::write(&dest, "via-link").unwrap();
    assert_eq!(fs::read_to_string(&src).unwrap(), "via-link");
}

/// -l with -r hard-links every file in the tree.
#[test]
fn link_recursive_hard_links_files() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("sub");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("inner.txt"), "inner").unwrap();
    cargo_bin_cmd!("cp")
        .args(["-lr"])
        .arg(sub.join("sub"))
        .arg(dir.path().join("copy"))
        .assert()
        .failure();
    // retry with correct operand order (source dir, dest dir)
    let out = dir.path().join("tree-copy");
    cargo_bin_cmd!("cp")
        .args(["-lr"])
        .arg(&sub)
        .arg(&out)
        .assert()
        .success();
    fs::write(out.join("inner.txt"), "via-link").unwrap();
    assert_eq!(
        fs::read_to_string(sub.join("inner.txt")).unwrap(),
        "via-link"
    );
}

/// GNU refuses -l with -s.
#[test]
fn hard_and_symbolic_links_conflict() {
    cargo_bin_cmd!("cp")
        .args(["-l", "-s", "src.txt", "dest.txt"])
        .assert()
        .failure()
        .stderr(
            "cp: cannot make both hard and symbolic links\nTry 'cp --help' for more information.\n",
        );
}

// --- backups (-b/--backup, -S/--suffix) -------------------------------------

/// Plain -b renames the destination to `name~` before copying.
#[test]
fn backup_simple_default_suffix() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dest = dir.path().join("dest.txt");
    fs::write(&src, "new").unwrap();
    fs::write(&dest, "old").unwrap();
    cargo_bin_cmd!("cp")
        .args(["-b"])
        .arg(&src)
        .arg(&dest)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(dir.path().join("dest.txt~")).unwrap(),
        "old"
    );
    assert_eq!(fs::read_to_string(&dest).unwrap(), "new");
}

/// A new destination gets no backup.
#[test]
fn backup_not_made_for_new_destination() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    fs::write(&src, "new").unwrap();
    cargo_bin_cmd!("cp")
        .args(["-b"])
        .arg(&src)
        .arg(dir.path().join("fresh.txt"))
        .assert()
        .success();
    let leftovers: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(leftovers.len(), 2);
}

/// --backup=numbered produces name.~1~, name.~2~, ... and plain -b continues
/// numbering when numbered backups already exist (GNU "existing" mode).
#[test]
fn backup_numbered_sequence_and_existing_mode() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dest = dir.path().join("dest.txt");
    fs::write(&src, "new").unwrap();
    fs::write(&dest, "v0").unwrap();
    cargo_bin_cmd!("cp")
        .args(["--backup=numbered"])
        .arg(&src)
        .arg(&dest)
        .assert()
        .success();
    fs::write(&dest, "v1").unwrap();
    cargo_bin_cmd!("cp")
        .args(["--backup=numbered"])
        .arg(&src)
        .arg(&dest)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(dir.path().join("dest.txt.~1~")).unwrap(),
        "v0"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("dest.txt.~2~")).unwrap(),
        "v1"
    );
    fs::write(&dest, "v2").unwrap();
    cargo_bin_cmd!("cp")
        .args(["-b"])
        .arg(&src)
        .arg(&dest)
        .assert()
        .success();
    // existing mode keeps numbering while numbered backups are around
    assert_eq!(
        fs::read_to_string(dir.path().join("dest.txt.~3~")).unwrap(),
        "v2"
    );
}

/// -S overrides the simple backup suffix; --backup=simple forces it even
/// when numbered backups exist.
#[test]
fn backup_suffix_override() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dest = dir.path().join("dest.txt");
    fs::write(&src, "new").unwrap();
    fs::write(&dest, "old").unwrap();
    cargo_bin_cmd!("cp")
        .args(["--backup=simple", "-S", ".bak"])
        .arg(&src)
        .arg(&dest)
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(dir.path().join("dest.txt.bak")).unwrap(),
        "old"
    );
}

/// --backup=none makes no backups at all.
#[test]
fn backup_none_disables_backups() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dest = dir.path().join("dest.txt");
    fs::write(&src, "new").unwrap();
    fs::write(&dest, "old").unwrap();
    cargo_bin_cmd!("cp")
        .args(["--backup=none"])
        .arg(&src)
        .arg(&dest)
        .assert()
        .success();
    let leftovers: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(leftovers.len(), 2);
}

/// GNU rejects -b with -n and invalid --backup values.
#[test]
fn backup_option_conflicts_and_validation() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    fs::write(&src, "new").unwrap();
    cargo_bin_cmd!("cp")
        .args(["-b", "-n"])
        .arg(&src)
        .arg(dir.path().join("dest.txt"))
        .assert()
        .failure()
        .stderr(
            "cp: options --backup and --no-clobber are mutually exclusive\nTry 'cp --help' for more information.\n",
        );
    cargo_bin_cmd!("cp")
        .args(["--backup=bogus"])
        .arg(&src)
        .arg(dir.path().join("dest.txt"))
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "cp: invalid argument 'bogus' for '--backup'",
        ));
}

/// Backups also happen inside recursive copies when a tree entry is replaced.
#[test]
fn backup_applies_inside_recursive_copy() {
    let dir = tempfile::tempdir().unwrap();
    let src_sub = dir.path().join("src-sub");
    let dst_sub = dir.path().join("dst-sub");
    fs::create_dir(&src_sub).unwrap();
    fs::create_dir(&dst_sub).unwrap();
    fs::write(src_sub.join("f.txt"), "new").unwrap();
    fs::write(dst_sub.join("f.txt"), "old").unwrap();
    // Copy the contents of src-sub into the existing dst-sub (src/. form).
    cargo_bin_cmd!("cp")
        .args(["-rb"])
        .arg(src_sub.join("."))
        .arg(&dst_sub)
        .assert()
        .success();
    // dst-sub/f.txt was overwritten; the old content went to f.txt~
    assert_eq!(fs::read_to_string(dst_sub.join("f.txt~")).unwrap(), "old");
    assert_eq!(fs::read_to_string(dst_sub.join("f.txt")).unwrap(), "new");
}

// --- VERSION_CONTROL environment variable -----------------------------------

/// Bare -b takes its mode from VERSION_CONTROL: numbered produces .~N~,
/// none produces no backup.
#[test]
fn version_control_env_selects_mode() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dest = dir.path().join("dest.txt");
    fs::write(&src, "new").unwrap();
    fs::write(&dest, "old").unwrap();
    cargo_bin_cmd!("cp")
        .env("VERSION_CONTROL", "numbered")
        .args(["-b"])
        .arg(&src)
        .arg(&dest)
        .assert()
        .success();
    assert!(dir.path().join("dest.txt.~1~").exists());

    fs::write(&dest, "newer").unwrap();
    cargo_bin_cmd!("cp")
        .env("VERSION_CONTROL", "none")
        .args(["-b"])
        .arg(&src)
        .arg(&dest)
        .assert()
        .success();
    // none: the old backup stays, no new file appears.
    let leftovers: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(leftovers.len(), 3);
}

/// An invalid VERSION_CONTROL value fails with GNU's argument error.
#[test]
fn version_control_invalid_value_errors() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    fs::write(&src, "new").unwrap();
    cargo_bin_cmd!("cp")
        .env("VERSION_CONTROL", "bogus")
        .args(["-b"])
        .arg(&src)
        .arg(dir.path().join("dest.txt"))
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "cp: invalid argument 'bogus' for '$VERSION_CONTROL'",
        ))
        .stderr(predicates::str::contains("  - 'numbered', 't'"));
}

/// Without -b, VERSION_CONTROL does not enable backups.
#[test]
fn version_control_without_backup_flag_makes_no_backup() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("src.txt");
    fs::write(&src, "new").unwrap();
    cargo_bin_cmd!("cp")
        .env("VERSION_CONTROL", "simple")
        .arg(&src)
        .arg(dir.path().join("dest.txt"))
        .assert()
        .success();
    let leftovers: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(leftovers.len(), 2);
}
