mod cli;

use clap::Parser;
use std::env;

#[cfg(windows)]
fn get_current_user_groups() -> Vec<String> {
    use std::process::Command;

    let username = env::var("USERNAME").unwrap_or_else(|_| "".to_string());
    if username.is_empty() {
        return Vec::new();
    }

    let output = Command::new("net")
        .arg("user")
        .arg(&username)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let lines: Vec<&str> = stdout.split('\n').collect();

            let mut groups = Vec::new();

            for line in &lines {
                let line = line.trim_end_matches('\r');
                if line.contains("Local Group") {
                    if let Some(idx) = line.find('*') {
                        let groups_str = &line[idx..];
                        for group in groups_str.split_whitespace() {
                            if !group.is_empty() {
                                let cleaned_group = group.trim_start_matches('*');
                                groups.push(cleaned_group.to_string());
                            }
                        }
                    }
                    break;
                }
            }

            if groups.is_empty() {
                for line in &lines {
                    let line = line.trim_end_matches('\r');
                    if line.contains("Group Memberships") || line.contains("本地组成员") {
                        let parts: Vec<&str> = line.split(':').collect();
                        if parts.len() > 1 {
                            let groups_str = parts[1].trim();
                            for group in groups_str.split_whitespace() {
                                if !group.is_empty() {
                                    let cleaned_group = group.trim_start_matches('*');
                                    groups.push(cleaned_group.to_string());
                                }
                            }
                        }
                        break;
                    }
                }
            }

            groups
        }
        Err(_) => {
            let mut groups = Vec::new();
            if let Ok(domain) = env::var("USERDOMAIN") {
                groups.push(domain);
            }
            if let Ok(username) = env::var("USERNAME") {
                groups.push(username);
            }
            if let Ok(computername) = env::var("COMPUTERNAME") {
                groups.push(computername);
            }
            groups.push("Everyone".to_string());
            groups.push("Authenticated Users".to_string());
            groups
        }
    }
}

#[cfg(not(windows))]
fn get_current_user_groups() -> Vec<String> {
    Vec::new()
}

fn main() {
    let args = cli::Args::parse();

    let username = args.user.clone().unwrap_or_else(|| {
        env::var("USERNAME").unwrap_or_else(|_| env::var("USER").unwrap_or_else(|_| "".to_string()))
    });

    let groups = get_current_user_groups();
    if groups.is_empty() {
        println!("{} :", username);
    } else {
        println!("{} : {}", username, groups.join(" "));
    }
}