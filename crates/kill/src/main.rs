mod cli;

use clap::Parser;
use std::ffi::OsString;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess};

const PROCESS_TERMINATE: u32 = 0x0001;

/// GNU-style signal table; every entry maps to TerminateProcess on Windows
/// and only sets the exit code of the terminated process.
const SIGNALS: &[(&str, u32)] = &[
    ("HUP", 1),
    ("INT", 2),
    ("QUIT", 3),
    ("ABRT", 6),
    ("KILL", 9),
    ("ALRM", 14),
    ("TERM", 15),
];

fn main() {
    // Rewrite `kill -9 PID` / `kill -KILL PID` into `--signal 9 PID` so the
    // classic attached form works without confusing clap with negative PIDs.
    let argv: Vec<OsString> = std::env::args_os().collect();
    let args = cli::Args::parse_from(rewrite_signal_flags(argv));

    if args.list_signals {
        list_signals();
        return;
    }

    let signal = args.signal.as_deref().unwrap_or("TERM");
    let exit_code = match signal_exit_code(signal) {
        Ok(code) => code,
        Err(_) => {
            eprintln!("kill: {}: invalid signal", signal);
            std::process::exit(1);
        }
    };

    if args.pids.is_empty() {
        eprintln!("kill: missing pid operand");
        std::process::exit(1);
    }

    let mut failed = false;
    for pid in &args.pids {
        if let Err(e) = kill_process(*pid, exit_code) {
            eprintln!("kill: {}: {}", pid, e);
            failed = true;
        }
    }
    if failed {
        std::process::exit(1);
    }
}

fn rewrite_signal_flags(argv: Vec<OsString>) -> Vec<OsString> {
    let mut out = Vec::with_capacity(argv.len() + 2);
    for a in argv {
        let text = a.to_string_lossy();
        let signal_value = match text.strip_prefix('-') {
            Some(rest) if !rest.is_empty() => {
                let digits = rest.chars().all(|c| c.is_ascii_digit());
                let base = rest
                    .to_ascii_uppercase()
                    .strip_prefix("SIG")
                    .unwrap_or(&rest.to_ascii_uppercase())
                    .to_string();
                let known = SIGNALS.iter().any(|(name, _)| *name == base);
                // All-uppercase unknown names (e.g. -XYZ) also go through the
                // signal path so they fail validation with exit 1, not clap 2.
                let upper_name = rest.len() >= 2 && rest.chars().all(|c| c.is_ascii_uppercase());
                if digits || known || upper_name {
                    Some(rest.to_string())
                } else {
                    None
                }
            }
            _ => None,
        };
        match signal_value {
            Some(v) => {
                out.push(OsString::from("--signal"));
                out.push(OsString::from(v));
            }
            None => out.push(a),
        }
    }
    out
}

fn signal_exit_code(signal: &str) -> Result<u32, ()> {
    // Accept both KILL and SIGKILL spellings.
    let upper = signal
        .to_ascii_uppercase()
        .strip_prefix("SIG")
        .unwrap_or(&signal.to_ascii_uppercase())
        .to_string();
    for (name, code) in SIGNALS {
        if *name == upper {
            return Ok(*code);
        }
    }
    if let Ok(n) = signal.parse::<u32>() {
        if n <= 31 {
            return Ok(n);
        }
    }
    Err(())
}

fn list_signals() {
    for (name, code) in SIGNALS {
        println!("{:<3} {}", code, name);
    }
}

#[cfg(windows)]
fn kill_process(pid: i32, exit_code: u32) -> Result<(), String> {
    use std::ptr;

    let handle: HANDLE = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid as u32) };
    if handle == ptr::null_mut() {
        return Err("cannot open process".to_string());
    }
    if exit_code == 0 {
        // Signal 0 probes existence only, like GNU kill.
        unsafe { CloseHandle(handle) };
        return Ok(());
    }
    let result = unsafe { TerminateProcess(handle, exit_code) };
    unsafe { CloseHandle(handle) };
    if result == 0 {
        Err("cannot terminate process".to_string())
    } else {
        Ok(())
    }
}

#[cfg(unix)]
fn kill_process(pid: i32, exit_code: u32) -> Result<(), String> {
    let _ = (pid, exit_code);
    Err("not supported on this platform".to_string())
}
