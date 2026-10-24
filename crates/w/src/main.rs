mod cli;

use clap::Parser;
use cli::Cli;
use std::process::Command;

fn get_uptime() -> String {
    let uptime_ms = unsafe { windows::Win32::System::SystemInformation::GetTickCount64() };
    let boot = chrono::Local::now() - chrono::Duration::milliseconds(uptime_ms as i64);
    boot.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn get_users() -> Vec<String> {
    wts_user_lines().unwrap_or_else(|| collect_users("query", "whoami"))
}

/// One "USER tty" line per logged-on session via the Terminal Services API.
fn wts_user_lines() -> Option<Vec<String>> {
    use windows::core::PWSTR;
    use windows::Win32::System::RemoteDesktop::{
        WTSEnumerateSessionsW, WTSFreeMemory, WTSQuerySessionInformationW, WTSUserName,
        WTS_SESSION_INFOW,
    };

    let mut raw: *mut WTS_SESSION_INFOW = std::ptr::null_mut();
    let mut count: u32 = 0;
    // SAFETY: out-parameters point to valid storage; the buffer is released
    // with WTSFreeMemory below.
    if (unsafe { WTSEnumerateSessionsW(None, 0, 1, &mut raw, &mut count) }).is_err()
        || raw.is_null()
    {
        return None;
    }
    let entries = unsafe { std::slice::from_raw_parts(raw, count as usize) };
    let mut out = Vec::new();
    for e in entries {
        let mut buffer: PWSTR = PWSTR::null();
        let mut bytes: u32 = 0;
        // SAFETY: session ids come from the enumeration; the buffer is
        // released with WTSFreeMemory.
        let ok = unsafe {
            WTSQuerySessionInformationW(None, e.SessionId, WTSUserName, &mut buffer, &mut bytes)
        };
        if ok.is_err() || buffer.is_null() {
            continue;
        }
        let user = pwstr_to_string(buffer);
        unsafe { WTSFreeMemory(buffer.as_ptr() as *mut std::ffi::c_void) };
        if user.is_empty() {
            continue;
        }
        out.push(format!("{} {}", user, pwstr_to_string(e.pWinStationName)));
    }
    unsafe { WTSFreeMemory(raw as *mut std::ffi::c_void) };
    Some(out)
}

fn pwstr_to_string(p: windows::core::PWSTR) -> String {
    if p.is_null() {
        return String::new();
    }
    unsafe {
        let mut len = 0usize;
        while *p.0.add(len) != 0 {
            len += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(p.0, len))
    }
}

/// Fallback when the Terminal Services API is unavailable: best-effort
/// `query user` with a `whoami` last resort.
fn collect_users(query_program: &str, whoami_program: &str) -> Vec<String> {
    let mut users = Vec::new();

    if let Ok(output) = std::process::Command::new(query_program)
        .arg("user")
        .output()
    {
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
    }

    if users.is_empty() {
        if let Ok(output) = std::process::Command::new(whoami_program).output() {
            let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !username.is_empty() {
                users.push(username);
            }
        }
    }

    if users.is_empty() {
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
    use super::collect_users;

    // A name that cannot be resolved by any PATH search, forcing a launch
    // failure without touching the real system tools.
    const MISSING: &str = "ruxcmd-w-missing-command-under-test";

    #[test]
    fn users_degrade_when_sources_cannot_launch() {
        // Regression: used to panic via .expect() when query user / whoami
        // could not be launched (e.g. Windows Home without query.exe).
        assert_eq!(collect_users(MISSING, MISSING), vec!["unavailable"]);
    }
}
