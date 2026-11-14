//! Local-time conversion and strftime-subset formatting shared by commands
//! that print timestamps (`du --time-style`, `diff -u` headers).
//!
//! Windows has no strftime and std has no local-time conversion, so this
//! module renders a moment through the OS timezone database (DST rules
//! apply exactly like the system time functions) and formats it with the
//! GNU strftime subset coreutils consumers rely on.

use std::time::{Duration, SystemTime, UNIX_EPOCH};
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

/// A relative date shift parsed from GNU's `N unit(s) [ago]` grammar:
/// calendar months move the wall clock (clamping the day), then the
/// duration part (days + seconds) shifts the instant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelativeShift {
    pub months: i64,
    pub days: i64,
    pub seconds: i64,
}

/// Recognizes GNU's relative-date grammar: `now`, `yesterday`, `tomorrow`,
/// `N unit(s) [ago]` with units second/minute/hour/day/week/fortnight/
/// month/year (case-insensitive, optional plural), and `a`/`an` as the
/// count.  Returns `None` when the text is not a relative expression.
pub fn parse_relative(raw: &str) -> Option<RelativeShift> {
    let lower = raw.trim().to_ascii_lowercase();
    if lower == "now" {
        return Some(RelativeShift {
            months: 0,
            days: 0,
            seconds: 0,
        });
    }
    if lower == "yesterday" {
        return Some(RelativeShift {
            months: 0,
            days: -1,
            seconds: 0,
        });
    }
    if lower == "tomorrow" {
        return Some(RelativeShift {
            months: 0,
            days: 1,
            seconds: 0,
        });
    }
    let mut parts = lower.split_whitespace();
    let count_part = parts.next()?;
    let count: i64 = match count_part {
        "a" | "an" => 1,
        digits if digits.bytes().all(|b| b.is_ascii_digit()) => digits.parse().ok()?,
        _ => return None,
    };
    let unit = parts.next()?;
    let unit = unit.strip_suffix('s').unwrap_or(unit);
    let unit_seconds = match unit {
        "second" | "sec" => 1i64,
        "minute" | "min" => 60,
        "hour" => 3600,
        "day" => 86400,
        "week" => 7 * 86400,
        "fortnight" => 14 * 86400,
        "month" | "mon" => 0,
        "year" => 0,
        _ => return None,
    };
    let mut shift = RelativeShift {
        months: 0,
        days: 0,
        seconds: 0,
    };
    match unit {
        "month" | "mon" => shift.months = count,
        "year" => shift.months = count * 12,
        _ => shift.seconds += count * unit_seconds,
    }
    match parts.next() {
        None => {}
        Some("ago") => {
            shift.months = -shift.months;
            shift.seconds = -shift.seconds;
            shift.days = -shift.days;
        }
        _ => return None,
    }
    if parts.next().is_some() {
        return None;
    }
    Some(shift)
}

/// Applies a relative shift to the current instant: calendar months move on
/// the local wall clock (clamping the day, like GNU), then the duration part
/// shifts the resulting instant.  Returns `None` when pre-1970 or the
/// FILETIME range cannot hold the result.
pub fn apply_relative(shift: RelativeShift) -> Option<SystemTime> {
    let now = SystemTime::now();
    let total_seconds = shift.days * 86400 + shift.seconds;
    if shift.months == 0 {
        return shift_instant(now, total_seconds);
    }
    let local = local_time(now)?;
    let total_months = local.year as i64 * 12 + local.month as i64 - 1 + shift.months;
    let year = total_months.div_euclid(12) as u16;
    let month = (total_months.rem_euclid(12) + 1) as u16;
    let day = local.day.min(days_in_month_of(year, month)) as u32;
    let wall = WallDateTime {
        year,
        month,
        day,
        hour: local.hour as u32,
        minute: local.minute as u32,
        second: local.second as u32,
    };
    let base = wall_datetime_to_epoch(&wall)?;
    epoch_to_system_time(base + total_seconds)
}

/// Shifts an instant by a signed number of seconds.
pub fn shift_instant(instant: SystemTime, seconds: i64) -> Option<SystemTime> {
    if seconds >= 0 {
        instant.checked_add(Duration::from_secs(seconds as u64))
    } else {
        instant.checked_sub(Duration::from_secs(seconds.unsigned_abs()))
    }
}

/// Epoch seconds from a local wall clock read as UTC (no zone conversion).
pub fn wall_datetime_to_epoch(wall: &WallDateTime) -> Option<i64> {
    let days = days_from_civil(wall.year as i64, wall.month as i64, wall.day as i64);
    Some(days * 86400 + wall.hour as i64 * 3600 + wall.minute as i64 * 60 + wall.second as i64)
}

fn epoch_to_system_time(seconds: i64) -> Option<SystemTime> {
    if seconds >= 0 {
        SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(seconds as u64))
    } else {
        SystemTime::UNIX_EPOCH.checked_sub(Duration::from_secs(seconds.unsigned_abs()))
    }
}

/// Local wall-clock fields for the calendar math above.
pub struct WallDateTime {
    pub year: u16,
    pub month: u16,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

fn days_in_month_of(year: u16, month: u16) -> u16 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        _ => 0,
    }
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
