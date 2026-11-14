mod cli;

use clap::Parser;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE, SYSTEMTIME};
use windows::Win32::Storage::FileSystem::{
    CREATE_NEW, CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_READ_ATTRIBUTES, FILE_SHARE_READ,
    FILE_SHARE_WRITE, FILE_WRITE_ATTRIBUTES, GetFileTime, OPEN_EXISTING, SetFileTime,
};
use windows::core::{HRESULT, PCWSTR};

/// Local wall-clock time as parsed from `-d`/`-t`, in the OS local timezone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WallClock {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

/// A resolved `-d` argument: epoch seconds for `@SECONDS`, wall clock for the
/// ISO subset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DateSource {
    Epoch(i64),
    Wall(WallClock),
}

/// The source of the times to apply, resolved once up front like GNU:
/// -r > -d/-t > now; parse errors and reference errors abort before any file
/// is touched.
#[derive(Clone, Copy)]
enum Source {
    Now,
    /// Fixed atime/mtime pair (from -d/-t/-r), narrowed by -a/-m per file.
    Fixed {
        atime: FILETIME,
        mtime: FILETIME,
    },
}

fn main() {
    let args = cli::Args::parse();

    if args.files.is_empty() {
        eprintln!("touch: missing file operand");
        std::process::exit(1);
    }

    // GNU resolves the time source once (reference stat, date parse) before
    // the file loop; a failure aborts without touching anything.
    let source = match resolve_source(&args) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("touch: {error}");
            std::process::exit(1);
        }
    };

    let mut exit_code = 0i32;

    for path in &args.files {
        if let Err(e) = touch_file(path, args.no_create, args.atime, args.mtime, source) {
            eprintln!("touch: {}: {}", path.display(), e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

// --- time source resolution -------------------------------------------------

/// Determines the time source following the GNU rules measured on coreutils
/// 8.32: `-t` conflicts with both `-d` and `-r` ("cannot specify times from
/// more than one source"), `-d` silently wins over `-r`, and repeated flags
/// take the last value.
fn resolve_source(args: &cli::Args) -> Result<Source, String> {
    if let Some(stamp) = &args.stamp {
        if args.date.is_some() || args.reference.is_some() {
            return Err("cannot specify times from more than one source".to_string());
        }
        let wall = parse_stamp(stamp).map_err(|()| invalid_date(stamp))?;
        return Ok(Source::Fixed {
            atime: wall_to_filetime(wall)?,
            mtime: wall_to_filetime(wall)?,
        });
    }
    if let Some(date) = &args.date {
        return match parse_date(date).map_err(|()| invalid_date(date))? {
            DateSource::Epoch(epoch) => {
                let ft = epoch_to_filetime(epoch);
                Ok(Source::Fixed {
                    atime: ft,
                    mtime: ft,
                })
            }
            DateSource::Wall(wall) => Ok(Source::Fixed {
                atime: wall_to_filetime(wall)?,
                mtime: wall_to_filetime(wall)?,
            }),
        };
    }
    if let Some(reference) = &args.reference {
        let (atime, mtime) = reference_times(reference)?;
        return Ok(Source::Fixed { atime, mtime });
    }
    Ok(Source::Now)
}

fn invalid_date(raw: &str) -> String {
    // GNU coreutils wording, including for bad `-t` stamps.
    format!("invalid date format '{raw}'")
}

// --- date parsing (declared GNU subset) -------------------------------------

/// `-d/--date` subset: `@SECONDS`, relative expressions (`now`, `yesterday`,
/// `N unit(s) [ago]`), `YYYY-MM-DD`, `YYYY-MM-DDTHH:MM[:SS]` and
/// `YYYY-MM-DD HH:MM[:SS]`, all interpreted as local time.
fn parse_date(raw: &str) -> Result<DateSource, ()> {
    if let Some(shift) = ruxcmd_core::timefmt::parse_relative(raw) {
        let instant = ruxcmd_core::timefmt::apply_relative(shift).ok_or(())?;
        let secs = instant
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or_else(|_| 0);
        return Ok(DateSource::Epoch(secs));
    }
    if let Some(seconds) = raw.strip_prefix('@') {
        // Integer seconds only; GNU's fractional "@12.5" form is not in the
        // supported subset.
        return Ok(DateSource::Epoch(seconds.parse::<i64>().map_err(|_| ())?));
    }
    let (date_part, time_part) = match raw.find(['T', ' ']) {
        Some(index) => (&raw[..index], Some(&raw[index + 1..])),
        None => (raw, None),
    };
    let (year, month, day) = parse_ymd(date_part)?;
    let (hour, minute, second) = match time_part {
        None => (0, 0, 0),
        Some(time) => parse_hms(time)?,
    };
    validate_wall(WallClock {
        year,
        month,
        day,
        hour,
        minute,
        second,
    })?;
    Ok(DateSource::Wall(WallClock {
        year,
        month,
        day,
        hour,
        minute,
        second,
    }))
}

/// `-t STAMP` POSIX form: `[[CC]YY]MMDDhhmm[.ss]` (8, 10 or 12 digits plus an
/// optional two-digit second part), interpreted as local time.
fn parse_stamp(raw: &str) -> Result<WallClock, ()> {
    let (main, second) = match raw.split_once('.') {
        Some((main, seconds)) => {
            if seconds.len() != 2 || !seconds.bytes().all(|b| b.is_ascii_digit()) {
                return Err(());
            }
            (main, seconds.parse::<u32>().map_err(|_| ())?)
        }
        None => (raw, 0),
    };
    if main.is_empty() || !main.bytes().all(|b| b.is_ascii_digit()) {
        return Err(());
    }
    let digits = |part: &str| part.parse::<u32>().map_err(|_| ());
    let year = match main.len() {
        8 => {
            // POSIX: MMDDhhmm uses the current year.
            current_year()
        }
        10 => {
            // YYMMDDhhmm: YY 00-68 -> 2000..2068, 69-99 -> 1969..1999.
            let yy = digits(&main[..2])?;
            (if yy <= 68 { 2000 + yy } else { 1900 + yy }) as i32
        }
        12 => (digits(&main[..2])? * 100 + digits(&main[2..4])?) as i32,
        _ => return Err(()),
    };
    let base = match main.len() {
        8 => 0,
        10 => 2, // skip YY
        12 => 4, // skip CC YY
        _ => unreachable!(),
    };
    let parts = &main[base..];
    let wall = WallClock {
        year,
        month: digits(&parts[0..2])?,
        day: digits(&parts[2..4])?,
        hour: digits(&parts[4..6])?,
        minute: digits(&parts[6..8])?,
        second,
    };
    validate_wall(wall)?;
    Ok(wall)
}

/// `YYYY-MM-DD` (strict 4-2-2 digits).
fn parse_ymd(raw: &str) -> Result<(i32, u32, u32), ()> {
    let parts: Vec<&str> = raw.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return Err(());
    }
    for part in &parts {
        if !part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(());
        }
    }
    Ok((
        parts[0].parse().map_err(|_| ())?,
        parts[1].parse().map_err(|_| ())?,
        parts[2].parse().map_err(|_| ())?,
    ))
}

/// `HH:MM` or `HH:MM:SS` (strict 2-2[:2] digits).
fn parse_hms(raw: &str) -> Result<(u32, u32, u32), ()> {
    let parts: Vec<&str> = raw.split(':').collect();
    let seconds = match parts.len() {
        2 => 0,
        3 => {
            if parts[2].len() != 2 || !parts[2].bytes().all(|b| b.is_ascii_digit()) {
                return Err(());
            }
            parts[2].parse().map_err(|_| ())?
        }
        _ => return Err(()),
    };
    for part in &parts[..2] {
        if part.len() != 2 || !part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(());
        }
    }
    Ok((
        parts[0].parse().map_err(|_| ())?,
        parts[1].parse().map_err(|_| ())?,
        seconds,
    ))
}

/// Calendar range checks; anything the OS cannot represent is rejected as an
/// invalid date (GNU rejects e.g. 2025-02-30 the same way).
fn validate_wall(wall: WallClock) -> Result<(), ()> {
    if wall.year < 1601
        || wall.month == 0
        || wall.month > 12
        || wall.day == 0
        || wall.day > days_in_month(wall.year, wall.month)
        || wall.hour > 23
        || wall.minute > 59
        || wall.second > 59
    {
        return Err(());
    }
    Ok(())
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn current_year() -> i32 {
    // Safe wrapper around kernel32 GetLocalTime (windows crate provides it).
    unsafe { windows::Win32::System::SystemInformation::GetLocalTime().wYear as i32 }
}

// --- epoch / wall clock -> FILETIME -----------------------------------------

fn epoch_to_filetime(epoch: i64) -> FILETIME {
    // FILETIME counts 100ns intervals since 1601-01-01; the offset between
    // that epoch and the Unix epoch is 11644473600 seconds.
    let intervals = epoch * 10_000_000 + UNIX_TO_FILETIME_INTERVALS;
    to_filetime(intervals)
}

// Converts a local wall-clock time through the OS timezone database
// (TzSpecificLocalTimeToSystemTime), so DST rules apply exactly like the
// system time functions. Declared directly against kernel32 because the
// `windows` crate features enabled in this workspace do not include
// Win32_System_Time, and the dependency set must stay frozen.
#[link(name = "kernel32")]
unsafe extern "system" {
    fn TzSpecificLocalTimeToSystemTime(
        timezone: *const core::ffi::c_void,
        local: *const SYSTEMTIME,
        utc: *mut SYSTEMTIME,
    ) -> i32;
    fn SystemTimeToFileTime(system_time: *const SYSTEMTIME, file_time: *mut FILETIME) -> i32;
}

fn wall_to_filetime(wall: WallClock) -> Result<FILETIME, String> {
    let local = SYSTEMTIME {
        wYear: wall.year as u16,
        wMonth: wall.month as u16,
        wDayOfWeek: 0,
        wDay: wall.day as u16,
        wHour: wall.hour as u16,
        wMinute: wall.minute as u16,
        wSecond: wall.second as u16,
        wMilliseconds: 0,
    };
    unsafe {
        let mut utc = SYSTEMTIME::default();
        if TzSpecificLocalTimeToSystemTime(std::ptr::null(), &local, &mut utc) == 0 {
            return Err(format!("invalid date format '{}'", format_wall(wall)));
        }
        let mut ft = FILETIME::default();
        if SystemTimeToFileTime(&utc, &mut ft) == 0 {
            return Err(format!("invalid date format '{}'", format_wall(wall)));
        }
        Ok(ft)
    }
}

fn format_wall(wall: WallClock) -> String {
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        wall.year, wall.month, wall.day, wall.hour, wall.minute, wall.second
    )
}

// --- reference file ----------------------------------------------------------

/// Reads the reference file's atime/mtime via GetFileTime (GNU message on
/// failure: `failed to get attributes of '<file>': <system message>`).
fn reference_times(reference: &Path) -> Result<(FILETIME, FILETIME), String> {
    let wide = to_wide(reference);
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            FILE_READ_ATTRIBUTES.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            HANDLE::default(),
        )
    }
    .map_err(|e| {
        format!(
            "failed to get attributes of '{}': {}",
            reference.display(),
            os_message(&e)
        )
    })?;

    let mut atime = FILETIME::default();
    let mut mtime = FILETIME::default();
    let result = unsafe { GetFileTime(handle, None, Some(&mut atime), Some(&mut mtime)) };
    unsafe { CloseHandle(handle) }.map_err(|e| {
        format!(
            "failed to get attributes of '{}': {}",
            reference.display(),
            os_message(&e)
        )
    })?;
    result.map_err(|e| {
        format!(
            "failed to get attributes of '{}': {}",
            reference.display(),
            os_message(&e)
        )
    })?;
    Ok((atime, mtime))
}

/// Maps the common not-found HRESULTs to the POSIX wording GNU prints.
fn os_message(error: &windows::core::Error) -> String {
    const ERROR_FILE_NOT_FOUND: HRESULT = HRESULT(0x8007_0002_u32 as i32);
    const ERROR_PATH_NOT_FOUND: HRESULT = HRESULT(0x8007_0003_u32 as i32);
    if error.code() == ERROR_FILE_NOT_FOUND || error.code() == ERROR_PATH_NOT_FOUND {
        "No such file or directory".to_string()
    } else {
        error.to_string()
    }
}

// --- touching ----------------------------------------------------------------

fn touch_file(
    path: &Path,
    no_create: bool,
    atime_only: bool,
    mtime_only: bool,
    source: Source,
) -> Result<(), String> {
    let exists = path.symlink_metadata().is_ok();
    if !exists && no_create {
        return Ok(());
    }

    // FILE_WRITE_ATTRIBUTES lets us update timestamps of read-only files, and
    // FILE_FLAG_BACKUP_SEMANTICS lets us open directories as well as files.
    let wide = to_wide(path);
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            FILE_WRITE_ATTRIBUTES.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            if exists { OPEN_EXISTING } else { CREATE_NEW },
            FILE_FLAG_BACKUP_SEMANTICS,
            HANDLE::default(),
        )
    }
    .map_err(|e| format!("cannot open file: {}", e))?;

    let (source_atime, source_mtime) = match source {
        Source::Fixed { atime, mtime } => (atime, mtime),
        Source::Now => {
            let now = systemtime_to_filetime(SystemTime::now());
            (now, now)
        }
    };
    let (atime, mtime) = match (atime_only, mtime_only) {
        (true, false) => (Some(source_atime), None),
        (false, true) => (None, Some(source_mtime)),
        // Default (and the redundant -a -m combination) updates both, like GNU.
        _ => (Some(source_atime), Some(source_mtime)),
    };

    let result = unsafe {
        SetFileTime(
            handle,
            None,
            atime.as_ref().map(|t| t as *const FILETIME),
            mtime.as_ref().map(|t| t as *const FILETIME),
        )
    };
    unsafe { CloseHandle(handle) }.map_err(|e| format!("cannot close file: {}", e))?;

    result.map_err(|e| format!("cannot set file time: {}", e))?;
    Ok(())
}

fn to_wide(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

const UNIX_TO_FILETIME_INTERVALS: i64 = 116_444_736_000_000_000;

fn to_filetime(intervals: i64) -> FILETIME {
    FILETIME {
        dwLowDateTime: intervals as u32,
        dwHighDateTime: (intervals >> 32) as u32,
    }
}

fn systemtime_to_filetime(t: SystemTime) -> FILETIME {
    let dur = t.duration_since(UNIX_EPOCH).unwrap_or_default();
    let intervals = dur.as_secs() as i64 * 10_000_000
        + dur.subsec_nanos() as i64 / 100
        + UNIX_TO_FILETIME_INTERVALS;
    to_filetime(intervals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_unix_time_to_filetime() {
        // 2026-10-07T00:00:00Z = 1791024000 epoch seconds.
        let t = UNIX_EPOCH + std::time::Duration::from_secs(1_791_024_000);
        let ft = systemtime_to_filetime(t);
        let intervals = ((ft.dwHighDateTime as i64) << 32) | ft.dwLowDateTime as i64;
        // FILETIME = (unix_secs + 11644473600) * 10^7
        let expected = (1_791_024_000i64 + 11_644_473_600) * 10_000_000;
        assert_eq!(intervals, expected);
    }

    #[test]
    fn converts_epoch_to_filetime() {
        let ft = systemtime_to_filetime(UNIX_EPOCH);
        let intervals = ((ft.dwHighDateTime as i64) << 32) | ft.dwLowDateTime as i64;
        assert_eq!(intervals, 116_444_736_000_000_000);
    }

    #[test]
    fn epoch_to_filetime_matches_systemtime_path() {
        let t = UNIX_EPOCH + std::time::Duration::from_secs(1_791_024_000);
        assert_eq!(
            epoch_to_filetime(1_791_024_000).dwLowDateTime,
            systemtime_to_filetime(t).dwLowDateTime
        );
        assert_eq!(
            epoch_to_filetime(1_791_024_000).dwHighDateTime,
            systemtime_to_filetime(t).dwHighDateTime
        );
    }

    #[test]
    fn parses_stamp_forms() {
        // 8 digits: current year; 10 digits: POSIX century rule.
        assert_eq!(
            parse_stamp("2503040506").unwrap().year,
            2025,
            "YY 25 belongs to the 2000s"
        );
        assert_eq!(
            parse_stamp("6901010000").unwrap().year,
            1969,
            "YY 69 belongs to the 1900s"
        );
        assert_eq!(
            parse_stamp("196901010000").unwrap().year,
            1969,
            "explicit CC YY is taken verbatim"
        );
        assert_eq!(
            parse_stamp("03040506").unwrap(),
            WallClock {
                year: current_year(),
                month: 3,
                day: 4,
                hour: 5,
                minute: 6,
                second: 0
            }
        );
        assert_eq!(
            parse_stamp("202503040506.07").unwrap().second,
            7,
            ".ss sets the seconds"
        );
    }

    #[test]
    fn rejects_invalid_stamps_and_dates() {
        for bad in ["1234", "202503040506.5.5", "20250304x506", "123456789", ""] {
            assert!(parse_stamp(bad).is_err(), "-t {bad:?} must be rejected");
        }
        for bad in [
            "not a date",
            "2025-1-2",
            "2025-01-02 03:04:05 junk",
            "2025-02-30",
            "@12.5",
            "2025-01-02T03",
            "2025-01-02T03:04:05Z",
        ] {
            assert!(parse_date(bad).is_err(), "-d {bad:?} must be rejected");
        }
    }

    #[test]
    fn parses_date_subset() {
        assert_eq!(
            parse_date("@-86400").unwrap(),
            DateSource::Epoch(-86_400),
            "@ accepts negative epochs"
        );
        assert_eq!(
            parse_date("2025-01-02T03:04").unwrap(),
            DateSource::Wall(WallClock {
                year: 2025,
                month: 1,
                day: 2,
                hour: 3,
                minute: 4,
                second: 0
            })
        );
    }
}
