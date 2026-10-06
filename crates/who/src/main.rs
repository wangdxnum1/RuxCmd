mod cli;

use clap::Parser;
use std::env;

#[cfg(windows)]
fn get_username() -> String {
    env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(windows)]
fn get_domain() -> String {
    env::var("USERDOMAIN").unwrap_or_else(|_| "".to_string())
}

#[cfg(windows)]
fn get_computername() -> String {
    env::var("COMPUTERNAME").unwrap_or_else(|_| "localhost".to_string())
}

#[cfg(windows)]
fn get_logon_server() -> String {
    env::var("LOGONSERVER").unwrap_or_else(|_| "".to_string())
}

#[cfg(windows)]
fn get_sessionname() -> String {
    if let Ok(sessionname) = env::var("SESSIONNAME") {
        sessionname
    } else {
        "console".to_string()
    }
}

#[cfg(windows)]
fn get_boot_time() -> Option<String> {
    use std::process::Command;
    let output = Command::new("systeminfo").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.to_lowercase().contains("system boot time") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 1 {
                return Some(parts[1..].join(":").trim().to_string());
            }
        }
    }
    None
}

fn main() {
    let args = cli::Args::parse();

    if args.boot_time || args.all {
        if let Some(boot_time) = get_boot_time() {
            println!("system boot time: {}", boot_time);
        } else {
            println!("system boot time: unavailable");
        }
    }

    if args.dead || args.all {
        println!("dead processes: not supported on Windows");
    }

    if !args.boot_time && !args.dead || args.all {
        let username = get_username();
        let domain = get_domain();
        let computername = get_computername();
        let sessionname = get_sessionname();
        let logon_server = get_logon_server();

        if args.all {
            let full_username = if !domain.is_empty() {
                format!("{}\\{}", domain, username)
            } else {
                username.clone()
            };
            println!("{} {} {}", full_username, sessionname, computername);
            if !logon_server.is_empty() {
                println!("logon server: {}", logon_server);
            }
        } else {
            println!("{}", username);
        }
    }
}
