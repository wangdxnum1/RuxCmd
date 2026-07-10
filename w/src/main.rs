mod cli;

use clap::Parser;
use cli::Cli;
use std::process::Command;

fn get_uptime() -> String {
    let output = Command::new("systeminfo")
        .output()
        .expect("Failed to execute systeminfo");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
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
    let output = Command::new("query")
        .arg("user")
        .output()
        .expect("Failed to execute query user");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut users = Vec::new();
    
    for line in stdout.lines().skip(1) {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if !parts.is_empty() {
                users.push(parts[0].to_string());
            }
        }
    }
    
    if users.is_empty() {
        let output = Command::new("whoami")
            .output()
            .expect("Failed to execute whoami");
        let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
        users.push(username);
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
