mod cli;

use clap::Parser;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let args = cli::Args::parse();

    let paths = if args.paths.is_empty() {
        vec![std::path::PathBuf::from(".")]
    } else {
        args.paths.clone()
    };

    // `-a APP PATH...` (and `-e`) launches APP once with all operands as its
    // command line, like macOS open. The operands belong to APP, so they
    // must not be routed through the per-operand association lookup.
    if !args.reveal && (args.edit || args.application.is_some()) {
        let app = if args.edit {
            "notepad.exe".to_string()
        } else {
            args.application.clone().unwrap()
        };
        let parameters = join_parameters(&paths);
        if let Err(e) = shell_execute(&app, Some(&parameters), args.wait) {
            eprintln!("open: {}", e);
            std::process::exit(1);
        }
        std::process::exit(0);
    }

    let mut exit_code = 0i32;

    for path in &paths {
        if let Err(e) = open_path(path, &args) {
            eprintln!("open: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

fn open_path(path: &Path, args: &cli::Args) -> Result<(), String> {
    if args.reveal {
        return reveal_in_explorer(path);
    }

    let path_str = path.to_string_lossy();

    // ShellExecuteW routes files, directories and URLs (including `&` and
    // other metacharacters) to their associations untouched — there is no
    // cmd/PowerShell shell in between to truncate or expand them.
    shell_execute(&path_str, None, args.wait)
}

/// Build the lpParameters command line for `-a APP PATH...`: quote each
/// operand that contains whitespace, join with single spaces.
fn join_parameters(paths: &[std::path::PathBuf]) -> String {
    paths
        .iter()
        .map(|p| {
            let s = p.to_string_lossy();
            if s.is_empty() {
                "\"\"".to_string()
            } else if s.bytes().any(|b| b == b' ' || b == b'\t') {
                format!("\"{}\"", s)
            } else {
                s.into_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn reveal_in_explorer(path: &Path) -> Result<(), String> {
    let path_str = path.to_string_lossy();

    let is_dir = fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false);

    // explorer.exe routinely exits with code 1 even on success (e.g. when an
    // existing window reuses the launch), so its exit status is deliberately
    // ignored; only a failure to start it at all is an error.
    let mut cmd = Command::new("explorer.exe");
    if is_dir {
        cmd.arg(&*path_str);
    } else {
        cmd.arg("/select,").arg(&*path_str);
    }

    cmd.status()
        .map_err(|e| format!("failed to start explorer.exe: {}", e))?;
    Ok(())
}

/// Open `file` through the shell association mechanism (ShellExecuteExW).
/// When `wait` is set (-W) the launched process is awaited via
/// SEE_MASK_NOCLOSEPROCESS + WaitForSingleObject.
#[cfg(windows)]
fn shell_execute(file: &str, parameters: Option<&str>, wait: bool) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE};
    use windows_sys::Win32::System::Threading::{WaitForSingleObject, INFINITE};
    use windows_sys::Win32::UI::Shell::{
        ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    let file_w = to_wide(file);
    let verb_w = to_wide("open");
    let params_w = parameters.map(to_wide);

    let mut sei: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
    sei.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
    // SEE_MASK_NOASYNC: this process exits right after launching, so the
    // shell must not defer the call to a DDE conversation that would die
    // with it. SEE_MASK_NOCLOSEPROCESS (only for -W) returns a process
    // handle we can wait on.
    sei.fMask = SEE_MASK_NOASYNC
        | if wait {
            SEE_MASK_NOCLOSEPROCESS
        } else {
            0
        };
    sei.lpVerb = verb_w.as_ptr();
    sei.lpFile = file_w.as_ptr();
    sei.lpParameters = match &params_w {
        Some(p) => p.as_ptr(),
        None => std::ptr::null(),
    };
    sei.nShow = SW_SHOWNORMAL;

    let launched = unsafe { ShellExecuteExW(&mut sei) };
    if launched == 0 {
        let err = unsafe { GetLastError() };
        return Err(format!("cannot open '{}': error {}", file, err));
    }

    if wait {
        let handle: HANDLE = sei.hProcess;
        if !handle.is_null() {
            unsafe {
                WaitForSingleObject(handle, INFINITE);
                CloseHandle(handle);
            }
        }
        // Without a process handle (e.g. protocol handlers handing off to an
        // already-running browser) there is nothing to wait on; -W then
        // behaves like a plain open, matching macOS best-effort semantics.
    }

    Ok(())
}

#[cfg(not(windows))]
fn shell_execute(_file: &str, _parameters: Option<&str>, _wait: bool) -> Result<(), String> {
    Err("open is only supported on Windows".to_string())
}
