mod cli;

use clap::Parser;
use cli::Args;
use ps_sys::{collect_with, CollectOptions};
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess};

const PROCESS_TERMINATE: u32 = 0x0001;

fn main() {
    let args = Args::parse();
    let self_pid = std::process::id();

    let signal_exit_code = signal_code(&args.signal);
    let signal = match signal_exit_code {
        Ok(code) => code,
        Err(name) => {
            eprintln!("pkill: {}: invalid signal", name);
            std::process::exit(1);
        }
    };

    let opts = if args.full {
        CollectOptions::FULL
    } else {
        CollectOptions::FAST
    };
    let procs = match collect_with(opts) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("pkill: cannot enumerate processes: {}", e);
            std::process::exit(1);
        }
    };

    let mut killed = 0usize;
    let mut failed = 0usize;
    for p in &procs {
        if p.pid == self_pid || !matches(&args, p) {
            continue;
        }
        match terminate(p.pid, signal) {
            Ok(()) => killed += 1,
            Err(e) => {
                failed += 1;
                eprintln!("pkill: {} ({}): {}", p.pid, p.name, e);
            }
        }
    }

    if failed > 0 || killed == 0 {
        std::process::exit(1);
    }
}

/// All signals map to TerminateProcess; the number only sets the exit code.
fn signal_code(signal: &Option<String>) -> Result<u32, String> {
    let Some(name) = signal.as_deref() else {
        return Ok(15);
    };
    match name.to_ascii_uppercase().as_str() {
        "TERM" | "15" => Ok(15),
        "KILL" | "9" => Ok(9),
        "INT" | "2" => Ok(2),
        "HUP" | "1" => Ok(1),
        "QUIT" | "3" => Ok(3),
        "ABRT" | "6" => Ok(6),
        _ => Err(name.to_string()),
    }
}

/// Image-name comparison is ASCII case-insensitive (Toolhelp32 casing is
/// unreliable); -f cmdline matching stays case-sensitive unless -i.
fn matches(args: &Args, p: &ps_sys::ProcessRaw) -> bool {
    let pat = args.pattern.to_lowercase();
    let hay_name = p.name.to_lowercase();
    let hay_stem = hay_name
        .strip_suffix(".exe")
        .unwrap_or(&hay_name)
        .to_string();

    let name_hit = if args.exact {
        pat == hay_name || pat == hay_stem
    } else {
        hay_name.contains(&pat) || hay_stem.contains(&pat)
    };
    if name_hit {
        return true;
    }
    if args.full {
        if let Some(cmd) = &p.cmdline {
            let hay_cmd = if args.ignore_case {
                cmd.to_lowercase()
            } else {
                cmd.clone()
            };
            return hay_cmd.contains(&args.pattern);
        }
    }
    false
}

fn terminate(pid: u32, exit_code: u32) -> Result<(), String> {
    unsafe {
        let handle: HANDLE = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            return Err("cannot open process".to_string());
        }
        let ok = TerminateProcess(handle, exit_code);
        CloseHandle(handle);
        if ok == 0 {
            Err("cannot terminate process".to_string())
        } else {
            Ok(())
        }
    }
}
