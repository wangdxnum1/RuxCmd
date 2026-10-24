mod cli;

use clap::Parser;
use std::fs;
use std::process;

fn main() {
    let args = cli::Args::parse();

    match eval(&args.operand) {
        Ok(true) => process::exit(0),
        Ok(false) => process::exit(1),
        Err(msg) => {
            eprintln!("test: {}", msg);
            process::exit(2);
        }
    }
}

/// Evaluate the expression following the POSIX test grammar subset:
/// single argument (non-empty string), unary operators, binary operators,
/// and '!' negation (recursively).
fn eval(args: &[String]) -> Result<bool, String> {
    match args.len() {
        0 => Ok(false),
        1 => Ok(!args[0].is_empty()),
        2 => {
            if args[0] == "!" {
                return eval(&args[1..]).map(|b| !b);
            }
            if is_unary(&args[0]) {
                return Ok(unary(&args[0], &args[1]));
            }
            Err(format!("unary operator expected: '{}'", args[0]))
        }
        _ => {
            if args[0] == "!" {
                return eval(&args[1..]).map(|b| !b);
            }
            if is_binary(&args[1]) {
                return binary(&args[0], &args[1], &args[2]);
            }
            Err(format!("expected operator: '{}'", args[1]))
        }
    }
}

fn is_unary(op: &str) -> bool {
    matches!(
        op,
        "-e" | "-f" | "-d" | "-s" | "-z" | "-n" | "-r" | "-w" | "-x"
    )
}

fn is_binary(op: &str) -> bool {
    matches!(
        op,
        "=" | "!=" | "-eq" | "-ne" | "-lt" | "-le" | "-gt" | "-ge"
    )
}

fn unary(op: &str, arg: &str) -> bool {
    match op {
        "-z" => arg.is_empty(),
        "-n" => !arg.is_empty(),
        _ => file_test(op, arg),
    }
}

fn file_test(op: &str, path: &str) -> bool {
    match fs::metadata(path) {
        Ok(meta) => match op {
            "-e" => true,
            "-f" => meta.is_file(),
            "-d" => meta.is_dir(),
            "-s" => meta.len() > 0,
            // Windows approximations: everything existing is readable and
            // "executable"; -w honors the readonly bit.
            "-r" => true,
            "-w" => !meta.permissions().readonly(),
            "-x" => true,
            _ => false,
        },
        Err(_) => false,
    }
}

fn binary(a: &str, op: &str, b: &str) -> Result<bool, String> {
    match op {
        "=" => Ok(a == b),
        "!=" => Ok(a != b),
        _ => {
            let x = parse_int(a).map_err(|_| format!("invalid integer '{}'", a))?;
            let y = parse_int(b).map_err(|_| format!("invalid integer '{}'", b))?;
            Ok(match op {
                "-eq" => x == y,
                "-ne" => x != y,
                "-lt" => x < y,
                "-le" => x <= y,
                "-gt" => x > y,
                _ => x >= y,
            })
        }
    }
}

fn parse_int(s: &str) -> Result<i64, ()> {
    s.trim().parse::<i64>().map_err(|_| ())
}
