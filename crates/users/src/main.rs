mod cli;

use clap::Parser;
use std::collections::HashSet;
use windows_sys::Win32::System::RemoteDesktop::{
    WTS_SESSION_INFOW, WTSEnumerateSessionsW, WTSFreeMemory, WTSQuerySessionInformationW,
    WTSUserName,
};
use windows_sys::core::PWSTR;

fn main() {
    let args = cli::Args::parse();

    let names = logged_on_users();

    if args.count {
        println!("{}", names.len());
        return;
    }

    // GNU users prints every name on one line separated by single spaces.
    println!("{}", names.join(" "));
}

/// Distinct user names of live sessions, first-seen order preserved. When the
/// Terminal Services enumeration yields nothing, the current process identity
/// stands in — the same degradation as the who port.
fn logged_on_users() -> Vec<String> {
    let users = session_users();
    let names = dedup_preserving_order(users);

    if names.is_empty() {
        return vec![fallback_user()];
    }
    names
}

/// SAFETY: session ids come from WTSEnumerateSessionsW; the returned buffers
/// are released with WTSFreeMemory.
fn session_users() -> Vec<String> {
    let mut raw: *mut WTS_SESSION_INFOW = std::ptr::null_mut();
    let mut count: u32 = 0;
    let ok = unsafe { WTSEnumerateSessionsW(std::ptr::null_mut(), 0, 1, &mut raw, &mut count) };
    if ok == 0 || raw.is_null() {
        return Vec::new();
    }

    let entries = unsafe { std::slice::from_raw_parts(raw, count as usize) };
    let mut out = Vec::new();
    for e in entries {
        let user = query_user_name(e.SessionId);
        if !user.is_empty() {
            out.push(user);
        }
    }
    unsafe { WTSFreeMemory(raw as *mut std::ffi::c_void) };
    out
}

/// SAFETY: session ids come from WTSEnumerateSessionsW; the returned buffer is
/// released with WTSFreeMemory.
fn query_user_name(session_id: u32) -> String {
    let mut buffer: PWSTR = std::ptr::null_mut();
    let mut bytes: u32 = 0;
    let ok = unsafe {
        WTSQuerySessionInformationW(
            std::ptr::null_mut(),
            session_id,
            WTSUserName,
            &mut buffer,
            &mut bytes,
        )
    };
    if ok == 0 || buffer.is_null() {
        return String::new();
    }
    let s = pwstr_to_string(buffer);
    unsafe { WTSFreeMemory(buffer as *mut std::ffi::c_void) };
    s
}

fn pwstr_to_string(p: PWSTR) -> String {
    unsafe {
        let mut len = 0usize;
        while *p.add(len) != 0 {
            len += 1;
        }
        String::from_utf16_lossy(std::slice::from_raw_parts(p, len))
    }
}

fn fallback_user() -> String {
    std::env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string())
}

/// Keep the first occurrence of each name (GNU users lists each user once).
fn dedup_preserving_order(names: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for name in names {
        if seen.insert(name.clone()) {
            out.push(name);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{dedup_preserving_order, logged_on_users};

    #[test]
    fn dedup_keeps_first_occurrence_order() {
        let names = vec!["alice".into(), "bob".into(), "alice".into(), "carol".into()];
        assert_eq!(dedup_preserving_order(names), vec!["alice", "bob", "carol"]);
        assert!(dedup_preserving_order(Vec::new()).is_empty());
    }

    #[test]
    fn output_is_never_empty_on_a_desktop() {
        // Either WTS enumeration or the USERNAME fallback must yield a name.
        assert!(!logged_on_users().is_empty());
    }
}
