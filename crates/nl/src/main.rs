mod cli;

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

fn main() {
    let args = cli::parse_args();

    let body_numbering = args.body_numbering;
    let separator = args.separator;
    let number_width = args.number_width;

    let mut exit_code = 0i32;

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = nl_reader(
            stdin.lock(),
            &mut out,
            body_numbering,
            &separator,
            number_width,
        ) {
            eprintln!("nl: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for path in &args.files {
            let path_str = path.to_string_lossy();
            if path_str == "-" {
                let stdin = std::io::stdin();
                if let Err(e) = nl_reader(
                    stdin.lock(),
                    &mut out,
                    body_numbering,
                    &separator,
                    number_width,
                ) {
                    eprintln!("nl: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) =
                            nl_reader(reader, &mut out, body_numbering, &separator, number_width)
                        {
                            eprintln!("nl: {}: {}", path.display(), e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("nl: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    let _ = out.flush();
    std::process::exit(exit_code);
}

/// Number lines as they stream through: one line is read, formatted and
/// written before the next one is fetched, so neither the input nor the
/// output is ever buffered in full.
fn nl_reader<R: BufRead, W: Write>(
    mut reader: R,
    out: &mut W,
    body_numbering: cli::BodyNumbering,
    separator: &str,
    number_width: usize,
) -> Result<(), std::io::Error> {
    let mut line_num: usize = 0;
    let mut raw: Vec<u8> = Vec::new();

    loop {
        raw.clear();
        // Byte-oriented variant of the previous `read_line` call: the line
        // content is converted lossily below, so non-UTF-8 input no longer
        // aborts the run (valid UTF-8 input is passed through verbatim,
        // exactly as before).
        let bytes_read = reader.read_until(b'\n', &mut raw)?;
        if bytes_read == 0 {
            break;
        }

        // Remember whether the line was '\n'-terminated so a missing final
        // newline is not invented in the output (GNU behavior).
        let terminated = raw.last() == Some(&b'\n');
        if terminated {
            raw.pop();
        }
        // Drop only a trailing '\r' (CRLF input, plus a trailing '\r' on an
        // unterminated final line, matching the previous read_line version);
        // '\r' inside a line is ordinary content and must be preserved.
        if raw.last() == Some(&b'\r') {
            raw.pop();
        }

        let line = String::from_utf8_lossy(&raw);

        let number_it = match body_numbering {
            cli::BodyNumbering::All => true,
            cli::BodyNumbering::NonEmpty => !line.is_empty(),
            cli::BodyNumbering::None => false,
        };

        if number_it {
            line_num += 1;
            write!(
                out,
                "{:width$}{}",
                line_num,
                separator,
                width = number_width
            )?;
        } else {
            // GNU nl blanks out the whole "number + separator" field.
            write!(out, "{}", " ".repeat(number_width + separator.len()))?;
        }
        out.write_all(line.as_bytes())?;
        if terminated {
            out.write_all(b"\n")?;
        }
    }

    Ok(())
}
