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

    process::exit(0);
}