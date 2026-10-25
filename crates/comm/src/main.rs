mod cli;

use clap::Parser;
use ruxcmd_core::lines::ByteLines;
use std::fs::File;
use std::io::{self, BufReader, Write};

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

fn run(args: &cli::Cli) -> io::Result<()> {
    let first = args
        .file1
        .as_deref()
        .ok_or_else(|| io::Error::other("missing FILE1 and FILE2 operands"))?;
    let second = args
        .file2
        .as_deref()
        .ok_or_else(|| io::Error::other("missing FILE2 operand"))?;
    let separator = if args.zero_terminated { 0 } else { b'\n' };
    let mut reader1 = ByteLines::with_separator(BufReader::new(File::open(first)?), separator);
    let mut reader2 = ByteLines::with_separator(BufReader::new(File::open(second)?), separator);
    let mut a = reader1.next().transpose()?;
    let mut b = reader2.next().transpose()?;
    let hidden = [args.hide_col1, args.hide_col2, args.hide_col3];
    let mut out = io::BufWriter::new(io::stdout().lock());
    loop {
        match (a.as_ref(), b.as_ref()) {
            (Some(first), Some(second)) => match first.cmp(second) {
                std::cmp::Ordering::Less => {
                    print_record(&mut out, first, 0, hidden, separator)?;
                    a = reader1.next().transpose()?;
                }
                std::cmp::Ordering::Greater => {
                    print_record(&mut out, second, 1, hidden, separator)?;
                    b = reader2.next().transpose()?;
                }
                std::cmp::Ordering::Equal => {
                    print_record(&mut out, first, 2, hidden, separator)?;
                    a = reader1.next().transpose()?;
                    b = reader2.next().transpose()?;
                }
            },
            (Some(first), None) => {
                print_record(&mut out, first, 0, hidden, separator)?;
                a = reader1.next().transpose()?;
            }
            (None, Some(second)) => {
                print_record(&mut out, second, 1, hidden, separator)?;
                b = reader2.next().transpose()?;
            }
            (None, None) => break,
        }
    }
    out.flush()
}

fn print_record<W: Write>(
    out: &mut W,
    record: &[u8],
    column: usize,
    hidden: [bool; 3],
    separator: u8,
) -> io::Result<()> {
    if hidden[column] {
        return Ok(());
    }
    for _ in 0..hidden[..column].iter().filter(|hide| !**hide).count() {
        out.write_all(b"\t")?;
    }
    out.write_all(record)?;
    out.write_all(&[separator])
}
