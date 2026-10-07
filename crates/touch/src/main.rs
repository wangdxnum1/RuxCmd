mod cli;

use clap::Parser;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, SetFileTime, CREATE_NEW, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_WRITE_ATTRIBUTES,
};

fn main() {
    let args = cli::Args::parse();

    if args.files.is_empty() {
        eprintln!("touch: missing file operand");
        std::process::exit(1);
    }

    let mut exit_code = 0i32;

    for path in &args.files {
        if let Err(e) = touch_file(path, args.no_create, args.atime, args.mtime) {
            eprintln!("touch: {}: {}", path.display(), e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

fn touch_file(
    path: &Path,
    no_create: bool,
    atime_only: bool,
    mtime_only: bool,
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

    let now = systemtime_to_filetime(SystemTime::now());
    let (atime, mtime) = match (atime_only, mtime_only) {
        (true, false) => (Some(now), None),
        (false, true) => (None, Some(now)),
        // Default (and the redundant -a -m combination) updates both, like GNU.
        _ => (Some(now), Some(now)),
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

fn systemtime_to_filetime(t: SystemTime) -> FILETIME {
    // FILETIME counts 100ns intervals since 1601-01-01; the offset between
    // that epoch and the Unix epoch is 11644473600 seconds.
    const UNIX_TO_FILETIME_INTERVALS: i64 = 116_444_736_000_000_000;
    let dur = t.duration_since(UNIX_EPOCH).unwrap_or_default();
    let intervals = dur.as_secs() as i64 * 10_000_000
        + dur.subsec_nanos() as i64 / 100
        + UNIX_TO_FILETIME_INTERVALS;
    FILETIME {
        dwLowDateTime: intervals as u32,
        dwHighDateTime: (intervals >> 32) as u32,
    }
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
}
