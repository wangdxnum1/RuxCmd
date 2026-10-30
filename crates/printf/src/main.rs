mod cli;

use clap::Parser;
use printf::{run_format, scan_has_conversion};
use std::io::Write;

fn main() {
    let args = cli::Args::parse();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let mut had_error = false;
    let mut stop = false;
    let has_conversions = scan_has_conversion(&args.format);
    let mut arg_index = 0usize;

    loop {
        let (consumed, error) = run_format(
            &mut out,
            &args.format,
            &args.args[arg_index.min(args.args.len())..],
            &mut stop,
        );
        arg_index += consumed;
        if error {
            had_error = true;
            break;
        }
        if stop {
            break;
        }
        if arg_index >= args.args.len() {
            break;
        }
        if !has_conversions {
            eprintln!(
                "printf: warning: ignoring excess arguments, starting with '{}'",
                args.args[arg_index]
            );
            break;
        }
    }

    if let Err(e) = out.flush() {
        eprintln!("printf: write error: {}", e);
        std::process::exit(1);
    }
    if had_error {
        std::process::exit(1);
    }
}
