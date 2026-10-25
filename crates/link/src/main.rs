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
            eprintln!("link: missing file operand");
            std::process::exit(1);
        }
        1 => {
            eprintln!(
                "link: missing operand after '{}'",
                args.operands[0].display()
            );
            std::process::exit(1);
        }
        n if n > 2 => {
            eprintln!("link: extra operand '{}'", args.operands[2].display());
            std::process::exit(1);
        }
        _ => {}
    }

    let file1: &PathBuf = &args.operands[0];
    let file2: &PathBuf = &args.operands[1];

    if let Err(e) = fs::hard_link(file1, file2) {
        eprintln!(
            "link: cannot create link '{}' to '{}': {}",
            file2.display(),
            file1.display(),
            e
        );
        std::process::exit(1);
    }
}
