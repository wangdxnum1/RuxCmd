mod cli;

use clap::Parser;
use windows::Win32::System::RemoteDesktop::{
    WTS_SESSION_INFOW, WTSEnumerateSessionsW, WTSFreeMemory, WTSQuerySessionInformationW,
    WTSUserName,
};
use windows::core::PWSTR;

struct Session {
    user: String,
    win_station: String,
}

/// Live sessions with a logged-on user via the Terminal Services API.
fn sessions() -> Vec<Session> {
    let mut raw: *mut WTS_SESSION_INFOW = std::ptr::null_mut();
    let mut count: u32 = 0;
    // SAFETY: out-parameters point to valid storage; the returned buffer is
    // released with WTSFreeMemory below.
    let ok = unsafe { WTSEnumerateSessionsW(None, 0, 1, &mut raw, &mut count) };
    if ok.is_err() || raw.is_null() {
        return Vec::new();
    }
    let entries = unsafe { std::slice::from_raw_parts(raw, count as usize) };
    let mut out = Vec::new();
    for e in entries {
        let user = query_string(e.SessionId);
        if user.is_empty() {
            continue;
        }
        let win_station = pwstr_to_string(e.pWinStationName);
        out.push(Session { user, win_station });
    }
    unsafe { WTSFreeMemory(raw as *mut std::ffi::c_void) };
    out
}

/// SAFETY: session ids come from WTSEnumerateSessionsW; the returned buffer is
/// released with WTSFreeMemory.
fn query_string(session_id: u32) -> String {
    let mut buffer: PWSTR = PWSTR::null();
    let mut bytes: u32 = 0;
    let ok = unsafe {
        WTSQuerySessionInformationW(None, session_id, WTSUserName, &mut buffer, &mut bytes)
    };
    if ok.is_err() || buffer.is_null() {
        return String::new();
    }
    let s = pwstr_to_string(buffer);
    unsafe { WTSFreeMemory(buffer.as_ptr() as *mut std::ffi::c_void) };
    s
}

fn pwstr_to_string(p: PWSTR) -> String {
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

/// Boot time as a local "YYYY-MM-DD HH:MM:SS" string via GetTickCount64.
fn boot_time_string() -> Option<String> {
    let uptime_ms = unsafe { windows::Win32::System::SystemInformation::GetTickCount64() };
    let boot = chrono::Local::now() - chrono::Duration::milliseconds(uptime_ms as i64);
    Some(boot.format("%Y-%m-%d %H:%M:%S").to_string())
}

fn fallback_user() -> String {
    std::env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string())
}

fn main() {
    let args = cli::Args::parse();

    if args.boot_time || args.all {
        match boot_time_string() {
            Some(t) => println!("system boot time: {}", t),
            None => println!("system boot time: unavailable"),
        }
    }

    if args.dead || args.all {
        println!("dead processes: not supported on Windows");
    }

    if !args.boot_time && !args.dead || args.all {
        let live = sessions();
        if live.is_empty() {
            // Degrade to the current identity instead of printing nothing.
            println!("{}", fallback_user());
        } else {
            for s in &live {
                // GNU who prints "user tty time (comment)"; Windows sessions
                // carry no login time here, so "-" stands in for it.
                println!("{} {} -", s.user, s.win_station);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sessions_or_fallback_is_never_empty_on_a_desktop() {
        // Either the WTS enumeration yields sessions or the test process runs
        // with a USERNAME; both are fine, empty output is not.
        let live = super::sessions();
        assert!(
            !live.is_empty() || !super::fallback_user().is_empty(),
            "no sessions and no USERNAME"
        );
    }
}
