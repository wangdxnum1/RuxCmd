use clap::Parser;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Occurrence counter shared by the `-n`/`-c` value parsers below. clap
/// invokes value parsers in command-line order, so recording a sequence
/// number per occurrence lets main decide which option appeared LAST.
/// GNU head does not reject `-n` + `-c`; the later option governs the
/// output (`head -n 5 -c 3` prints 3 bytes).
static NEXT_OCCURRENCE: AtomicUsize = AtomicUsize::new(0);
static LINES_OCCURRENCE: AtomicUsize = AtomicUsize::new(0);
static BYTES_OCCURRENCE: AtomicUsize = AtomicUsize::new(0);

fn track_lines(raw: &str) -> Result<String, String> {
    LINES_OCCURRENCE.store(
        NEXT_OCCURRENCE.fetch_add(1, Ordering::Relaxed) + 1,
        Ordering::Relaxed,
    );
    Ok(raw.to_owned())
}

fn track_bytes(raw: &str) -> Result<String, String> {
    BYTES_OCCURRENCE.store(
        NEXT_OCCURRENCE.fetch_add(1, Ordering::Relaxed) + 1,
        Ordering::Relaxed,
    );
    Ok(raw.to_owned())
}

/// 0 when `-n` was not given; otherwise its 1-based position among the
/// `-n`/`-c` occurrences on the command line.
pub fn lines_occurrence() -> usize {
    LINES_OCCURRENCE.load(Ordering::Relaxed)
}

/// 0 when `-c` was not given; otherwise its 1-based position among the
/// `-n`/`-c` occurrences on the command line.
pub fn bytes_occurrence() -> usize {
    BYTES_OCCURRENCE.load(Ordering::Relaxed)
}

/// Output the first part of files — a Windows port of the Linux head command.
#[derive(Parser, Debug)]
// GNU getopt allows repeated options and uses the last value; without this
// clap 4 would reject `head -n 5 -n 2` with "cannot be used multiple times".
#[command(name = "head", version, about, args_override_self = true)]
pub struct Args {
    /// Use NUL instead of newline as the record delimiter
    #[arg(short = 'z', long = "zero-terminated")]
    pub zero_terminated: bool,
    /// Print the first N lines instead of the first 10; a negative -N
    /// prints all but the last N lines
    #[arg(
        short = 'n',
        long = "lines",
        value_name = "N",
        allow_negative_numbers = true,
        value_parser = track_lines
    )]
    pub lines: Option<String>,

    /// Print the first N bytes; a negative -N prints all but the last N bytes
    #[arg(
        short = 'c',
        long = "bytes",
        value_name = "N",
        allow_negative_numbers = true,
        value_parser = track_bytes
    )]
    pub bytes: Option<String>,

    /// Never print headers giving file names
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Always print headers giving file names
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// The file(s) to read
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
