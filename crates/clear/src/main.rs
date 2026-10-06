mod cli;

use clap::Parser;

fn main() {
    let _args = cli::Args::parse();

    print!("\x1B[2J\x1B[H");
}
