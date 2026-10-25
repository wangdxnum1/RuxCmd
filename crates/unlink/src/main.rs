mod cli;

use clap::Parser;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    // GNU validates the operand count before touching the filesystem and
    // exits 1 (not clap's 2) for every operand error.
    match args.operands.len() {
        0 => {
            eprintln!("unlink: missing operand");
            std::process::exit(1);
        }
        n if n > 1 => {
            eprintln!("unlink: extra operand '{}'", args.operands[1].display());
            std::process::exit(1);
        }
        _ => {}
    }

    let file: &PathBuf = &args.operands[0];

    if let Err(e) = fs::remove_file(file) {
        eprintln!("unlink: cannot unlink '{}': {}", file.display(), e);
        std::process::exit(1);
    }
}
