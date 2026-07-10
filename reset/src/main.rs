mod cli;

use clap::Parser;
use std::io::{self, Write};

fn main() {
    let _args = cli::Args::parse();

    #[cfg(windows)]
    {
        use std::ptr;
        use winapi::um::consoleapi::SetConsoleMode;
        use winapi::um::processenv::GetStdHandle;
        use winapi::um::winbase::STD_INPUT_HANDLE;

        const ENABLE_ECHO_INPUT: u32 = 0x0004;
        const ENABLE_LINE_INPUT: u32 = 0x0002;
        const ENABLE_PROCESSED_INPUT: u32 = 0x0001;

        unsafe {
            let h_stdin = GetStdHandle(STD_INPUT_HANDLE);
            if h_stdin != ptr::null_mut() {
                let default_mode = ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT;
                SetConsoleMode(h_stdin, default_mode);
            }
        }
    }

    print!("\x1b[H\x1b[2J\x1b[3J");
    io::stdout().flush().unwrap();
}