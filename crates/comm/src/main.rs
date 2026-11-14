mod cli;

use clap::Parser;
use ruxcmd_core::lines::ByteLines;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

fn main() {
    let args = cli::Cli::parse();
    if args.version {
        println!("comm 0.1.0");
        return;
    }
    if let Err(error) = run(&args) {
        eprintln!("comm: {error}");
        std::process::exit(1);
    }
}

/// Opens an operand: `-` reads standard input, anything else is a file.
fn open_input(operand: &str, separator: u8) -> io::Result<ByteLines<Box<dyn BufRead>>> {
    let reader: Box<dyn BufRead> = if operand == "-" {
        Box::new(BufReader::new(io::stdin().lock()))
    } else {
        Box::new(BufReader::new(File::open(operand)?))
    };
    Ok(ByteLines::with_separator(reader, separator))
}

fn run(args: &cli::Cli) -> io::Result<()> {
    // Sortedness tracking: GNU comm reports the first out-of-order line per
    // input and a general diagnostic at the end, exiting 1.
    let mut prev: [Option<Vec<u8>>; 2] = [None, None];
    let mut unsorted: [bool; 2] = [false, false];
    let first = args
        .file1
        .as_deref()
        .ok_or_else(|| io::Error::other("missing FILE1 and FILE2 operands"))?;
    let second = args
        .file2
        .as_deref()
        .ok_or_else(|| io::Error::other("missing FILE2 operand"))?;
    if first == "-" && second == "-" {
        return Err(io::Error::other("both operands are standard input"));
    }
    let separator = if args.zero_terminated { 0 } else { b'\n' };
    let mut reader1 = open_input(first, separator)?;
    let mut reader2 = open_input(second, separator)?;
    let mut a = reader1.next().transpose()?;
    let mut b = reader2.next().transpose()?;
    let hidden = [args.hide_col1, args.hide_col2, args.hide_col3];
    let mut total = [0u64; 3];
    let mut err = io::BufWriter::new(io::stderr().lock());
    let mut out = io::BufWriter::new(io::stdout().lock());
    loop {
        match (a.as_ref(), b.as_ref()) {
            (Some(first), Some(second)) => {
                check_order(&mut err, 1, first, &mut prev[0], &mut unsorted[0])?;
                check_order(&mut err, 2, second, &mut prev[1], &mut unsorted[1])?;
                match first.cmp(second) {
                    std::cmp::Ordering::Less => {
                        print_record(&mut out, first, 0, hidden, separator, &mut total)?;
                        a = reader1.next().transpose()?;
                    }
                    std::cmp::Ordering::Greater => {
                        print_record(&mut out, second, 1, hidden, separator, &mut total)?;
                        b = reader2.next().transpose()?;
                    }
                    std::cmp::Ordering::Equal => {
                        print_record(&mut out, first, 2, hidden, separator, &mut total)?;
                        a = reader1.next().transpose()?;
                        b = reader2.next().transpose()?;
                    }
                }
            }
            (Some(first), None) => {
                check_order(&mut err, 1, first, &mut prev[0], &mut unsorted[0])?;
                print_record(&mut out, first, 0, hidden, separator, &mut total)?;
                a = reader1.next().transpose()?;
            }
            (None, Some(second)) => {
                check_order(&mut err, 2, second, &mut prev[1], &mut unsorted[1])?;
                print_record(&mut out, second, 1, hidden, separator, &mut total)?;
                b = reader2.next().transpose()?;
            }
            (None, None) => break,
        }
    }
    err.flush()?;

    // GNU: after processing, if either input was unsorted, print the general
    // diagnostic (via main's error handler) and exit 1.
    if unsorted[0] || unsorted[1] {
        return Err(io::Error::other("input is not in sorted order"));
    }

    // GNU `--total` prints the unsuppressed per-column counts (counters are
    // incremented even for hidden columns) and is emitted even for empty
    // inputs; the numbers keep their literal `\t` separators in `-z` mode and
    // the line ends with the record separator.
    if args.total {
        write!(out, "{}\t{}\t{}\ttotal", total[0], total[1], total[2])?;
        out.write_all(&[separator])?;
    }
    out.flush()
}

/// Checks one line against the previous line from the same input; reports
/// the first out-of-order line per file (GNU) and marks the input unsorted.
fn check_order<W: Write>(
    err: &mut W,
    file_no: usize,
    line: &[u8],
    prev: &mut Option<Vec<u8>>,
    unsorted: &mut bool,
) -> io::Result<()> {
    if let Some(previous) = prev {
        if line < previous.as_slice() && !*unsorted {
            writeln!(err, "comm: file {file_no} is not in sorted order")?;
            *unsorted = true;
        }
    }
    *prev = Some(line.to_vec());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn print_record<W: Write>(
    out: &mut W,
    record: &[u8],
    column: usize,
    hidden: [bool; 3],
    separator: u8,
    total: &mut [u64; 3],
) -> io::Result<()> {
    // Counted even when the column is suppressed, like GNU.
    total[column] += 1;
    if hidden[column] {
        return Ok(());
    }
    for _ in 0..hidden[..column].iter().filter(|hide| !**hide).count() {
        out.write_all(b"\t")?;
    }
    out.write_all(record)?;
    out.write_all(&[separator])
}
