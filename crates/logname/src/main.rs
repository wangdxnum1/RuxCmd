mod cli;

use clap::Parser;
use windows_sys::Win32::System::WindowsProgramming::GetUserNameW;

fn main() {
    let args = cli::Args::parse();

    if let Some(extra) = args.operands.first() {
        eprintln!("logname: extra operand '{}'", extra);
        std::process::exit(1);
    }

    match login_name() {
        Some(name) => println!("{}", name),
        None => {
            eprintln!("logname: no login name");
            std::process::exit(1);
        }
    }
}

/// The owning account of the current process token via GetUserNameW. When the
/// API call fails, USERNAME from the environment is used as a fallback so the
/// command still works in shells where advapi32 lookup fails.
fn login_name() -> Option<String> {
    if let Some(name) = get_user_name() {
        return Some(name);
    }
    match std::env::var("USERNAME") {
        Ok(name) if !name.is_empty() => Some(name),
        _ => None,
    }
}

/// SAFETY: the buffer out-parameter points at valid storage for the whole
/// call, and the size in/out parameter is initialized.
fn get_user_name() -> Option<String> {
    // UNLEN is 256; one extra slot for the terminating null.
    let mut buffer = [0u16; 257];
    let mut size: u32 = buffer.len() as u32;

    let ok = unsafe { GetUserNameW(buffer.as_mut_ptr(), &mut size) };
    if ok == 0 || size == 0 {
        return None;
    }

    // On success size counts the terminating null as well.
    let len = (size as usize - 1).min(buffer.len());
    let name = String::from_utf16_lossy(&buffer[..len]);
    if name.is_empty() { None } else { Some(name) }
}
