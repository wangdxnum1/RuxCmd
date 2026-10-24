mod cli;

use clap::Parser;
use cli::Cli;
use std::fs;
use std::path::Path;

fn main() {
    let cli = Cli::parse();

    let path = Path::new(&cli.file);
    let result = if cli.canonicalize {
        fs::canonicalize(path)
    } else {
        fs::read_link(path)
    };

    match result {
        Ok(target) => {
            let target_str = if cli.canonicalize {
                strip_verbatim(&target)
            } else {
                match target.to_str() {
                    Some(s) => s.to_string(),
                    None => {
                        eprintln!("readlink: cannot convert target path to string");
                        std::process::exit(1);
                    }
                }
            };
            if cli.no_newline {
                print!("{}", target_str);
            } else {
                println!("{}", target_str);
            }
        }
        Err(e) => {
            eprintln!("readlink: {}: {}", cli.file, e);
            std::process::exit(1);
        }
    }
}

/// Strip the `\\?\` verbatim prefix (mapping `\\?\UNC\` to `//`) and normalize
/// backslashes to forward slashes so the output is directly reusable.
fn strip_verbatim(path: &Path) -> String {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        return format!("//{}", rest.replace('\\', "/"));
    }
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s.replace('\\', "/")
}
