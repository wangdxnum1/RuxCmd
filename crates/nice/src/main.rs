mod cli;

use clap::Parser;
use cli::Args;
use std::os::windows::io::AsRawHandle;
use std::process::Command;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Threading::{
    SetPriorityClass, ABOVE_NORMAL_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS,
    HIGH_PRIORITY_CLASS, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, PROCESS_CREATION_FLAGS,
};

fn main() {
    let args = Args::parse();

    if args.command.is_empty() {
        eprintln!("nice: missing command");
        std::process::exit(1);
    }

    let program = &args.command[0];

    let mut child = Command::new(program)
        .args(&args.command[1..])
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("nice: failed to execute '{}': {}", program, e);
            std::process::exit(1);
        });

    let class = priority_class(args.adjustment);
    let handle = HANDLE(child.as_raw_handle());
    if let Err(e) = unsafe { SetPriorityClass(handle, class) } {
        eprintln!("nice: warning: failed to set priority class: {}", e);
    }

    let status = match child.wait() {
        Ok(status) => status,
        Err(e) => {
            eprintln!("nice: failed to wait for '{}': {}", program, e);
            std::process::exit(1);
        }
    };
    std::process::exit(status.code().unwrap_or(1));
}

/// Maps a nice adjustment to a Windows priority class with GNU semantics:
/// a higher nice value means a lower priority, so negative adjustments
/// raise the priority and positive ones lower it.
///
/// -20..=-15 -> HIGH_PRIORITY_CLASS
///   -14..=-5 -> ABOVE_NORMAL_PRIORITY_CLASS
///    -4..=4  -> NORMAL_PRIORITY_CLASS
///     5..=14 -> BELOW_NORMAL_PRIORITY_CLASS
///    15..=20 -> IDLE_PRIORITY_CLASS
///
/// Values outside -20..19 are accepted (GNU nice allows them) and clamp to
/// the end classes. REALTIME_PRIORITY_CLASS is deliberately never used.
fn priority_class(adjustment: i32) -> PROCESS_CREATION_FLAGS {
    if adjustment <= -15 {
        HIGH_PRIORITY_CLASS
    } else if adjustment <= -5 {
        ABOVE_NORMAL_PRIORITY_CLASS
    } else if adjustment <= 4 {
        NORMAL_PRIORITY_CLASS
    } else if adjustment <= 14 {
        BELOW_NORMAL_PRIORITY_CLASS
    } else {
        IDLE_PRIORITY_CLASS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::{
        ABOVE_NORMAL_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS,
        IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
    };

    #[test]
    fn maps_adjustment_to_priority_class() {
        // Negative adjustments raise the priority (GNU nice semantics).
        assert_eq!(priority_class(i32::MIN), HIGH_PRIORITY_CLASS);
        assert_eq!(priority_class(-20), HIGH_PRIORITY_CLASS);
        assert_eq!(priority_class(-15), HIGH_PRIORITY_CLASS);
        assert_eq!(priority_class(-14), ABOVE_NORMAL_PRIORITY_CLASS);
        assert_eq!(priority_class(-5), ABOVE_NORMAL_PRIORITY_CLASS);
        assert_eq!(priority_class(-4), NORMAL_PRIORITY_CLASS);
        assert_eq!(priority_class(0), NORMAL_PRIORITY_CLASS);
        assert_eq!(priority_class(4), NORMAL_PRIORITY_CLASS);
        assert_eq!(priority_class(5), BELOW_NORMAL_PRIORITY_CLASS);
        assert_eq!(priority_class(14), BELOW_NORMAL_PRIORITY_CLASS);
        assert_eq!(priority_class(15), IDLE_PRIORITY_CLASS);
        assert_eq!(priority_class(20), IDLE_PRIORITY_CLASS);
        assert_eq!(priority_class(i32::MAX), IDLE_PRIORITY_CLASS);
    }
}
