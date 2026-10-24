mod cli;

use clap::Parser;
use cli::Count;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Interval between follow-mode polls of the watched files.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

fn main() {
    let args = cli::Args::parse();

    // `-c N -f` follows like the line mode does: the initial output covers
    // the requested byte region, then appended bytes are streamed (pre-fix
    // the follow flag was silently ignored in the byte branch). Standard
    // input still cannot be followed on this build.
    let byte_mode = args.bytes.is_some();
    let count = if byte_mode {
        Count::parse(args.bytes.as_deref().unwrap_or_default(), 'c')
    } else {
        Count::parse(&args.lines, 'n')
    };

    if args.files.is_empty() {
        if args.follow {
            eprintln!("tail: missing file operand");
            std::process::exit(1);
        }
        // P0 #23: no operand means standard input, like other filters.
        if let Err(e) = tail_stdin(count, byte_mode, false) {
            eprintln!("tail: {}", e);
            std::process::exit(1);
        }
        std::process::exit(0);
    }

    let multiple_files = args.files.len() > 1;
    let headers = args.verbose || (!args.quiet && multiple_files);

    if args.follow {
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
                if let Err(e) = tail_follow_single(&targets[0], count, byte_mode, args.verbose) {
                    eprintln!("tail: {}: {}", targets[0].display(), e);
                    exit_code = 1;
                }
                std::process::exit(exit_code);
            }
            _ => {
                let show_headers = args.verbose || (!args.quiet && targets.len() > 1);
                follow_multi(&targets, count, byte_mode, show_headers)
            }
        }
    }

    // Without -f, operands are processed in order; `-` reads stdin.
    let mut exit_code = 0i32;
    for file in &args.files {
        if file.as_os_str() == OsStr::new("-") {
            if let Err(e) = tail_stdin(count, byte_mode, headers) {
                eprintln!("tail: {}", e);
                exit_code = 1;
            }
        } else {
            let name = file.display().to_string();
            if let Err(e) = tail_file(file, &name, count, byte_mode, headers) {
                eprintln!("tail: {}: {}", name, e);
                exit_code = 1;
            }
        }
    }

    std::process::exit(exit_code);
}

/// Non-follow output of one file operand: the region selected by `count`
/// (last N lines/bytes, or from +N onward).
fn tail_file(
    path: &Path,
    name: &str,
    count: Count,
    byte_mode: bool,
    show_header: bool,
) -> Result<(), String> {
    let mut file = fs::File::open(path).map_err(|e| format!("cannot open file: {}", e))?;

    if show_header {
        println!("==> {} <==", name);
    }

    if byte_mode {
        let file_size = file
            .metadata()
            .map_err(|e| format!("cannot stat file: {}", e))?
            .len();
        file.seek(SeekFrom::Start(byte_start(file_size, count)))
            .map_err(|e| format!("cannot seek: {}", e))?;
        let mut stdout = io::stdout();
        io::copy(&mut file, &mut stdout).map_err(|e| format!("cannot write: {}", e))?;
        stdout.flush().map_err(|e| format!("cannot flush: {}", e))?;
    } else {
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|e| format!("cannot read: {}", e))?;
        write_line_region(&data, count)?;
    }

    Ok(())
}

/// P0 #23: `tail` with no file operand (or a `-` operand) reads standard
/// input; following standard input is not supported.
fn tail_stdin(count: Count, byte_mode: bool, show_header: bool) -> Result<(), String> {
    if show_header {
        println!("==> standard input <==");
    }

    let mut data = Vec::new();
    io::stdin()
        .lock()
        .read_to_end(&mut data)
        .map_err(|e| format!("cannot read standard input: {}", e))?;

    if byte_mode {
        let start = byte_start(data.len() as u64, count) as usize;
        io::stdout()
            .write_all(&data[start..])
            .map_err(|e| format!("cannot write: {}", e))?;
    } else {
        write_line_region(&data, count)?;
    }

    Ok(())
}

// --- byte-transparent region selection ---------------------------------------

/// Start offset of the region `-c` outputs: the last N bytes, or from byte
/// N onward for `+N` (1-indexed; N <= 1 is the whole input).
fn byte_start(len: u64, count: Count) -> u64 {
    match count {
        Count::Last(n) => len.saturating_sub(n as u64),
        Count::From(n) => (n as u64).saturating_sub(1).min(len),
    }
}

/// Start offset of the region the line mode outputs within `data`: the last
/// N lines, or from line N onward for `+N` (1-indexed).
fn line_start(data: &[u8], count: Count) -> usize {
    match count {
        Count::Last(0) => data.len(),
        Count::Last(n) => start_after_lines(data, line_count(data).saturating_sub(n)),
        Count::From(n) => start_after_lines(data, n.saturating_sub(1)),
    }
}

/// Number of lines in `data`: newline-terminated lines plus a final
/// unterminated one.
fn line_count(data: &[u8]) -> usize {
    let mut lines = data.iter().filter(|&&b| b == b'\n').count();
    if !data.is_empty() && *data.last().unwrap() != b'\n' {
        lines += 1;
    }
    lines
}

/// Offset just past the `skip`-th newline in `data` (the start of line
/// skip+1); `data.len()` when the data has fewer newlines than `skip`.
fn start_after_lines(data: &[u8], skip: usize) -> usize {
    if skip == 0 {
        return 0;
    }
    let mut seen = 0usize;
    for (i, &b) in data.iter().enumerate() {
        if b == b'\n' {
            seen += 1;
            if seen == skip {
                return i + 1;
            }
        }
    }
    data.len()
}

/// Writes the selected lines of `data`, byte-transparent: terminators are
/// echoed as-is, a final line without a newline is not padded, and invalid
/// UTF-8 passes through (the pre-fix `lines()` + `writeln!` rewrote all
/// three and did not support `+N`).
fn write_line_region(data: &[u8], count: Count) -> Result<(), String> {
    let start = line_start(data, count);
    io::stdout()
        .write_all(&data[start..])
        .map_err(|e| format!("cannot write: {}", e))
}

/// Initial-output plan for follow mode: every non-empty request (`-n N` with
/// N > 0, `-n +N`, any `-c`) prints its initial region first, then following
/// resumes at end-of-file so previously read content is never replayed.
/// (Pre-fix, `-n 0 -f` resumed at offset 0 and dumped the whole file —
/// P0 #23.)
fn follow_plan(file_len: u64, count: Count) -> (bool, u64) {
    match count {
        Count::Last(0) => (false, file_len),
        _ => (true, file_len),
    }
}

/// Follows a single file: initial tail output, then streams appended bytes.
fn tail_follow_single(
    path: &Path,
    count: Count,
    byte_mode: bool,
    show_header: bool,
) -> Result<(), String> {
    let name = path.display().to_string();
    let mut file = fs::File::open(path).map_err(|e| format!("cannot open file: {}", e))?;
    let file_len = file
        .metadata()
        .map_err(|e| format!("cannot stat file: {}", e))?
        .len();
    let (print_initial, follow_from) = follow_plan(file_len, count);

    if show_header {
        println!("==> {} <==", name);
    }
    if print_initial {
        write_initial_region(&mut file, file_len, count, byte_mode)?;
    }
    file.seek(SeekFrom::Start(follow_from))
        .map_err(|e| format!("cannot seek: {}", e))?;
    follow_file(file, &name)
}

/// Writes the pre-follow region (last N or from +N) of `file`: byte mode
/// seeks straight to the start offset, line mode reads the content to find
/// the line boundaries.
fn write_initial_region(
    file: &mut fs::File,
    file_len: u64,
    count: Count,
    byte_mode: bool,
) -> Result<(), String> {
    if byte_mode {
        file.seek(SeekFrom::Start(byte_start(file_len, count)))
            .map_err(|e| format!("cannot seek: {}", e))?;
        let mut stdout = io::stdout();
        io::copy(file, &mut stdout).map_err(|e| format!("cannot write: {}", e))?;
        stdout.flush().map_err(|e| format!("cannot flush: {}", e))?;
    } else {
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|e| format!("cannot read: {}", e))?;
        write_line_region(&data, count)?;
    }
    Ok(())
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
fn follow_multi(paths: &[PathBuf], count: Count, byte_mode: bool, show_headers: bool) -> ! {
    let mut headers = HeaderPrinter::new(show_headers);
    let mut targets: Vec<FollowTarget> = Vec::new();

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
        let file_len = match file.metadata() {
            Ok(meta) => meta.len(),
            Err(e) => {
                eprintln!("tail: {}: cannot stat file: {}", name, e);
                continue;
            }
        };
        let (print_initial, follow_from) = follow_plan(file_len, count);
        if let Some(header) = headers.ensure(targets.len(), &name) {
            println!("{}", header);
        }
        if print_initial && let Err(e) = write_initial_region(&mut file, file_len, count, byte_mode)
        {
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
        let (print_initial, follow_from) = follow_plan(6, Count::Last(0));
        assert!(!print_initial);
        assert_eq!(follow_from, 6);
    }

    #[test]
    fn follow_plan_n_positive_prints_tail_then_resumes_at_eof() {
        let (print_initial, follow_from) = follow_plan(6, Count::Last(2));
        assert!(print_initial);
        assert_eq!(follow_from, 6);
    }

    #[test]
    fn follow_plan_plus_n_prints_from_n_then_resumes_at_eof() {
        let (print_initial, follow_from) = follow_plan(6, Count::From(2));
        assert!(print_initial);
        assert_eq!(follow_from, 6);
    }

    #[test]
    fn byte_start_last_and_from() {
        assert_eq!(byte_start(6, Count::Last(2)), 4);
        assert_eq!(byte_start(3, Count::Last(9)), 0);
        assert_eq!(byte_start(6, Count::Last(0)), 6);
        assert_eq!(byte_start(6, Count::From(3)), 2);
        assert_eq!(byte_start(6, Count::From(1)), 0);
        assert_eq!(byte_start(6, Count::From(0)), 0);
        assert_eq!(byte_start(6, Count::From(9)), 6);
    }

    #[test]
    fn line_start_last_n() {
        assert_eq!(line_start(b"a\nb\nc\n", Count::Last(2)), 2);
        // No final newline: the last line is still a line.
        assert_eq!(line_start(b"a\nb\nc", Count::Last(2)), 2);
        assert_eq!(line_start(b"a\nb\n", Count::Last(9)), 0);
        assert_eq!(line_start(b"a\nb\n", Count::Last(0)), 4);
        assert_eq!(line_start(b"", Count::Last(1)), 0);
    }

    #[test]
    fn line_start_from_n() {
        assert_eq!(line_start(b"a\nb\nc\n", Count::From(2)), 2);
        assert_eq!(line_start(b"a\nb\nc\n", Count::From(1)), 0);
        assert_eq!(line_start(b"a\nb\nc\n", Count::From(0)), 0);
        // From a line beyond the end prints nothing.
        assert_eq!(line_start(b"a\nb\nc\n", Count::From(9)), 6);
        assert_eq!(line_start(b"a\nb\nc", Count::From(3)), 4);
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
