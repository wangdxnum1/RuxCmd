mod cli;

use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use clap::Parser;
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() -> ExitCode {
    let args = cli::Args::parse();

    // Resolve the positional operand: a leading '+' selects the output format,
    // anything else is a date string equivalent to --date.
    let mut format: Option<String> = None;
    let mut date_string: Option<String> = args.date_string;
    if let Some(operand) = &args.operand {
        if let Some(stripped) = operand.strip_prefix('+') {
            format = Some(stripped.to_string());
        } else if date_string.is_some() {
            eprintln!("date: extra operand '{}'", operand);
            return ExitCode::from(1);
        } else {
            date_string = Some(operand.clone());
        }
    }

    // -s/--set: parse the value, set the system time, and exit silently on
    // success. An invalid value never reaches the system-time API.
    if let Some(value) = &args.set_time {
        return match parse_date_string(value, args.utc) {
            Some(instant) => match set_system_time(instant) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("date: cannot set date: {}", err);
                    ExitCode::from(1)
                }
            },
            None => {
                eprintln!("date: invalid date '{}'", value);
                ExitCode::from(1)
            }
        };
    }

    let instant = match &date_string {
        Some(value) => match parse_date_string(value, args.utc) {
            Some(instant) => instant,
            None => {
                eprintln!("date: invalid date '{}'", value);
                return ExitCode::from(1);
            }
        },
        None => SystemTime::now(),
    };

    if args.seconds {
        match instant.duration_since(UNIX_EPOCH) {
            Ok(d) => println!("{}", d.as_secs()),
            Err(e) => println!("-{}", e.duration().as_secs()),
        }
        return ExitCode::SUCCESS;
    }

    if args.nanoseconds {
        match instant.duration_since(UNIX_EPOCH) {
            Ok(d) => println!("{}", d.as_nanos()),
            Err(e) => println!("-{}", e.duration().as_nanos()),
        }
        return ExitCode::SUCCESS;
    }

    let format = format.unwrap_or_else(|| "%a %b %d %H:%M:%S %Z %Y".to_string());
    println!("{}", format_date(&instant, &format, args.utc));
    ExitCode::SUCCESS
}

/// Minimal --date parsing: empty string means now, `@<epoch-seconds>` is an
/// instant, and ISO 8601 forms (`YYYY-MM-DD`, `YYYY-MM-DDTHH:MM[:SS]`) are
/// interpreted in UTC when `utc` is set and in the local timezone otherwise.
/// Returns None for anything else (reported as an invalid date).
fn parse_date_string(value: &str, utc: bool) -> Option<SystemTime> {
    if value.is_empty() {
        return Some(SystemTime::now());
    }

    if let Some(rest) = value.strip_prefix('@') {
        let secs: i64 = rest.parse().ok()?;
        return epoch_to_system_time(secs);
    }

    let (date_part, time_part) = match value.split_once('T') {
        Some((date, time)) => (date, Some(time)),
        None => (value, None),
    };
    let date = NaiveDate::parse_from_str(date_part, "%Y-%m-%d").ok()?;
    let naive = match time_part {
        Some(time) => {
            let time = NaiveTime::parse_from_str(time, "%H:%M:%S")
                .or_else(|_| NaiveTime::parse_from_str(time, "%H:%M"))
                .ok()?;
            date.and_time(time)
        }
        None => date.and_hms_opt(0, 0, 0)?,
    };

    Some(if utc {
        SystemTime::from(Utc.from_utc_datetime(&naive))
    } else {
        SystemTime::from(Local.from_local_datetime(&naive).earliest()?)
    })
}

fn epoch_to_system_time(secs: i64) -> Option<SystemTime> {
    if secs >= 0 {
        UNIX_EPOCH.checked_add(Duration::from_secs(secs as u64))
    } else {
        UNIX_EPOCH.checked_sub(Duration::from_secs(secs.unsigned_abs()))
    }
}

fn format_date(instant: &SystemTime, format: &str, utc: bool) -> String {
    let secs = match instant.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64),
    };

    let dt: DateTime<Utc> = DateTime::from_timestamp(secs, 0).unwrap_or_else(Utc::now);

    if utc {
        dt.format(format).to_string()
    } else {
        dt.with_timezone(&Local).format(format).to_string()
    }
}

#[cfg(windows)]
fn set_system_time(instant: SystemTime) -> Result<(), String> {
    use windows::Win32::Foundation::SYSTEMTIME;
    use windows::Win32::System::SystemInformation::SetLocalTime;

    let local: DateTime<Local> = DateTime::<Utc>::from(instant).with_timezone(&Local);
    let system_time = SYSTEMTIME {
        wYear: local.year() as u16,
        wMonth: local.month() as u16,
        wDay: local.day() as u16,
        wDayOfWeek: local.weekday().num_days_from_sunday() as u16,
        wHour: local.hour() as u16,
        wMinute: local.minute() as u16,
        wSecond: local.second() as u16,
        wMilliseconds: local.timestamp_subsec_millis() as u16,
    };
    unsafe { SetLocalTime(&system_time) }.map_err(|e| e.to_string())
}

#[cfg(not(windows))]
fn set_system_time(_instant: SystemTime) -> Result<(), String> {
    Err("setting the system time is only supported on windows".to_string())
}
