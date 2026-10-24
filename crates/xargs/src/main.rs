mod cli;

use std::io::{self, BufRead, Read, Write};
use std::process::{Command, Stdio};

fn main() {
    let args = cli::Args::parse();

    let mut raw_input: Vec<u8> = Vec::new();
    if let Err(e) = io::stdin().read_to_end(&mut raw_input) {
        eprintln!("xargs: error reading stdin: {}", e);
        std::process::exit(1);
    }

    let input_args: Vec<String> = if args.null_delimited {
        raw_input
            .split(|&b| b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).into_owned())
            .collect()
    } else {
        let text = String::from_utf8_lossy(&raw_input);
        text.split_whitespace().map(|s| s.to_string()).collect()
    };

    // GNU xargs runs the command once even with empty input; the old
    // built-in -r silently skipped it.
    let input_args = input_args;

    let replace_str = &args.replace;
    let replace_set = args.replace_set;
    let uses_replace = args
        .command
        .iter()
        .any(|arg| arg == replace_str || arg.contains(replace_str));
    let max_args_val = args.max_args;

    let mut command_template = args.command;
    if command_template.is_empty() {
        command_template.push("echo".to_string());
    }

    let (max_args, is_replace_mode) = if replace_set || uses_replace {
        (1, true)
    } else if max_args_val == 0 {
        (input_args.len(), false)
    } else {
        (max_args_val, false)
    };

    const MAX_COMMAND_CHARS: usize = 8000;

    // GNU xargs (without -r) runs the command once even with empty input.
    if input_args.is_empty() {
        let program = &command_template[0];
        if args.verbose {
            eprintln!("{} {}", program, command_template[1..].join(" "));
        }
        let status = Command::new(program)
            .args(&command_template[1..])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
        match status {
            Ok(s) if s.success() => return,
            Ok(s) => std::process::exit(s.code().unwrap_or(1)),
            Err(e) => {
                eprintln!("xargs: {}: {}", program, e);
                std::process::exit(1);
            }
        }
    }

    let mut i = 0;
    while i < input_args.len() {
        // Respect -n but never exceed the Windows command-line length limit.
        let mut end = std::cmp::min(i + max_args, input_args.len());
        if !is_replace_mode && max_args_val == 0 {
            let mut used: usize = command_template.iter().map(|a| a.len() + 1).sum();
            end = i;
            while end < input_args.len() {
                used += input_args[end].len() + 1;
                if used > MAX_COMMAND_CHARS && end > i {
                    break;
                }
                end += 1;
            }
        }
        let batch_args = &input_args[i..end];

        let mut cmd_args: Vec<String> = Vec::new();

        if is_replace_mode {
            for arg in &command_template {
                if arg == replace_str {
                    for input_arg in batch_args {
                        cmd_args.push(input_arg.clone());
                    }
                } else if arg.contains(replace_str) {
                    let replaced = arg.replace(replace_str, "{}");
                    let mut has_placeholder = false;
                    let mut replaced_arg = String::new();
                    for part in replaced.split("{}") {
                        replaced_arg.push_str(part);
                        if !has_placeholder && !batch_args.is_empty() {
                            replaced_arg.push_str(&batch_args[0]);
                            has_placeholder = true;
                        }
                    }
                    cmd_args.push(replaced_arg);
                } else {
                    cmd_args.push(arg.clone());
                }
            }

            if !uses_replace {
                for input_arg in batch_args {
                    cmd_args.push(input_arg.clone());
                }
            }
        } else {
            cmd_args.extend_from_slice(&command_template);
            cmd_args.extend_from_slice(batch_args);
        }

        if cmd_args.is_empty() {
            cmd_args.extend_from_slice(batch_args);
        }

        let program = &cmd_args[0];
        let cmd_args_slice = if cmd_args.len() > 1 {
            &cmd_args[1..]
        } else {
            &[]
        };

        if args.verbose {
            eprintln!("{} {}", program, cmd_args_slice.join(" "));
        }

        if args.interactive {
            print!("{} {} ?...", program, cmd_args_slice.join(" "));
            io::stdout().flush().unwrap();
            let response = read_console_line();
            let response = response.trim().to_lowercase();
            if response != "y" && response != "yes" {
                i = end;
                continue;
            }
        }

        let status = Command::new(program)
            .args(cmd_args_slice)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();

        match status {
            Ok(s) => {
                if !s.success() {
                    std::process::exit(s.code().unwrap_or(1));
                }
            }
            Err(e) => {
                eprintln!("xargs: {}: {}", program, e);
                std::process::exit(1);
            }
        }

        i = end;
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
