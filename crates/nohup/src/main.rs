mod cli;

use clap::Parser;
use std::io::{self, IsTerminal};
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use windows_sys::Win32::Foundation::{DUPLICATE_SAME_ACCESS, DuplicateHandle};
use windows_sys::Win32::System::Threading::{
    CREATE_BREAKAWAY_FROM_JOB, DETACHED_PROCESS, GetCurrentProcess,
};

fn main() {
    let args = cli::Cli::parse();

    if args.command.is_empty() {
        eprintln!("nohup: missing operand");
        eprintln!("Try 'nohup --help' for more information.");
        std::process::exit(1);
    }

    let mut command = Command::new(&args.command[0]);
    command.args(&args.command[1..]);

    command.stdin(if io::stdin().is_terminal() {
        Stdio::null()
    } else {
        Stdio::inherit()
    });
    // Preserve redirected streams; terminal output needs a durable file.
    let mut output_file = None;
    if io::stdout().is_terminal() {
        match open_output_file() {
            Ok((file, path)) => {
                eprintln!(
                    "nohup: ignoring input and appending output to '{}'",
                    path.display()
                );
                output_file = Some(file);
            }
            Err(e) => {
                eprintln!("nohup: cannot open nohup.out: {}", e);
                std::process::exit(1);
            }
        }
    }

    match &output_file {
        Some(file) => {
            let stdout = match file.try_clone() {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("nohup: cannot share nohup.out: {}", e);
                    std::process::exit(1);
                }
            };
            command.stdout(Stdio::from(stdout));
            if io::stderr().is_terminal() {
                match file.try_clone() {
                    Ok(stderr) => {
                        command.stderr(Stdio::from(stderr));
                    }
                    Err(error) => {
                        eprintln!("nohup: cannot share nohup.out: {error}");
                        std::process::exit(1);
                    }
                }
            } else {
                command.stderr(Stdio::inherit());
            }
        }
        None => {
            command.stdout(Stdio::inherit());
            if io::stderr().is_terminal() {
                match duplicate_stdout_handle() {
                    Some(stderr) => {
                        command.stderr(stderr);
                    }
                    None => {
                        eprintln!("nohup: cannot share standard output");
                        std::process::exit(1);
                    }
                }
            } else {
                command.stderr(Stdio::inherit());
            }
        }
    }

    // Detach from console and try to leave the enclosing job. Restricted
    // jobs may deny breakaway; report that reduced lifetime guarantee.
    command.creation_flags(CREATE_BREAKAWAY_FROM_JOB | DETACHED_PROCESS);
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) if error.raw_os_error() == Some(5) => {
            command.creation_flags(DETACHED_PROCESS);
            match command.spawn() {
                Ok(child) => {
                    eprintln!(
                        "nohup: warning: job prevents breakaway; child remains subject to job lifetime"
                    );
                    child
                }
                Err(error) => {
                    eprintln!("nohup: {}: {error}", args.command[0]);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("nohup: {}: {}", args.command[0], e);
            std::process::exit(if e.kind() == io::ErrorKind::NotFound {
                127
            } else {
                1
            });
        }
    };

    let status = match child.wait() {
        Ok(status) => status,
        Err(e) => {
            eprintln!("nohup: {}: {}", args.command[0], e);
            std::process::exit(1);
        }
    };
    std::process::exit(status.code().unwrap_or(1));
}

/// Opens nohup.out for appending next to the current directory, falling back
/// to the temporary directory when the current one is not writable. Returns
/// the file plus the path to name in the user-facing message.
fn open_output_file() -> io::Result<(std::fs::File, PathBuf)> {
    let cwd = std::env::current_dir().unwrap_or_default();
    open_output_in(Some(&cwd))
}

fn open_output_in(base: Option<&Path>) -> io::Result<(std::fs::File, PathBuf)> {
    if let Some(base) = base {
        let candidate = base.join("nohup.out");
        if let Ok(file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&candidate)
        {
            return Ok((file, PathBuf::from("nohup.out")));
        }
    }
    let candidate = std::env::temp_dir().join("nohup.out");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&candidate)?;
    Ok((file, candidate))
}

/// Duplicates our own stdout handle so the child's stderr can point at the
/// same stream even when stdout is a pipe rather than a console.
fn duplicate_stdout_handle() -> Option<Stdio> {
    let raw = io::stdout().as_raw_handle();
    if raw.is_null() {
        return None;
    }
    let process = unsafe { GetCurrentProcess() };
    let mut duplicate: windows_sys::Win32::Foundation::HANDLE = std::ptr::null_mut();
    let ok = unsafe {
        DuplicateHandle(
            process,
            raw,
            process,
            &mut duplicate,
            0,
            0,
            DUPLICATE_SAME_ACCESS,
        )
    };
    if ok == 0 {
        return None;
    }
    let owned = unsafe { OwnedHandle::from_raw_handle(duplicate) };
    Some(Stdio::from(owned))
}

#[cfg(test)]
mod tests {
    use super::open_output_in;

    #[test]
    fn uses_nohup_out_in_base_when_writable() {
        let dir = tempfile::tempdir().unwrap();
        let (file, shown) = open_output_in(Some(dir.path())).unwrap();
        assert_eq!(shown, std::path::PathBuf::from("nohup.out"));
        drop(file);
        // A leftover file from an earlier run is fine; appending must not fail.
        let (_, shown_again) = open_output_in(Some(dir.path())).unwrap();
        assert_eq!(shown_again, std::path::PathBuf::from("nohup.out"));
    }

    #[test]
    fn falls_back_to_temp_when_base_not_writable() {
        // The parent of this path is an existing FILE, so creating nohup.out
        // inside it must fail and force the %TEMP% fallback.
        let file = tempfile::NamedTempFile::new().unwrap();
        let blocked = file.path().join("blocked");
        let (out, shown) = open_output_in(Some(&blocked)).unwrap();
        assert_eq!(shown, std::env::temp_dir().join("nohup.out"));
        drop(out);
    }
}
