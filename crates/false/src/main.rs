mod cli;

use clap::Parser;

fn main() {
    let _args = cli::Args::parse();
    std::process::exit(1);
}
