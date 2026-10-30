use clap::Parser;
use std::io::Write;

/// Print environment variables - a Windows port of Linux printenv.
#[derive(Parser, Debug)]
#[command(name = "printenv", version, about)]
pub struct Args {
    /// End each output line with NUL, not newline
    #[arg(short = '0', long = "null")]
    pub null: bool,

    /// Variable names to look up; without names the whole environment prints
    #[arg(value_name = "NAME")]
    pub names: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let term = if args.null { b'\0' } else { b'\n' };
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let mut all_found = true;
    if args.names.is_empty() {
        for (key, value) in std::env::vars_os() {
            let _ = out.write_all(key.to_string_lossy().as_bytes());
            let _ = out.write_all(b"=");
            let _ = out.write_all(value.to_string_lossy().as_bytes());
            let _ = out.write_all(&[term]);
        }
    } else {
        for name in &args.names {
            match std::env::var_os(name) {
                Some(value) => {
                    let _ = out.write_all(value.to_string_lossy().as_bytes());
                    let _ = out.write_all(&[term]);
                }
                None => all_found = false,
            }
        }
    }
    let _ = out.flush();
    std::process::exit(if all_found { 0 } else { 1 });
}
