mod cli;

use clap::Parser;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
#[cfg(windows)]
use std::os::windows::fs::symlink_file;

fn main() {
    let args = cli::Args::parse();

    if args.sources.is_empty() {
        eprintln!("cp: missing file operand");
        std::process::exit(1);
    }

    let mut exit_code = 0i32;

    for source in &args.sources {
        if let Err(e) = copy_file(source, &args.destination, &args) {
            eprintln!("cp: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

fn copy_file(source: &Path, dest: &Path, args: &cli::Args) -> Result<(), String> {
    if !source.exists() {
        return Err(format!("cannot stat '{}': No such file or directory", source.display()));
    }

    let source_metadata = fs::metadata(source).map_err(|e| format!("cannot stat '{}': {}", source.display(), e))?;

    if source_metadata.is_dir() {
        if !args.recursive {
            return Err(format!("omitting directory '{}'", source.display()));
        }
        copy_directory(source, dest, args)
    } else {
        let dest_path = if dest.exists() && dest.is_dir() {
            dest.join(source.file_name().unwrap())
        } else {
            dest.to_path_buf()
        };

        if dest_path.exists() && !args.force {
            if args.interactive {
                if !prompt_yes_no(&format!("cp: overwrite '{}'? ", dest_path.display())) {
                    return Ok(());
                }
            } else {
                return Err(format!("cannot create regular file '{}': File exists", dest_path.display()));
            }
        }

        if args.symbolic_link {
            #[cfg(windows)]
            symlink_file(source, &dest_path).map_err(|e| format!("cannot create symbolic link '{}': {}", dest_path.display(), e))?;
            #[cfg(not(windows))]
            fs::symlink_file(source, &dest_path).map_err(|e| format!("cannot create symbolic link '{}': {}", dest_path.display(), e))?;
        } else {
            fs::copy(source, &dest_path).map_err(|e| format!("cannot copy '{}' to '{}': {}", source.display(), dest_path.display(), e))?;
        }

        if args.verbose {
            println!("'{}' -> '{}'", source.display(), dest_path.display());
        }

        Ok(())
    }
}

fn copy_directory(source: &Path, dest: &Path, args: &cli::Args) -> Result<(), String> {
    let dest_path = if dest.exists() && dest.is_dir() {
        dest.join(source.file_name().unwrap())
    } else {
        dest.to_path_buf()
    };

    if !dest_path.exists() {
        fs::create_dir_all(&dest_path).map_err(|e| format!("cannot create directory '{}': {}", dest_path.display(), e))?;
    }

    let entries = fs::read_dir(source).map_err(|e| format!("cannot read directory '{}': {}", source.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("cannot read entry in '{}': {}", source.display(), e))?;
        let entry_path = entry.path();
        let dest_entry_path = dest_path.join(entry.file_name());

        let entry_metadata = fs::metadata(&entry_path).map_err(|e| format!("cannot stat '{}': {}", entry_path.display(), e))?;

        if entry_metadata.is_dir() {
            copy_directory(&entry_path, &dest_entry_path, args)?;
        } else {
            if dest_entry_path.exists() && !args.force {
                if args.interactive {
                    if !prompt_yes_no(&format!("cp: overwrite '{}'? ", dest_entry_path.display())) {
                        continue;
                    }
                } else {
                    eprintln!("cp: cannot create regular file '{}': File exists", dest_entry_path.display());
                    continue;
                }
            }

            fs::copy(&entry_path, &dest_entry_path).map_err(|e| format!("cannot copy '{}' to '{}': {}", entry_path.display(), dest_entry_path.display(), e))?;

            if args.verbose {
                println!("'{}' -> '{}'", entry_path.display(), dest_entry_path.display());
            }
        }
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