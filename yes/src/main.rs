mod cli;

use clap::Parser;

fn main() {
    let args = cli::Args::parse();

    let s = args.string.unwrap_or_else(|| "y".to_string());

    loop {
        println!("{}", s);
    }
}
