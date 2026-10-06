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
        print_pretty(uptime);
    } else {
        println!("{}", uptime);
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
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("无法获取系统时间")
        .as_secs()
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

fn print_pretty(seconds: u64) {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{}天", days));
    }
    if hours > 0 {
        parts.push(format!("{}小时", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}分钟", minutes));
    }
    if secs > 0 || parts.is_empty() {
        parts.push(format!("{}秒", secs));
    }

    println!("{}", parts.join(" "));
}
