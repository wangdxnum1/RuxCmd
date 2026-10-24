mod cli;

use clap::Parser;
use cli::Args;
use std::os::windows::io::AsRawHandle;
use std::process::Command;
use std::time::Instant;
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::System::Threading::GetProcessTimes;

fn main() {
    let args = Args::parse();

    if args.command.is_empty() {
        eprintln!("time: missing command");
        std::process::exit(1);
    }

    let program = &args.command[0];
    let rest: Vec<&str> = args.command[1..].iter().map(|s| s.as_str()).collect();

    let start = Instant::now();

    let mut child = Command::new(program)
        .args(rest)
        .spawn()
        .unwrap_or_else(|e| {
            // 127 is the shell convention for "command not found".
            let code = if e.kind() == std::io::ErrorKind::NotFound {
                127
            } else {
                1
            };
            eprintln!("time: failed to execute '{}': {}", program, e);
            std::process::exit(code);
        });

    let status = child.wait().unwrap_or_else(|e| {
        eprintln!("time: failed to wait for '{}': {}", program, e);
        std::process::exit(1);
    });

    let elapsed = start.elapsed();
    let (user_secs, sys_secs) = child_cpu_times(child.as_raw_handle());

    println!();
    println!("real\t{}", gnu_duration(elapsed.as_secs_f64()));
    println!("user\t{}", gnu_duration(user_secs));
    println!("sys\t{}", gnu_duration(sys_secs));

    std::process::exit(status.code().unwrap_or(1));
}

/// Kernel and user CPU seconds of the finished child, (0.0, 0.0) if the
/// query fails. FILETIME values are 100 ns units since 1601-01-01, so the
/// absolute epoch cancels in the deltas.
fn child_cpu_times(handle: std::os::windows::io::RawHandle) -> (f64, f64) {
    let mut creation = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    // SAFETY: all out-parameters point to valid FILETIME storage and the
    // handle came from std's Child (still valid after wait).
    let ok = unsafe {
        GetProcessTimes(
            handle as _,
            &mut creation,
            &mut exit,
            &mut kernel,
            &mut user,
        )
    };
    if ok == 0 {
        return (0.0, 0.0);
    }
    let secs = |ft: &FILETIME| {
        let ticks = (ft.dwHighDateTime as u64) << 32 | ft.dwLowDateTime as u64;
        ticks as f64 * 1e-7
    };
    (secs(&user), secs(&kernel))
}

/// GNU time's `0m0.003s` format.
fn gnu_duration(secs: f64) -> String {
    let whole = secs.trunc() as u64;
    let frac = secs - secs.trunc();
    format!("{}m{:.3}s", whole / 60, (whole % 60) as f64 + frac)
}

#[cfg(test)]
mod tests {
    use super::gnu_duration;

    #[test]
    fn gnu_duration_formats_minutes_and_fraction() {
        assert_eq!(gnu_duration(0.003), "0m0.003s");
        assert_eq!(gnu_duration(2.5), "0m2.500s");
        assert_eq!(gnu_duration(65.25), "1m5.250s");
    }
}
