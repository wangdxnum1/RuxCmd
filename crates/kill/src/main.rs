mod cli;

use clap::Parser;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess};

const PROCESS_TERMINATE: u32 = 0x0001;

fn main() {
    let args = cli::Args::parse();

    if args.list_signals {
        list_signals();
        return;
    }

    let signal = args.signal.as_deref().unwrap_or("TERM");

    for pid in &args.pids {
        match kill_process(*pid, signal) {
            Ok(_) => (),
            Err(e) => eprintln!("kill: {}: {}", pid, e),
        }
    }
}

fn list_signals() {
    println!("Windows signals:");
    println!("  TERM  - Terminate process");
    println!("  KILL  - Force kill process");
}

#[cfg(windows)]
fn kill_process(pid: i32, signal: &str) -> Result<(), String> {
    use std::ptr;

    let handle: HANDLE = unsafe { OpenProcess(PROCESS_TERMINATE, 0, pid as u32) };

    if handle == ptr::null_mut() {
        return Err("cannot open process".to_string());
    }

    let exit_code = match signal {
        "KILL" => 9,
        _ => 15,
    };

    let result = unsafe { TerminateProcess(handle, exit_code) };

    unsafe { CloseHandle(handle) };

    if result == 0 {
        Err("cannot terminate process".to_string())
    } else {
        Ok(())
    }
}

#[cfg(unix)]
fn kill_process(pid: i32, signal: &str) -> Result<(), String> {
    Err("not supported on this platform".to_string())
}
