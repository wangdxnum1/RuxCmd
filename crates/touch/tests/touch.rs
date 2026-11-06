//! Regression tests for P0 #14: touch on an existing file must actually
//! update timestamps (SetFileTime), directories and read-only files must be
//! touchable, and `-c` must never create files.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::fs::FileTimes;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Open `path` for writing and force its timestamps to the given value so we
/// do not depend on sleep-based timing.
fn force_times(path: &Path, atime: SystemTime, mtime: SystemTime) {
    let f = fs::OpenOptions::new()
        .write(true)
        .open(path)
        .expect("open for pre-setting timestamps");
    f.set_times(FileTimes::new().set_accessed(atime).set_modified(mtime))
        .expect("pre-set timestamps");
}

fn mtime(path: &Path) -> SystemTime {
    fs::metadata(path)
        .expect("metadata after touch")
        .modified()
        .expect("mtime after touch")
}

fn atime(path: &Path) -> SystemTime {
    fs::metadata(path)
        .expect("metadata after touch")
        .accessed()
        .expect("atime after touch")
}

// --- Core P0: touching an existing file must change its timestamps ---

#[test]
fn touch_existing_file_updates_mtime() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("existing.txt");
    fs::write(&file, b"payload").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&file, old, old);
    let before = mtime(&file);

    cargo_bin_cmd!("touch").arg(&file).assert().success();

    let after = mtime(&file);
    assert!(
        after > before,
        "mtime must advance after touching an existing file (before={before:?}, after={after:?})"
    );
    // The file content must be preserved.
    assert_eq!(fs::read(&file).unwrap(), b"payload");
}

#[test]
fn touch_existing_file_updates_atime() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("existing-atime.txt");
    fs::write(&file, b"payload").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&file, old, old);
    let before = atime(&file);

    cargo_bin_cmd!("touch").arg(&file).assert().success();

    let after = atime(&file);
    assert!(
        after > before,
        "atime must advance after touching an existing file (before={before:?}, after={after:?})"
    );
}

#[test]
fn touch_m_updates_mtime_but_leaves_atime() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("m-only.txt");
    fs::write(&file, b"payload").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&file, old, old);

    cargo_bin_cmd!("touch")
        .arg("-m")
        .arg(&file)
        .assert()
        .success();

    assert!(mtime(&file) > old, "-m must update mtime");
    assert!(
        atime(&file) <= old + Duration::from_secs(2),
        "-m must not update atime (got {:?}, pre-set {:?})",
        atime(&file),
        old
    );
}

#[test]
fn touch_a_updates_atime_but_leaves_mtime() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a-only.txt");
    fs::write(&file, b"payload").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&file, old, old);

    cargo_bin_cmd!("touch")
        .arg("-a")
        .arg(&file)
        .assert()
        .success();

    assert!(atime(&file) > old, "-a must update atime");
    assert!(
        mtime(&file) <= old + Duration::from_secs(2),
        "-a must not update mtime (got {:?}, pre-set {:?})",
        mtime(&file),
        old
    );
}

// --- -c / --no-create ---

#[test]
fn no_create_does_not_create_missing_file() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.txt");

    cargo_bin_cmd!("touch")
        .arg("-c")
        .arg(&missing)
        .assert()
        .success();
    assert!(!missing.exists(), "-c must not create the file");

    cargo_bin_cmd!("touch")
        .arg("--no-create")
        .arg(&missing)
        .assert()
        .success();
    assert!(!missing.exists(), "--no-create must not create the file");
}

#[test]
fn no_create_on_existing_file_still_updates_mtime() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("existing.txt");
    fs::write(&file, b"payload").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&file, old, old);

    cargo_bin_cmd!("touch")
        .arg("-c")
        .arg(&file)
        .assert()
        .success();

    assert!(
        mtime(&file) > old,
        "-c on an existing file must still update mtime"
    );
}

// --- Directories and read-only files ---

#[test]
fn touch_directory_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let sub = dir.path().join("subdir");
    fs::create_dir(&sub).unwrap();

    cargo_bin_cmd!("touch").arg(&sub).assert().success();
}

#[test]
fn touch_readonly_file_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("readonly.txt");
    fs::write(&file, b"payload").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&file, old, old);

    // Make the file read-only, like `attrib +r`.
    let mut perms = fs::metadata(&file).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&file, perms).unwrap();

    cargo_bin_cmd!("touch").arg(&file).assert().success();
    assert!(mtime(&file) > old, "mtime must advance on a read-only file");

    // Restore write access so the temp dir can be cleaned up; clearing the
    // read-only bit is the whole point here.
    let mut perms = fs::metadata(&file).unwrap().permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    perms.set_readonly(false);
    fs::set_permissions(&file, perms).unwrap();
}

// --- Creation of missing files ---

#[test]
fn touch_missing_file_creates_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("created.txt");
    assert!(!file.exists());

    cargo_bin_cmd!("touch").arg(&file).assert().success();

    assert!(file.exists(), "touch must create the file");
    let meta = fs::metadata(&file).unwrap();
    assert!(meta.is_file());
    assert_eq!(meta.len(), 0, "created file must be empty");
}

#[test]
fn touch_multiple_files_updates_all() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.txt");
    let b = dir.path().join("b.txt");
    fs::write(&a, b"").unwrap();
    fs::write(&b, b"").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&a, old, old);
    force_times(&b, old, old);

    cargo_bin_cmd!("touch").arg(&a).arg(&b).assert().success();

    assert!(mtime(&a) > old, "first file must be touched");
    assert!(mtime(&b) > old, "second file must be touched");
}

#[test]
fn missing_operand_fails_with_error() {
    cargo_bin_cmd!("touch")
        .assert()
        .code(1)
        .stderr(predicates::str::contains("missing file operand"));
}

#[test]
fn unwritable_path_fails_with_error() {
    cargo_bin_cmd!("touch")
        .arg("Z:\\no\\such\\directory\\file.txt")
        .assert()
        .code(1)
        .stderr(predicates::str::contains("touch:"));
}

// --- 0.4.6 GNU alignment: -d/--date, -t STAMP, -r/--reference ---------------
//
// Measured against GNU coreutils 8.32 (`touch 8.32`, local Git Bash):
// - `-d` and `-t` accept a single time source applied to atime AND mtime
//   (then narrowed by -a/-m);
// - `-d` overrides `-r` (no conflict), while `-t` combined with `-d` or `-r`
//   fails with "cannot specify times from more than one source";
// - parse failures report `touch: invalid date format '<x>'` and exit 1;
// - `-t` century rule: YY 00-68 -> 2000..2068, 69-99 -> 1969..1999.

/// Minimal kernel32 bindings so the tests can compute local-time
/// expectations through the OS timezone database (no chrono dependency, and
/// the assertions hold on any Windows machine regardless of its time zone).
mod os_time {
    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    pub struct Systime {
        pub year: u16,
        pub month: u16,
        pub day_of_week: u16,
        pub day: u16,
        pub hour: u16,
        pub minute: u16,
        pub second: u16,
        pub milliseconds: u16,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    pub struct Filetime {
        pub low: u32,
        pub high: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub fn TzSpecificLocalTimeToSystemTime(
            tzi: *const core::ffi::c_void,
            local: *const Systime,
            utc: *mut Systime,
        ) -> i32;
        pub fn SystemTimeToFileTime(st: *const Systime, ft: *mut Filetime) -> i32;
        pub fn GetLocalTime(st: *mut Systime);
    }

    /// Epoch seconds for a LOCAL wall-clock time per the OS timezone rules.
    pub fn local_epoch(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> i64 {
        let local = Systime {
            year: year as u16,
            month: month as u16,
            day_of_week: 0,
            day: day as u16,
            hour: hour as u16,
            minute: minute as u16,
            second: second as u16,
            milliseconds: 0,
        };
        unsafe {
            let mut utc = Systime::default();
            let converted = TzSpecificLocalTimeToSystemTime(std::ptr::null(), &local, &mut utc);
            let mut ft = Filetime::default();
            if converted != 0 {
                SystemTimeToFileTime(&utc, &mut ft);
            }
            // On a conversion failure (DST gap) fall back to plain UTC.
            let intervals = ((ft.high as i64) << 32) | ft.low as i64;
            (intervals - 116_444_736_000_000_000) / 10_000_000
        }
    }

    pub fn current_year() -> i32 {
        unsafe {
            let mut st = Systime::default();
            GetLocalTime(&mut st);
            st.year as i32
        }
    }
}

fn epoch_time(secs: i64) -> SystemTime {
    if secs >= 0 {
        UNIX_EPOCH + Duration::from_secs(secs as u64)
    } else {
        UNIX_EPOCH - Duration::from_secs(secs.unsigned_abs())
    }
}

// --- -d/--date --------------------------------------------------------------

/// `@SECONDS` is timezone-independent by construction: the file time must be
/// exactly the given epoch instant.
#[test]
fn date_epoch_seconds_set_exact_times() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("epoch.txt");

    cargo_bin_cmd!("touch")
        .arg("-d")
        .arg("@1791024000")
        .arg(&file)
        .assert()
        .success();

    let expected = epoch_time(1_791_024_000);
    assert_eq!(mtime(&file), expected, "mtime must be the @epoch instant");
    assert_eq!(atime(&file), expected, "-d sets atime as well as mtime");
}

#[test]
fn date_datetime_sets_both_times_to_local_wall_clock() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("wall.txt");

    cargo_bin_cmd!("touch")
        .arg("-d")
        .arg("2025-01-02 03:04:05")
        .arg(&file)
        .assert()
        .success();

    let expected = epoch_time(os_time::local_epoch(2025, 1, 2, 3, 4, 5));
    assert_eq!(mtime(&file), expected);
    assert_eq!(atime(&file), expected);
}

/// The ISO `T` separator and the space separator denote the same instant.
#[test]
fn date_iso_t_form_and_space_form_agree() {
    let dir = tempfile::tempdir().unwrap();
    let t_form = dir.path().join("t-form.txt");
    let space_form = dir.path().join("space-form.txt");

    cargo_bin_cmd!("touch")
        .arg("-d")
        .arg("2025-01-02T03:04:05")
        .arg(&t_form)
        .assert()
        .success();
    cargo_bin_cmd!("touch")
        .arg("-d")
        .arg("2025-01-02 03:04:05")
        .arg(&space_form)
        .assert()
        .success();

    assert_eq!(mtime(&t_form), mtime(&space_form));
}

/// A bare date is midnight of that day (GNU measured: `2025-01-02 00:00`).
#[test]
fn date_only_sets_midnight() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("midnight.txt");

    cargo_bin_cmd!("touch")
        .arg("-d")
        .arg("2025-01-02")
        .arg(&file)
        .assert()
        .success();

    assert_eq!(
        mtime(&file),
        epoch_time(os_time::local_epoch(2025, 1, 2, 0, 0, 0))
    );
}

/// Minutes without seconds are accepted (GNU measured:
/// `-d 2025-01-02T03:04` -> `03:04:00`).
#[test]
fn date_minutes_without_seconds() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("no-seconds.txt");

    cargo_bin_cmd!("touch")
        .arg("-d")
        .arg("2025-01-02T03:04")
        .arg(&file)
        .assert()
        .success();

    assert_eq!(
        mtime(&file),
        epoch_time(os_time::local_epoch(2025, 1, 2, 3, 4, 0))
    );
}

// --- -t STAMP ---------------------------------------------------------------

/// `-t CCYYMMDDhhmm.ss` (GNU measured: `202501011200.30` -> `12:00:30`).
#[test]
fn stamp_full_form_with_seconds() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("stamp.txt");

    cargo_bin_cmd!("touch")
        .arg("-t")
        .arg("202503040506.07")
        .arg(&file)
        .assert()
        .success();

    let expected = epoch_time(os_time::local_epoch(2025, 3, 4, 5, 6, 7));
    assert_eq!(mtime(&file), expected);
    assert_eq!(atime(&file), expected);
}

/// POSIX century rule for two-digit years (GNU measured:
/// `-t 2503040506` -> 2025, `-t 6901010000` -> 1969).
#[test]
fn stamp_two_digit_years_follow_posix_century_rule() {
    let dir = tempfile::tempdir().unwrap();
    let in_2000s = dir.path().join("yy-25.txt");
    let in_1900s = dir.path().join("yy-69.txt");

    cargo_bin_cmd!("touch")
        .arg("-t")
        .arg("2503040506")
        .arg(&in_2000s)
        .assert()
        .success();
    cargo_bin_cmd!("touch")
        .arg("-t")
        .arg("6901010000")
        .arg(&in_1900s)
        .assert()
        .success();

    assert_eq!(
        mtime(&in_2000s),
        epoch_time(os_time::local_epoch(2025, 3, 4, 5, 6, 0))
    );
    assert_eq!(
        mtime(&in_1900s),
        epoch_time(os_time::local_epoch(1969, 1, 1, 0, 0, 0))
    );
}

/// `-t MMDDhhmm` uses the current year (POSIX).
#[test]
fn stamp_eight_digits_use_current_year() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("stamp-current-year.txt");

    cargo_bin_cmd!("touch")
        .arg("-t")
        .arg("03040506")
        .arg(&file)
        .assert()
        .success();

    let year = os_time::current_year();
    assert_eq!(
        mtime(&file),
        epoch_time(os_time::local_epoch(year, 3, 4, 5, 6, 0))
    );
}

// --- -r/--reference ---------------------------------------------------------

#[test]
fn reference_copies_atime_and_mtime() {
    let dir = tempfile::tempdir().unwrap();
    let reference = dir.path().join("ref.txt");
    let target = dir.path().join("target.txt");
    fs::write(&reference, b"ref").unwrap();
    fs::write(&target, b"target").unwrap();

    let ref_atime = SystemTime::now() - Duration::from_secs(7200);
    let ref_mtime = SystemTime::now() - Duration::from_secs(3600);
    force_times(&reference, ref_atime, ref_mtime);

    cargo_bin_cmd!("touch")
        .arg("-r")
        .arg(&reference)
        .arg(&target)
        .assert()
        .success();

    assert_eq!(mtime(&target), ref_mtime, "-r copies the reference mtime");
    assert_eq!(atime(&target), ref_atime, "-r copies the reference atime");
}

/// `-a`/`-m` narrow which of the reference times is copied (GNU semantics).
#[test]
fn reference_with_a_and_m_select_copied_times() {
    let dir = tempfile::tempdir().unwrap();
    let reference = dir.path().join("ref.txt");
    let a_target = dir.path().join("a-target.txt");
    let m_target = dir.path().join("m-target.txt");
    fs::write(&reference, b"ref").unwrap();
    fs::write(&a_target, b"a").unwrap();
    fs::write(&m_target, b"m").unwrap();

    let ref_atime = SystemTime::now() - Duration::from_secs(7200);
    let ref_mtime = SystemTime::now() - Duration::from_secs(3600);
    force_times(&reference, ref_atime, ref_mtime);

    let untouched = SystemTime::now() - Duration::from_secs(100);
    force_times(&a_target, untouched, untouched);
    force_times(&m_target, untouched, untouched);

    cargo_bin_cmd!("touch")
        .arg("-r")
        .arg(&reference)
        .arg("-a")
        .arg(&a_target)
        .assert()
        .success();
    cargo_bin_cmd!("touch")
        .arg("-r")
        .arg(&reference)
        .arg("-m")
        .arg(&m_target)
        .assert()
        .success();

    assert_eq!(atime(&a_target), ref_atime, "-a copies the reference atime");
    assert_eq!(
        mtime(&a_target),
        untouched,
        "-a must not touch mtime of the target"
    );
    assert_eq!(mtime(&m_target), ref_mtime, "-m copies the reference mtime");
    assert_eq!(
        atime(&m_target),
        untouched,
        "-m must not touch atime of the target"
    );
}

/// GNU allows `-d` together with `-r` (no conflict) and `-d` wins.
#[test]
fn date_overrides_reference() {
    let dir = tempfile::tempdir().unwrap();
    let reference = dir.path().join("ref.txt");
    let target = dir.path().join("target.txt");
    fs::write(&reference, b"ref").unwrap();
    fs::write(&target, b"target").unwrap();

    force_times(&reference, SystemTime::now(), SystemTime::now());

    cargo_bin_cmd!("touch")
        .arg("-r")
        .arg(&reference)
        .arg("-d")
        .arg("@1791024000")
        .arg(&target)
        .assert()
        .success();

    assert_eq!(
        mtime(&target),
        epoch_time(1_791_024_000),
        "-d must win over -r"
    );
}

/// GNU rejects `-t` together with `-d` or `-r`.
#[test]
fn stamp_conflicts_with_reference_and_date() {
    let dir = tempfile::tempdir().unwrap();
    let reference = dir.path().join("ref.txt");
    fs::write(&reference, b"ref").unwrap();
    let target = dir.path().join("target.txt");

    cargo_bin_cmd!("touch")
        .arg("-t")
        .arg("202501011200")
        .arg("-r")
        .arg(&reference)
        .arg(&target)
        .assert()
        .code(1)
        .stderr(predicates::str::contains(
            "cannot specify times from more than one source",
        ));

    cargo_bin_cmd!("touch")
        .arg("-d")
        .arg("2025-01-01")
        .arg("-t")
        .arg("202501011200")
        .arg(&target)
        .assert()
        .code(1)
        .stderr(predicates::str::contains(
            "cannot specify times from more than one source",
        ));
}

// --- error messages (GNU wording: `touch: invalid date format '<x>'`) -------

/// Relative date expressions are outside the supported subset and must fail
/// with the GNU message instead of silently using "now".
#[test]
fn invalid_dates_report_gnu_message_exit_1() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("bad.txt");

    for bad in ["not a date", "1 day ago", "yesterday", "2025-02-30"] {
        cargo_bin_cmd!("touch")
            .arg("-d")
            .arg(bad)
            .arg(&file)
            .assert()
            .code(1)
            .stderr(predicates::str::contains(&format!(
                "touch: invalid date format '{bad}'"
            )));
    }

    cargo_bin_cmd!("touch")
        .arg("-t")
        .arg("1234")
        .arg(&file)
        .assert()
        .code(1)
        .stderr(predicates::str::contains(
            "touch: invalid date format '1234'",
        ));
}

/// A failing `-r` reference aborts before any file is touched (GNU message).
#[test]
fn missing_reference_reports_gnu_message_exit_1() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.txt");

    cargo_bin_cmd!("touch")
        .arg("-r")
        .arg(dir.path().join("no-such-ref"))
        .arg(&target)
        .assert()
        .code(1)
        .stderr(predicates::str::contains(
            "touch: failed to get attributes of",
        ))
        .stderr(predicates::str::contains("No such file or directory"));
    assert!(
        !target.exists(),
        "no target may be created after a -r failure"
    );
}

/// `touch -d X missing` creates the file with the requested times.
#[test]
fn date_on_missing_file_creates_it() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("created-with-date.txt");
    assert!(!file.exists());

    cargo_bin_cmd!("touch")
        .arg("-d")
        .arg("@1791024000")
        .arg(&file)
        .assert()
        .success();

    assert!(file.exists());
    assert_eq!(mtime(&file), epoch_time(1_791_024_000));
}

/// `-a` selects only the atime when a date source is given.
#[test]
fn date_with_a_touches_only_atime() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("a-date.txt");
    fs::write(&file, b"payload").unwrap();

    let old = SystemTime::now() - Duration::from_secs(3600);
    force_times(&file, old, old);

    cargo_bin_cmd!("touch")
        .arg("-a")
        .arg("-d")
        .arg("@1791024000")
        .arg(&file)
        .assert()
        .success();

    assert_eq!(atime(&file), epoch_time(1_791_024_000), "-a -d sets atime");
    assert_eq!(mtime(&file), old, "-a -d must not touch mtime");
}
