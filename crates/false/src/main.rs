mod cli;

use clap::Parser;

fn main() {
    let args = cli::Args::parse();

    // POSIX: false ignores all operands and unknown options.
    let _ = args.ignored;

    std::process::exit(1);
}
