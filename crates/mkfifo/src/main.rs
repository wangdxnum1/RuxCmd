mod cli;

use clap::Parser;
use std::path::Path;
use winapi::shared::ntdef::HANDLE;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::winbase::{
    CreateNamedPipeA, FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX, PIPE_READMODE_BYTE,
    PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES,
};

const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;

fn main() {
    let args = cli::Args::parse();

    if args.pipes.is_empty() {
        eprintln!("mkfifo: missing operand");
        std::process::exit(1);
    }

    if args.mode.is_some() {
        eprintln!("mkfifo: mode option is not supported on Windows");
    }

    let mut exit_code = 0i32;

    for pipe in &args.pipes {
        if let Err(e) = create_named_pipe(pipe) {
            eprintln!("mkfifo: cannot create fifo '{}': {}", pipe.display(), e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

fn create_named_pipe(pipe: &Path) -> Result<(), String> {
    let pipe_name = format!(
        "\\\\.\\pipe\\{}",
        pipe.file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| "invalid pipe name".to_string())?
    );

    let handle = unsafe {
        CreateNamedPipeA(
            pipe_name.as_ptr() as *const i8,
            PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE,
            PIPE_UNLIMITED_INSTANCES,
            4096,
            4096,
            0,
            std::ptr::null_mut(),
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        let error_code = unsafe { GetLastError() };
        if error_code == 5 {
            return Err("permission denied".to_string());
        }
        return Err(format!("CreateNamedPipe failed with error {}", error_code));
    }

    unsafe {
        CloseHandle(handle);
    }

    Ok(())
}
