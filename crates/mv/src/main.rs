mod cli;

use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    if args.sources.is_empty() {
        eprintln!("mv: missing file operand");
        std::process::exit(1);
    }

    let mut exit_code = 0i32;

    for source in &args.sources {
        if let Err(e) = move_file(source, &args.destination, &args) {
            eprintln!("mv: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

fn move_file(source: &Path, dest: &Path, args: &cli::Args) -> Result<(), String> {
    if !source.exists() {
        return Err(format!(
            "cannot stat '{}': No such file or directory",
            source.display()
        ));
    }

    let dest_path = if dest.exists() && dest.is_dir() {
        dest.join(source.file_name().unwrap())
    } else {
        dest.to_path_buf()
    };

    if dest_path.exists() {
        if args.no_clobber {
            return Err(format!(
                "cannot move '{}' to '{}': File exists",
                source.display(),
                dest_path.display()
            ));
        }

        if !args.force {
            if args.interactive {
                if !prompt_yes_no(&format!("mv: overwrite '{}'? ", dest_path.display())) {
                    return Ok(());
                }
            } else {
                return Err(format!(
                    "cannot move '{}' to '{}': File exists",
                    source.display(),
                    dest_path.display()
                ));
            }
        }

        fs::remove_file(&dest_path)
            .or_else(|_| fs::remove_dir_all(&dest_path))
            .map_err(|e| format!("cannot remove '{}': {}", dest_path.display(), e))?;
    }

    fs::rename(source, &dest_path).map_err(|e| {
        format!(
            "cannot move '{}' to '{}': {}",
            source.display(),
            dest_path.display(),
            e
        )
    })?;

    if args.verbose {
        println!("'{}' -> '{}'", source.display(), dest_path.display());
    }

    Ok(())
}

fn prompt_yes_no(prompt: &str) -> bool {
    print!("{}", prompt);
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    let input = input.trim().to_lowercase();
    input == "y" || input == "yes"
}
