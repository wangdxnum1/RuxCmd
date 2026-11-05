mod cli;

use clap::Parser;
use cli::{Cli, VERSION};

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("{}", VERSION);
        return;
    }

    let uptime = get_uptime();

    if cli.pretty {
        println!("{}", format_pretty(uptime));
    } else {
        let now = chrono::Local::now().format("%H:%M:%S").to_string();
        println!("{}", format_default(uptime, &now));
    }
}

#[cfg(target_os = "linux")]
fn get_uptime() -> u64 {
    use std::fs;
    let content = fs::read_to_string("/proc/uptime").expect("无法读取 /proc/uptime");
    let parts: Vec<&str> = content.split_whitespace().collect();
    parts[0].parse::<f64>().unwrap() as u64
}

#[cfg(target_os = "windows")]
fn get_uptime() -> u64 {
    use windows::Win32::System::SystemInformation::GetTickCount64;

    // SAFETY: GetTickCount64 takes no parameters and cannot fail.
    let tick_ms = unsafe { GetTickCount64() };
    tick_ms / 1000
}

#[cfg(target_os = "macos")]
fn get_uptime() -> u64 {
    use std::process::Command;
    let output = Command::new("sysctl")
        .arg("-n")
        .arg("kern.boottime")
        .output()
        .expect("无法执行 sysctl");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.split_whitespace().collect();
    let time_str = parts[1].trim_end_matches(',');
    time_str.parse::<f64>().unwrap() as u64
}

/// procps-style default line: ` HH:MM:SS up N days, H:MM, 0 users`.
/// The user count has no lightweight Windows API and stays 0 (declared).
fn format_default(seconds: u64, now_hms: &str) -> String {
    let days = seconds / 86400;
    let hm = format!("{}:{:02}", (seconds % 86400) / 3600, (seconds % 3600) / 60);
    let days_part = if days > 0 {
        format!("{} day{}, ", days, if days == 1 { "" } else { "s" })
    } else {
        String::new()
    };
    format!("{} up {}{}, 0 users", now_hms, days_part, hm)
}

/// GNU/procps style pretty line: `up 1 day, 2 hours, 3 minutes`.
/// Seconds are omitted, and an uptime below one minute prints just `up`.
fn format_pretty(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{} day{}", days, if days == 1 { "" } else { "s" }));
    }
    if hours > 0 {
        parts.push(format!(
            "{} hour{}",
            hours,
            if hours == 1 { "" } else { "s" }
        ));
    }
    if minutes > 0 {
        parts.push(format!(
            "{} minute{}",
            minutes,
            if minutes == 1 { "" } else { "s" }
        ));
    }

    if parts.is_empty() {
        "up".to_string()
    } else {
        format!("up {}", parts.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::format_pretty;

    #[test]
    fn pretty_format_uses_gnu_english_wording() {
        assert_eq!(format_pretty(90_061), "up 1 day, 1 hour, 1 minute");
        assert_eq!(format_pretty(59), "up");
        assert_eq!(format_pretty(3600), "up 1 hour");
        assert_eq!(format_pretty(0), "up");
        assert_eq!(
            format_pretty(2 * 86400 + 3 * 3600 + 5 * 60),
            "up 2 days, 3 hours, 5 minutes"
        );
        // Seconds never appear on their own.
        assert_eq!(format_pretty(61), "up 1 minute");
    }
}
