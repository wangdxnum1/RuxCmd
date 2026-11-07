mod cli;
mod input;

use ruxcmd_core::command_batch::{CommandBudget, MAX_COMMAND_UNITS, command_units};
use std::io::{self, Write};
use std::process::{Command, Stdio};

fn main() {
    let code = match cli::Args::parse() {
        Ok(args) => match run(args) {
            Ok(code) => code,
            Err(code) => code,
        },
        Err(error) => {
            eprintln!("xargs: {error}");
            1
        }
    };
    std::process::exit(code);
}

fn fail(message: impl std::fmt::Display) -> i32 {
    eprintln!("xargs: {message}");
    1
}

fn run(mut args: cli::Args) -> Result<i32, i32> {
    let internal_echo = args.command.is_empty();
    if internal_echo {
        args.command.push("echo".into());
    }
    let mut executor = Executor {
        internal_echo,
        verbose: args.verbose,
        interactive: args.interactive,
        status: 0,
    };
    let stdin = io::stdin();
    let mut items = input::Items::new(stdin.lock(), args.null_delimited, args.replace.is_some());
    if let Some(marker) = &args.replace {
        while let Some(item) = items.next().map_err(fail)? {
            let command = replacement_command(&args.command, marker, &item)?;
            executor.execute(&command)?;
        }
        return Ok(executor.status);
    }
    let mut budget = CommandBudget::new(&args.command).map_err(fail)?;
    let mut batch = args.command.clone();
    let fixed_len = batch.len();
    let mut seen = false;
    while let Some(item) = items.next().map_err(fail)? {
        seen = true;
        if !budget.can_fit_item(&item) {
            return Err(fail("input item exceeds command-line limit"));
        }
        if !budget.try_add(&item) {
            executor.execute(&batch)?;
            batch.truncate(fixed_len);
            budget.reset();
            assert!(budget.try_add(&item));
        }
        batch.push(item);
        if batch.len() - fixed_len >= args.max_args {
            executor.execute(&batch)?;
            batch.truncate(fixed_len);
            budget.reset();
        }
    }
    if batch.len() > fixed_len || (!seen && !args.no_run_if_empty) {
        executor.execute(&batch)?;
    }
    Ok(executor.status)
}

fn replacement_command(template: &[String], marker: &str, item: &str) -> Result<Vec<String>, i32> {
    let mut command = Vec::new();
    let mut used = 1; // terminating NUL
    for argument in template {
        let mut replaced = String::new();
        let mut argument_units = 0;
        for (index, part) in argument.split(marker).enumerate() {
            for text in [if index == 0 { "" } else { item }, part] {
                argument_units += text.encode_utf16().count();
                if argument_units > MAX_COMMAND_UNITS {
                    return Err(fail("expanded command exceeds command-line limit"));
                }
                replaced.push_str(text);
            }
        }
        used += command_units(&[replaced.as_str()]) - 1 + usize::from(!command.is_empty());
        if used > MAX_COMMAND_UNITS {
            return Err(fail("expanded command exceeds command-line limit"));
        }
        command.push(replaced);
    }
    Ok(command)
}

struct Executor {
    internal_echo: bool,
    verbose: bool,
    interactive: bool,
    status: i32,
}

impl Executor {
    fn execute(&mut self, command: &[String]) -> Result<(), i32> {
        if command_units(command) > MAX_COMMAND_UNITS {
            return Err(fail("expanded command exceeds command-line limit"));
        }
        if self.verbose || self.interactive {
            eprintln!(
                "{}",
                command
                    .iter()
                    .map(|a| format!("{a:?}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        if self.interactive {
            eprint!("?... ");
            io::stderr().flush().map_err(fail)?;
            if !matches!(
                read_console_line().trim().to_lowercase().as_str(),
                "y" | "yes"
            ) {
                return Ok(());
            }
        }
        if self.internal_echo {
            let mut out = io::stdout().lock();
            writeln!(out, "{}", command[1..].join(" ")).map_err(fail)?;
            return Ok(());
        }
        match Command::new(&command[0])
            .args(&command[1..])
            .stdin(Stdio::null())
            .status()
        {
            Ok(status) if status.success() => Ok(()),
            Ok(status) if status.code() == Some(255) => {
                eprintln!("xargs: {}: exited with status 255; aborting", command[0]);
                Err(124)
            }
            Ok(_) => {
                self.status = 123;
                Ok(())
            }
            Err(error) => {
                eprintln!("xargs: {}: {error}", command[0]);
                Err(if error.kind() == io::ErrorKind::NotFound {
                    127
                } else {
                    126
                })
            }
        }
    }
}

/// The confirm prompt must read the terminal even though stdin has been
/// consumed; GNU xargs reads /dev/tty, Windows opens CONIN$.
fn read_console_line() -> String {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = std::ffi::OsStr::new("CONIN$")
        .encode_wide()
        .chain(Some(0))
        .collect();
    let handle = unsafe {
        windows_sys::Win32::Storage::FileSystem::CreateFileW(
            wide.as_ptr(),
            windows_sys::Win32::Foundation::GENERIC_READ
                | windows_sys::Win32::Foundation::GENERIC_WRITE,
            windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ
                | windows_sys::Win32::Storage::FileSystem::FILE_SHARE_WRITE,
            std::ptr::null(),
            windows_sys::Win32::Storage::FileSystem::OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    };
    if handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
        return String::new(); // no console (piped run): treat as "no"
    }
    let mut buf = [0u8; 256];
    let mut read = 0u32;
    let ok = unsafe {
        windows_sys::Win32::Storage::FileSystem::ReadFile(
            handle,
            buf.as_mut_ptr(),
            buf.len() as u32,
            &mut read,
            std::ptr::null_mut(),
        )
    };
    unsafe {
        windows_sys::Win32::Foundation::CloseHandle(handle);
    }
    if ok == 0 {
        return String::new();
    }
    String::from_utf8_lossy(&buf[..read as usize]).into_owned()
}

#[cfg(test)]
mod tests {
    #[test]
    fn replacement_expansion_is_bounded_across_all_arguments() {
        let template = vec!["{}".to_string(); 3000];
        assert!(super::replacement_command(&template, "{}", &"a".repeat(500)).is_err());
        assert_eq!(
            super::replacement_command(&["{}:{}".into()], "{}", "{}").unwrap(),
            ["{}:{}"]
        );
    }
}
