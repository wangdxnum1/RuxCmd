mod cli;

use clap::Parser;
use cli::Args;
use std::fs::{self, File};
use std::io::{self, Seek, Write};

fn main() {
    let args = Args::parse();

    let mut failed = false;
    for path in &args.files {
        if !shred_file(path, args.iterations, args.verbose, args.remove) {
            failed = true;
        }
    }
    if failed {
        std::process::exit(1)
    }
}

fn shred_file(path: &std::path::Path, iterations: usize, verbose: bool, remove: bool) -> bool {
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("shred: cannot open '{}': {}", path.display(), e);
            return false;
        }
    };

    let size = metadata.len();
    if size == 0 {
        if verbose {
            println!("shred: {} is empty", path.display());
        }
        if remove {
            remove_file_quiet(path);
        }
        return true;
    }

    let mut file = match File::options().write(true).open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("shred: cannot open '{}' for writing: {}", path.display(), e);
            return false;
        }
    };

    for i in 0..iterations {
        file.seek(io::SeekFrom::Start(0)).unwrap();

        if i == iterations - 1 {
            let zeros = vec![0u8; size as usize];
            file.write_all(&zeros).unwrap();
        } else {
            let pattern: u8 = ((i * 0x9E3779B9) & 0xFF) as u8;
            let data = vec![pattern; size as usize];
            file.write_all(&data).unwrap();
        }
    }

    file.sync_all().unwrap();

    if verbose {
        println!("shred: {} overwritten {} times", path.display(), iterations);
    }

    if remove {
        remove_file_quiet(path);
        if verbose {
            println!("shred: {} removed", path.display());
        }
    }
    true
}

fn remove_file_quiet(path: &std::path::Path) {
    let _ = fs::remove_file(path);
}
