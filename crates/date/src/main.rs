mod cli;

use chrono::{DateTime, Local, Utc};
use clap::Parser;
use std::time::SystemTime;

fn main() {
    let args = cli::Args::parse();

    let now = SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();

    if args.seconds {
        println!("{}", duration.as_secs());
        return;
    }

    if args.nanoseconds {
        println!("{}", duration.as_nanos());
        return;
    }

    let format = args
        .format
        .unwrap_or_else(|| "%a %b %d %H:%M:%S %Z %Y".to_string());
    let output = format_date(&now, &format, args.utc);
    println!("{}", output);
}

fn format_date(now: &SystemTime, format: &str, utc: bool) -> String {
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs() as i64;

    let dt: DateTime<Utc> = DateTime::from_timestamp(secs, 0).unwrap_or_else(|| Utc::now());

    if utc {
        dt.format(format).to_string()
    } else {
        dt.with_timezone(&Local).format(format).to_string()
    }
}
