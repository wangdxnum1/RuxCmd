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
    let mut out = io::BufWriter::new(io::stdout().lock());
    loop {
        match (a.as_ref(), b.as_ref()) {
            (Some(first), Some(second)) => match first.cmp(second) {
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
            },
            (Some(first), None) => {
                print_record(&mut out, first, 0, hidden, separator, &mut total)?;
                a = reader1.next().transpose()?;
            }
            (None, Some(second)) => {
                print_record(&mut out, second, 1, hidden, separator, &mut total)?;
                b = reader2.next().transpose()?;
            }
            (None, None) => break,
        }
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
