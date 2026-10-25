mod cli;

use clap::Parser;
use ruxcmd_core::lines::ByteLines;
use std::fs::File;
use std::io::{self, BufReader, Write};

enum Input {
    File(ByteLines<BufReader<File>>),
    Stdin,
}

fn main() {
    let args = cli::Args::parse();
    if let Err(error) = run(&args) {
        eprintln!("paste: {error}");
        std::process::exit(1);
    }
}

fn run(args: &cli::Args) -> io::Result<()> {
    let separator = if args.zero_terminated { 0 } else { b'\n' };
    let delimiters = args
        .delimiters
        .as_deref()
        .map(parse_delimiters)
        .unwrap_or_else(|| vec![vec![b'\t']]);
    let mut inputs = Vec::new();
    if args.files.is_empty() {
        inputs.push(Input::Stdin);
    }
    for path in &args.files {
        if path.as_os_str() == "-" {
            inputs.push(Input::Stdin);
        } else {
            inputs.push(Input::File(ByteLines::with_separator(
                BufReader::new(File::open(path)?),
                separator,
            )));
        }
    }
    let mut stdin = ByteLines::with_separator(BufReader::new(io::stdin()), separator);
    let mut out = io::BufWriter::new(io::stdout().lock());
    if args.serial {
        for input in &mut inputs {
            let mut count = 0;
            while let Some(record) = next_record(input, &mut stdin)? {
                if count > 0 {
                    out.write_all(delimiter_for(&delimiters, count - 1))?;
                }
                out.write_all(&record)?;
                count += 1;
            }
            out.write_all(&[separator])?;
        }
    } else {
        loop {
            let records: Vec<_> = inputs
                .iter_mut()
                .map(|input| next_record(input, &mut stdin))
                .collect::<io::Result<_>>()?;
            if records.iter().all(Option::is_none) {
                break;
            }
            for (index, record) in records.iter().enumerate() {
                if index > 0 {
                    out.write_all(delimiter_for(&delimiters, index - 1))?;
                }
                if let Some(record) = record {
                    out.write_all(record)?;
                }
            }
            out.write_all(&[separator])?;
        }
    }
    out.flush()
}

fn next_record(
    input: &mut Input,
    stdin: &mut ByteLines<BufReader<io::Stdin>>,
) -> io::Result<Option<Vec<u8>>> {
    match input {
        Input::File(reader) => reader.next().transpose(),
        Input::Stdin => stdin.next().transpose(),
    }
}

fn parse_delimiters(text: &str) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        let value = if ch == '\\' {
            match chars.peek().copied() {
                Some('n') => {
                    chars.next();
                    Some('\n')
                }
                Some('t') => {
                    chars.next();
                    Some('\t')
                }
                Some('0') => {
                    chars.next();
                    None
                }
                Some('\\') => {
                    chars.next();
                    Some('\\')
                }
                _ => Some(ch),
            }
        } else {
            Some(ch)
        };
        result.push(
            value
                .map(|ch| {
                    let mut buffer = [0; 4];
                    ch.encode_utf8(&mut buffer).as_bytes().to_vec()
                })
                .unwrap_or_default(),
        );
    }
    result
}

fn delimiter_for(delimiters: &[Vec<u8>], index: usize) -> &[u8] {
    if delimiters.is_empty() {
        &[]
    } else {
        &delimiters[index % delimiters.len()]
    }
}
