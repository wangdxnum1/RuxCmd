//! Local-time conversion and strftime-subset formatting shared by commands
//! that print timestamps (`du --time-style`, `diff -u` headers).
//!
//! Windows has no strftime and std has no local-time conversion, so this
//! module renders a moment through the OS timezone database (DST rules
//! apply exactly like the system time functions) and formats it with the
//! GNU strftime subset coreutils consumers rely on.

use std::time::{SystemTime, UNIX_EPOCH};
use windows_sys::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows_sys::Win32::System::Time::{
    FileTimeToSystemTime, GetTimeZoneInformation, SystemTimeToTzSpecificLocalTime,
    TIME_ZONE_INFORMATION,
};

/// TIME_ZONE_ID_* return values of GetTimeZoneInformation (DWORD).
const TIME_ZONE_ID_STANDARD: u32 = 1;
const TIME_ZONE_ID_DAYLIGHT: u32 = 2;

/// 100ns intervals between the Windows epoch (1601-01-01) and the Unix epoch.
const UNIX_TO_FILETIME_INTERVALS: i128 = 116_444_736_000_000_000;

/// One moment in the machine's local wall clock, with its offset from UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTime {
    pub year: u16,
    pub month: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub nanos: u32,
    /// Seconds since the Unix epoch (for `%s`).
    pub epoch_seconds: i64,
    /// Wall-clock offset from UTC in minutes (east positive, 480 for UTC+8).
    pub offset_minutes: i32,
}

/// Converts a `SystemTime` to the machine's local wall clock. Returns `None`
/// for pre-epoch instants the FILETIME range cannot hold.
pub fn local_time(when: SystemTime) -> Option<LocalTime> {
    let since_epoch = when.duration_since(UNIX_EPOCH).ok()?;
    let nanos = since_epoch.subsec_nanos();
    let epoch_seconds = since_epoch.as_secs() as i64;
    let intervals = (since_epoch.as_nanos() / 100) as i128 + UNIX_TO_FILETIME_INTERVALS;
    if intervals > i64::MAX as i128 {
        return None;
    }
    let filetime = FILETIME {
        dwLowDateTime: intervals as u32,
        dwHighDateTime: (intervals >> 32) as u32,
    };
    let mut utc = SYSTEMTIME::default();
    // SAFETY: both pointers are valid out/in-parameters for the duration of
    // the calls.
    if unsafe { FileTimeToSystemTime(&filetime, &mut utc) } == 0 {
        return None;
    }
    let mut local = SYSTEMTIME::default();
    // A null timezone means "use the active timezone database".
    if unsafe { SystemTimeToTzSpecificLocalTime(std::ptr::null(), &utc, &mut local) } == 0 {
        return None;
    }
    let mut tzi: TIME_ZONE_INFORMATION = unsafe { std::mem::zeroed() };
    let state = unsafe { GetTimeZoneInformation(&mut tzi) };
    let extra = match state {
        TIME_ZONE_ID_DAYLIGHT => tzi.DaylightBias,
        TIME_ZONE_ID_STANDARD => tzi.StandardBias,
        _ => 0,
    };
    // UTC = local + bias, so the printed offset is the negated bias.
    let offset_minutes = -(tzi.Bias + extra);
    Some(LocalTime {
        year: local.wYear,
        month: local.wMonth,
        day: local.wDay,
        hour: local.wHour,
        minute: local.wMinute,
        second: local.wSecond,
        nanos,
        epoch_seconds,
        offset_minutes,
    })
}

/// Minimal strftime subset: `%Y %m %d %H %M %S %N` (9-digit nanoseconds),
/// `%F` (`%Y-%m-%d`), `%T` (`%H:%M:%S`), `%s` (epoch seconds), `%z` (`+0800`),
/// `%:z` (`+08:00`), `%n` (newline), `%t` (tab), `%%`. Unknown directives
/// stay verbatim, like glibc.
pub fn strftime(format: &str, t: &LocalTime) -> String {
    let mut out = String::with_capacity(format.len() + 16);
    let mut chars = format.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('Y') => out.push_str(&format!("{:04}", t.year)),
            Some('m') => out.push_str(&format!("{:02}", t.month)),
            Some('d') => out.push_str(&format!("{:02}", t.day)),
            Some('H') => out.push_str(&format!("{:02}", t.hour)),
            Some('M') => out.push_str(&format!("{:02}", t.minute)),
            Some('S') => out.push_str(&format!("{:02}", t.second)),
            Some('N') => out.push_str(&format!("{:09}", t.nanos)),
            Some('F') => out.push_str(&format!("{:04}-{:02}-{:02}", t.year, t.month, t.day)),
            Some('T') => out.push_str(&format!("{:02}:{:02}:{:02}", t.hour, t.minute, t.second)),
            Some('s') => out.push_str(&t.epoch_seconds.to_string()),
            Some('z') => out.push_str(&offset_text(t, false)),
            Some(':') => {
                if chars.peek() == Some(&'z') {
                    chars.next();
                    out.push_str(&offset_text(t, true));
                } else {
                    out.push_str("%:");
                }
            }
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('%') => out.push('%'),
            Some(other) => {
                out.push('%');
                out.push(other);
            }
            None => out.push('%'),
        }
    }
    out
}

/// Days from 1970-01-01 to the given civil date (Howard Hinnant's
/// algorithm).  Works for any year/month/day combination, including dates
/// before the epoch.
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Seconds from the Unix epoch for a civil wall clock read as UTC (no zone
/// conversion).  Useful when a caller pins its own zone offset.
pub fn civil_to_epoch_seconds(
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
) -> i64 {
    days_from_civil(year, month, day) * 86400 + hour * 3600 + minute * 60 + second
}

/// Renders the offset as `+0800` (`colon = false`) or `+08:00`.
fn offset_text(t: &LocalTime, colon: bool) -> String {
    let (sign, minutes) = if t.offset_minutes >= 0 {
        ('+', t.offset_minutes)
    } else {
        ('-', -t.offset_minutes)
    };
    if colon {
        format!("{}{:02}:{:02}", sign, minutes / 60, minutes % 60)
    } else {
        format!("{}{:02}{:02}", sign, minutes / 60, minutes % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_roundtrips_through_local_time() {
        // A fixed moment: 2026-11-07 05:59:09.1737032 UTC.  The local wall
        // clock and offset come from the machine's zone (CI runners run UTC),
        // so only zone-independent facts and the round-trip shape assert.
        let when = UNIX_EPOCH + std::time::Duration::from_nanos(1_794_031_149_173_703_200);
        let local = local_time(when).expect("time converts");
        assert_eq!(local.epoch_seconds, 1_794_031_149);
        assert_eq!(local.nanos, 173_703_200);
        // Real zones are multiples of 15 minutes (UTC runners: 0).
        assert_eq!(local.offset_minutes % 15, 0);
        assert_eq!(
            strftime("%s", &local),
            "1794031149",
            "%s must be zone-independent"
        );
        // The stamp must round-trip the wall clock this zone produced.
        let stamp = strftime("%Y-%m-%d %H:%M:%S.%N %z", &local);
        assert_eq!(stamp.len(), 35, "stamp {stamp:?}");
        let (wall, offset) = stamp.rsplit_once(' ').expect("stamp layout");
        let (head, frac) = wall
            .split_once('.')
            .unwrap_or_else(|| panic!("stamp {stamp:?}"));
        assert_eq!(
            head,
            format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                local.year, local.month, local.day, local.hour, local.minute, local.second
            )
        );
        assert_eq!(frac, "173703200");
        assert!(
            offset.starts_with('+') || offset.starts_with('-'),
            "{stamp:?}"
        );
        assert_eq!(offset.len(), 5, "offset {stamp:?}");
    }

    #[test]
    fn strftime_directives_and_unknowns() {
        let t = LocalTime {
            year: 2026,
            month: 11,
            day: 7,
            hour: 14,
            minute: 52,
            second: 29,
            nanos: 123_000_000,
            epoch_seconds: 1_794_031_149,
            offset_minutes: 480,
        };
        assert_eq!(strftime("%F %T", &t), "2026-11-07 14:52:29");
        assert_eq!(strftime("%Y%m%d%H%M%S", &t), "20261107145229");
        assert_eq!(strftime("%:z", &t), "+08:00");
        assert_eq!(strftime("%s", &t), "1794031149");
        assert_eq!(strftime("a%Qb%n%t%%", &t), "a%Qb\n\t%");
        assert_eq!(strftime("trailing%", &t), "trailing%");
    }

    #[test]
    fn civil_math_matches_known_dates() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(1980, 1, 1), 3652);
        assert_eq!(days_from_civil(2026, 11, 7), 20764);
        assert_eq!(civil_to_epoch_seconds(1970, 1, 1, 0, 0, 0), 0);
        assert_eq!(civil_to_epoch_seconds(1970, 1, 1, 0, 0, 1), 1);
        assert_eq!(civil_to_epoch_seconds(2026, 11, 7, 0, 0, 0), 20764 * 86400);
    }
}
