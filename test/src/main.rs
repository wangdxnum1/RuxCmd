mod cli;

use clap::Parser;
use std::fs;
use std::process;

fn main() {
    let args = cli::Args::parse();

    let result = if let Some(path) = args.exists {
        fs::metadata(&path).is_ok()
    } else if let Some(path) = args.file {
        match fs::metadata(&path) {
            Ok(m) => m.is_file(),
            Err(_) => false,
        }
    } else if let Some(path) = args.dir {
        match fs::metadata(&path) {
            Ok(m) => m.is_dir(),
            Err(_) => false,
        }
    } else if let Some(path) = args.size {
        match fs::metadata(&path) {
            Ok(m) => m.len() > 0,
            Err(_) => false,
        }
    } else if args.operand.len() == 3 {
        let (s1, op, s2) = (&args.operand[0], &args.operand[1], &args.operand[2]);
        match op.as_str() {
            "=" => s1 == s2,
            "!=" => s1 != s2,
            _ => {
                println!("test: invalid operator '{}'", op);
                process::exit(2);
            }
        }
    } else {
        println!("test: missing argument");
        process::exit(2);
    };

    process::exit(if result { 0 } else { 1 });
}
