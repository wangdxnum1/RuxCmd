mod cli;

use clap::Parser;
use cli::Cli;
use std::process::Command;

fn get_uptime() -> String {
    uptime_from("systeminfo")
}

fn uptime_from(program: &str) -> String {
    let output = match Command::new(program).output() {
        Ok(output) => output,
        Err(_) => {
            eprintln!("w: {program} unavailable");
            return "unavailable".to_string();
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        eprintln!("w: {program} unavailable");
        return "unavailable".to_string();
    }

    for line in stdout.lines() {
        if line.contains("系统启动时间") || line.contains("System Boot Time") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 1 {
                return parts[1..].join(":").trim().to_string();
            }
        }
    }
    "Unknown".to_string()
}

fn get_users() -> Vec<String> {
    collect_users("query", "whoami")
}

fn collect_users(query_program: &str, whoami_program: &str) -> Vec<String> {
    let mut users = Vec::new();

    if let Ok(output) = Command::new(query_program).arg("user").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if !parts.is_empty() {
                    users.push(parts[0].to_string());
                }
            }
        }
    } else {
        eprintln!("w: {query_program} user unavailable");
    }

    if users.is_empty() {
        if let Ok(output) = Command::new(whoami_program).output() {
            let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !username.is_empty() {
                users.push(username);
            }
        }
    }

    if users.is_empty() {
        eprintln!("w: {whoami_program} unavailable");
        users.push("unavailable".to_string());
    }

    users
}

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("w 0.1.0");
        return;
    }

    if cli.help {
        println!("w 0.1.0 - 显示用户活动");
        println!("");
        println!("用法: w [选项]");
        println!("");
        println!("选项:");
        println!("  -h, --header          显示头部信息");
        println!("  -u, --ignore-username 忽略用户名");
        println!("  -v, --version         显示版本信息");
        println!("      --help            显示帮助信息");
        return;
    }

    let uptime = get_uptime();
    let users = get_users();

    if cli.header {
        println!("系统启动时间: {}", uptime);
    }

    for user in users {
        if !cli.ignore_username {
            println!("{}", user);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{collect_users, uptime_from};

    // A name that cannot be resolved by any PATH search, forcing a launch
    // failure without touching the real system tools.
    const MISSING: &str = "ruxcmd-w-missing-command-under-test";

    #[test]
    fn uptime_degrades_when_source_cannot_launch() {
        // Regression: used to panic via .expect() when systeminfo could not
        // be launched.
        assert_eq!(uptime_from(MISSING), "unavailable");
    }

    #[test]
    fn users_degrade_when_sources_cannot_launch() {
        // Regression: used to panic via .expect() when query user / whoami
        // could not be launched (e.g. Windows Home without query.exe).
        assert_eq!(collect_users(MISSING, MISSING), vec!["unavailable"]);
    }
}
