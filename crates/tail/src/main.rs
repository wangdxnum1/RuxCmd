mod cli;

use clap::Parser;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Interval between follow-mode polls of the watched files.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

fn main() {
    let args = cli::Args::parse();

    // `-f` only applies to real files: `-c ... -f` keeps the historical
    // behavior of ignoring the follow flag, and standard input cannot be
    // followed on this build.
    let follow = args.follow && args.bytes.is_none();

    if args.files.is_empty() {
        if follow {
            eprintln!("tail: missing file operand");
            std::process::exit(1);
        }
        // P0 #23: no operand means standard input, like other filters.
        if let Err(e) = tail_stdin(&args, false) {
            eprintln!("tail: {}", e);
            std::process::exit(1);
        }
        std::process::exit(0);
    }

    let multiple_files = args.files.len() > 1;
    let headers = args.verbose || (!args.quiet && multiple_files);

    if follow {
        // P0 #23: following several files must poll all of them from one
        // loop instead of blocking on the first file forever.
        let mut targets: Vec<PathBuf> = Vec::new();
        let mut exit_code = 0i32;
        for file in &args.files {
            if file.as_os_str() == OsStr::new("-") {
                eprintln!("tail: cannot follow '-' (standard input)");
                exit_code = 1;
            } else {
                targets.push(file.clone());
            }
        }
        match targets.len() {
            0 => std::process::exit(exit_code),
            1 => {
                if let Err(e) = tail_follow_single(&targets[0], &args, args.verbose) {
                    eprintln!("tail: {}: {}", targets[0].display(), e);
                    exit_code = 1;
                }
                std::process::exit(exit_code);
            }
            _ => follow_multi(&targets, &args),
        }
    }

    // Without -f, operands are processed in order; `-` reads stdin.
    let mut exit_code = 0i32;
    for file in &args.files {
        if file.as_os_str() == OsStr::new("-") {
            if let Err(e) = tail_stdin(&args, headers) {
                eprintln!("tail: {}", e);
                exit_code = 1;
            }
        } else {
            let name = file.display().to_string();
            if let Err(e) = tail_file(file, &name, &args, headers) {
                eprintln!("tail: {}: {}", name, e);
                exit_code = 1;
            }
        }
    }

    std::process::exit(exit_code);
}

/// Non-follow output of one file operand: the last N lines or N bytes.
/// Line semantics (UTF-8 `lines()` + `writeln!`) are a tracked P1 item
/// and intentionally unchanged.
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
        let start = file_size.saturating_sub(bytes);
        file.seek(SeekFrom::Start(start))
            .map_err(|e| format!("cannot seek: {}", e))?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(|e| format!("cannot read: {}", e))?;
        io::stdout()
            .write_all(&buf)
            .map_err(|e| format!("cannot write: {}", e))?;
    } else {
        let lines = args.lines.max(0) as usize;
        if lines > 0 {
            print_last_lines(BufReader::new(&mut file), lines)?;
        }
    }

    Ok(())
}

/// P0 #23: `tail` with no file operand (or a `-` operand) reads standard
/// input; following standard input is not supported.
fn tail_stdin(args: &cli::Args, show_header: bool) -> Result<(), String> {
    if show_header {
        println!("==> standard input <==");
    }

    if let Some(bytes) = args.bytes {
        let bytes = bytes.max(0) as usize;
        let mut data = Vec::new();
        io::stdin()
            .lock()
            .read_to_end(&mut data)
            .map_err(|e| format!("cannot read standard input: {}", e))?;
        let start = data.len().saturating_sub(bytes);
        io::stdout()
            .write_all(&data[start..])
            .map_err(|e| format!("cannot write: {}", e))?;
    } else {
        let lines = args.lines.max(0) as usize;
        if lines > 0 {
            print_last_lines(BufReader::new(io::stdin().lock()), lines)?;
        }
    }

    Ok(())
}

/// Prints the last `lines` lines using the historical line semantics
/// (UTF-8 `lines()` + `writeln!`; CRLF/encoding handling is P1, unchanged).
fn print_last_lines(reader: impl BufRead, lines: usize) -> Result<(), String> {
    let mut all_lines = Vec::new();
    for line in reader.lines() {
        all_lines.push(line.map_err(|e| format!("cannot read line: {}", e))?);
    }
    let start = all_lines.len().saturating_sub(lines);
    let mut stdout = io::stdout();
    for line in &all_lines[start..] {
        writeln!(stdout, "{}", line).map_err(|e| format!("cannot write: {}", e))?;
    }
    Ok(())
}

/// Initial-output plan for follow mode: `tail -n N -f` prints the last N
/// lines first (nothing at all when N is 0), then resumes following at
/// end-of-file so previously read content is never replayed. (Pre-fix,
/// `-n 0 -f` resumed at offset 0 and dumped the whole file — P0 #23.)
fn follow_plan(file_len: u64, n_lines: usize) -> (bool, u64) {
    (n_lines > 0, file_len)
}

/// Follows a single file: initial tail output, then streams appended bytes.
fn tail_follow_single(path: &Path, args: &cli::Args, show_header: bool) -> Result<(), String> {
    let name = path.display().to_string();
    let mut file = fs::File::open(path).map_err(|e| format!("cannot open file: {}", e))?;
    let file_len = file
        .metadata()
        .map_err(|e| format!("cannot stat file: {}", e))?
        .len();
    let n_lines = args.lines.max(0) as usize;
    let (print_initial, follow_from) = follow_plan(file_len, n_lines);

    if show_header {
        println!("==> {} <==", name);
    }
    if print_initial {
        print_last_lines(BufReader::new(&mut file), n_lines)?;
    }
    file.seek(SeekFrom::Start(follow_from))
        .map_err(|e| format!("cannot seek: {}", e))?;
    follow_file(file, &name)
}

/// One watched file in the multi-file follow loop.
struct FollowTarget {
    name: String,
    file: fs::File,
    /// Offset of the next unread byte; starts at end-of-file so only
    /// newly appended data is emitted.
    offset: u64,
    /// Set once the file stops being readable; dropped from polling.
    failed: bool,
}

/// GNU-style `==> name <==` headers: the first header ever printed has no
/// leading blank line, later ones are preceded by a blank line, and a
/// header is reprinted only when the displayed file changes.
struct HeaderPrinter {
    enabled: bool,
    current: Option<usize>,
}

impl HeaderPrinter {
    fn new(enabled: bool) -> Self {
        HeaderPrinter {
            enabled,
            current: None,
        }
    }

    /// Header text for `name`, or `None` when headers are suppressed or
    /// the file is already the one being displayed.
    fn ensure(&mut self, index: usize, name: &str) -> Option<String> {
        if !self.enabled || self.current == Some(index) {
            return None;
        }
        let header = if self.current.is_some() {
            format!("\n==> {} <==", name)
        } else {
            format!("==> {} <==", name)
        };
        self.current = Some(index);
        Some(header)
    }
}

/// P0 #23: multi-file follow polls every operand from a single loop, so
/// growth on any file is emitted (previously the first file's follow loop
/// never returned and starved the remaining operands). Never returns.
fn follow_multi(paths: &[PathBuf], args: &cli::Args) -> ! {
    let show_headers = args.verbose || (!args.quiet && paths.len() > 1);
    let mut headers = HeaderPrinter::new(show_headers);
    let mut targets: Vec<FollowTarget> = Vec::new();
    let n_lines = args.lines.max(0) as usize;

    // First frame: header plus initial tail output for every operand,
    // then watch from each file's end-of-file.
    for path in paths {
        let name = path.display().to_string();
        let mut file = match fs::File::open(path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("tail: {}: cannot open file: {}", name, e);
                continue;
            }
        };
        let (print_initial, follow_from) = match file.metadata() {
            Ok(meta) => follow_plan(meta.len(), n_lines),
            Err(e) => {
                eprintln!("tail: {}: cannot stat file: {}", name, e);
                continue;
            }
        };
        if let Some(header) = headers.ensure(targets.len(), &name) {
            println!("{}", header);
        }
        if print_initial && let Err(e) = print_last_lines(BufReader::new(&mut file), n_lines) {
            eprintln!("tail: {}: {}", name, e);
        }
        if let Err(e) = file.seek(SeekFrom::Start(follow_from)) {
            eprintln!("tail: {}: cannot seek: {}", name, e);
            continue;
        }
        targets.push(FollowTarget {
            name,
            file,
            offset: follow_from,
            failed: false,
        });
    }

    if targets.is_empty() {
        std::process::exit(1);
    }

    let mut stdout = io::stdout();
    let mut appended = Vec::new();
    loop {
        for (i, target) in targets.iter_mut().enumerate() {
            if target.failed {
                continue;
            }
            let size = match target.file.metadata() {
                Ok(meta) => meta.len(),
                Err(e) => {
                    eprintln!("tail: {}: cannot stat file: {}", target.name, e);
                    target.failed = true;
                    continue;
                }
            };
            if size < target.offset {
                // Truncated (e.g. log rotation): rewind like single-file follow.
                if let Err(e) = target.file.seek(SeekFrom::Start(0)) {
                    eprintln!("tail: {}: cannot seek: {}", target.name, e);
                    target.failed = true;
                    continue;
                }
                target.offset = 0;
            }
            if size <= target.offset {
                continue;
            }
            appended.clear();
            if let Err(e) = target.file.read_to_end(&mut appended) {
                eprintln!("tail: {}: cannot read: {}", target.name, e);
                target.failed = true;
                continue;
            }
            if let Some(header) = headers.ensure(i, &target.name) {
                println!("{}", header);
            }
            if let Err(e) = stdout.write_all(&appended) {
                eprintln!("tail: {}: cannot write: {}", target.name, e);
                target.failed = true;
                continue;
            }
            if let Err(e) = stdout.flush() {
                eprintln!("tail: {}: cannot flush: {}", target.name, e);
            }
            target.offset += appended.len() as u64;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

fn follow_file(mut file: fs::File, name: &str) -> Result<(), String> {
    let mut stdout = io::stdout();
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

        std::thread::sleep(POLL_INTERVAL);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn follow_plan_n0_starts_at_eof_without_initial_output() {
        // `tail -n 0 -f` must not replay the file (P0 #23): the pre-fix
        // code resumed following at offset 0.
        let (print_initial, follow_from) = follow_plan(6, 0);
        assert!(!print_initial);
        assert_eq!(follow_from, 6);
    }

    #[test]
    fn follow_plan_n_positive_prints_tail_then_resumes_at_eof() {
        let (print_initial, follow_from) = follow_plan(6, 2);
        assert!(print_initial);
        assert_eq!(follow_from, 6);
    }

    #[test]
    fn header_repeats_only_on_file_switch() {
        let mut headers = HeaderPrinter::new(true);
        assert_eq!(headers.ensure(0, "a").unwrap(), "==> a <==");
        // Later headers are blank-line prefixed (GNU style).
        assert_eq!(headers.ensure(1, "b").unwrap(), "\n==> b <==");
        // The same file keeps growing: no repeated header.
        assert_eq!(headers.ensure(1, "b"), None);
        // Switching back reprints the header.
        assert_eq!(headers.ensure(0, "a").unwrap(), "\n==> a <==");
    }

    #[test]
    fn header_printer_respects_quiet() {
        let mut headers = HeaderPrinter::new(false);
        assert_eq!(headers.ensure(0, "a"), None);
    }
}
