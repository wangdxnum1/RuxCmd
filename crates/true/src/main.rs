mod cli;

use clap::Parser;
use cli::Cli;
use std::process;

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("true 1.0.0");
        return;
    }

    // POSIX: true ignores all operands and unknown options.
    let _ = cli.ignored;

    process::exit(0);
}
