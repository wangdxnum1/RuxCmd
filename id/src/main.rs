mod cli;

use clap::Parser;
use std::env;

#[cfg(windows)]
fn get_user_name() -> String {
    env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(windows)]
fn get_domain_name() -> String {
    env::var("USERDOMAIN").unwrap_or_else(|_| "".to_string())
}

#[cfg(windows)]
fn get_group_name() -> String {
    if let Ok(domain) = env::var("USERDOMAIN") {
        if let Ok(username) = env::var("USERNAME") {
            format!("{}\\{}", domain, username)
        } else {
            domain
        }
    } else {
        env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string())
    }
}

#[cfg(windows)]
fn get_groups() -> Vec<String> {
    let mut groups = Vec::new();
    if let Ok(domain) = env::var("USERDOMAIN") {
        groups.push(domain);
    }
    if let Ok(username) = env::var("USERNAME") {
        groups.push(username.clone());
        if let Ok(domain) = env::var("USERDOMAIN") {
            groups.push(format!("{}\\{}", domain, username));
        }
    }
    if let Ok(computername) = env::var("COMPUTERNAME") {
        groups.push(computername);
    }
    groups.push("Everyone".to_string());
    groups.push("Authenticated Users".to_string());
    groups
}

#[cfg(not(windows))]
fn get_user_name() -> String {
    env::var("USER").unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(not(windows))]
fn get_group_name() -> String {
    "unknown".to_string()
}

#[cfg(not(windows))]
fn get_groups() -> Vec<String> {
    Vec::new()
}

fn main() {
    let args = cli::Args::parse();

    if args.user {
        if args.name {
            println!("{}", get_user_name());
        } else {
            let username = get_user_name();
            let domain = get_domain_name();
            if !domain.is_empty() {
                println!("{}\\{}", domain, username);
            } else {
                println!("{}", username);
            }
        }
    } else if args.group {
        if args.name {
            println!("{}", get_group_name());
        } else {
            println!("{}", get_group_name());
        }
    } else if args.groups {
        let groups = get_groups();
        if args.name {
            println!("{}", groups.join(" "));
        } else {
            println!("{}", groups.join(" "));
        }
    } else {
        let username = get_user_name();
        let domain = get_domain_name();
        let groups = get_groups();

        if !domain.is_empty() {
            println!("uid={}({}) gid={}({}) groups={}", 
                username, username, 
                get_group_name(), get_group_name(),
                groups.join(","));
        } else {
            println!("uid={}({}) gid={}({}) groups={}", 
                username, username, 
                get_group_name(), get_group_name(),
                groups.join(","));
        }
    }
}
