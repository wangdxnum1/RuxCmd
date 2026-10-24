mod cli;

use std::fs::File;
use std::io::{BufRead, BufReader};

fn main() {
    let args = cli::parse_args();

    let body_numbering = args.body_numbering;
    let separator = args.separator;
    let number_width = args.number_width;

    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = nl_reader(stdin.lock(), body_numbering, &separator, number_width) {
            eprintln!("nl: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for path in &args.files {
            let path_str = path.to_string_lossy();
            if path_str == "-" {
                let stdin = std::io::stdin();
                if let Err(e) = nl_reader(stdin.lock(), body_numbering, &separator, number_width) {
                    eprintln!("nl: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = nl_reader(reader, body_numbering, &separator, number_width)
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

    std::process::exit(exit_code);
}

fn nl_reader<R: BufRead>(
    mut reader: R,
    body_numbering: cli::BodyNumbering,
    separator: &str,
    number_width: usize,
) -> Result<(), std::io::Error> {
    let mut output = String::new();
    let mut line_num: usize = 0;
    let mut raw = String::new();

    loop {
        raw.clear();
        // read_line keeps the trailing '\n' (if any) and, like BufRead::lines,
        // decodes UTF-8 instead of mangling bytes through `byte as char`.
        let bytes_read = reader.read_line(&mut raw)?;
        if bytes_read == 0 {
            break;
        }

        // Remember whether the line was '\n'-terminated so a missing final
        // newline is not invented in the output (GNU behavior).
        let terminated = raw.ends_with('\n');
        let line = raw.strip_suffix('\n').unwrap_or(raw.as_str());
        // Drop only a trailing '\r' (CRLF input); '\r' inside a line is
        // ordinary content and must be preserved.
        let line = line.strip_suffix('\r').unwrap_or(line);

        let number_it = match body_numbering {
            cli::BodyNumbering::All => true,
            cli::BodyNumbering::NonEmpty => !line.is_empty(),
            cli::BodyNumbering::None => false,
        };

        if number_it {
            line_num += 1;
            output.push_str(&format!(
                "{:width$}{}",
                line_num,
                separator,
                width = number_width
            ));
        } else {
            // GNU nl blanks out the whole "number + separator" field.
            output.push_str(&" ".repeat(number_width + separator.len()));
        }
        output.push_str(line);
        if terminated {
            output.push('\n');
        }
    }

    print!("{}", output);
    Ok(())
}
