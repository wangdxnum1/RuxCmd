mod cli;

use clap::Parser;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::Duration;

fn main() {
    let args = cli::Args::parse();

    if args.files.is_empty() {
        eprintln!("tail: missing file operand");
        std::process::exit(1);
    }

    let multiple_files = args.files.len() > 1;
    let show_header = |name: &str| -> bool {
        if args.verbose {
            true
        } else if args.quiet {
            false
        } else {
            multiple_files
        }
    };

    let mut exit_code = 0i32;

    for file in &args.files {
        let file_name = file.display().to_string();
        let header = show_header(&file_name);

        if let Err(e) = tail_file(file, &file_name, &args, header) {
            eprintln!("tail: {}: {}", file_name, e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

fn tail_file(path: &Path, name: &str, args: &cli::Args, show_header: bool) -> Result<(), String> {
    let mut file = fs::File::open(path).map_err(|e| format!("cannot open file: {}", e))?;

    if show_header {
        println!("==> {} <==", name);
    }

    if let Some(bytes) = args.bytes {
        let bytes = bytes.max(0) as u64;
        let file_size = file
            .metadata()
            .map_err(|e| format!("cannot stat file: {}", e))?
            .len();

        let start = if file_size > bytes {
            file_size - bytes
        } else {
            0
        };
        file.seek(SeekFrom::Start(start))
            .map_err(|e| format!("cannot seek: {}", e))?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(|e| format!("cannot read: {}", e))?;
        std::io::stdout()
            .write_all(&buf)
            .map_err(|e| format!("cannot write: {}", e))?;
    } else {
        let lines = args.lines.max(0) as usize;

        if lines == 0 {
            if args.follow {
                follow_file(file, name)?;
            }
            return Ok(());
        }

        let mut reader = BufReader::new(&file);
        let mut all_lines = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("cannot read line: {}", e))?;
            all_lines.push(line);
        }

        let start = if all_lines.len() > lines {
            all_lines.len() - lines
        } else {
            0
        };
        let mut stdout = std::io::stdout();

        for line in &all_lines[start..] {
            writeln!(stdout, "{}", line).map_err(|e| format!("cannot write: {}", e))?;
        }

        if args.follow {
            let mut file =
                fs::File::open(path).map_err(|e| format!("cannot reopen file: {}", e))?;
            file.seek(SeekFrom::End(0))
                .map_err(|e| format!("cannot seek: {}", e))?;
            follow_file(file, name)?;
        }
    }

    Ok(())
}

fn follow_file(mut file: fs::File, name: &str) -> Result<(), String> {
    let mut stdout = std::io::stdout();
    let mut buf = [0u8; 8192];
    let mut last_size = file
        .metadata()
        .map_err(|e| format!("cannot stat file: {}", e))?
        .len();

    loop {
        match file.read(&mut buf) {
            Ok(n) if n > 0 => {
                stdout
                    .write_all(&buf[..n])
                    .map_err(|e| format!("cannot write: {}", e))?;
                stdout.flush().map_err(|e| format!("cannot flush: {}", e))?;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("tail: {}: error reading: {}", name, e);
                return Ok(());
            }
        }

        std::thread::sleep(Duration::from_millis(100));

        match file.metadata() {
            Ok(meta) => {
                let current_size = meta.len();
                if current_size < last_size {
                    file.seek(SeekFrom::Start(0))
                        .map_err(|e| format!("cannot seek: {}", e))?;
                }
                last_size = current_size;
            }
            Err(e) => {
                eprintln!("tail: {}: error statting: {}", name, e);
                return Ok(());
            }
        }
    }
}
