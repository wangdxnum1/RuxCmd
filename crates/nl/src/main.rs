mod cli;

use std::fs::File;
use std::io::{BufReader, Read};

fn main() {
    let args = cli::parse_args();

    let body_numbering = args.body_numbering;
    let separator = args.separator;
    let number_width = args.number_width;

    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = nl_reader(stdin, body_numbering, &separator, number_width) {
            eprintln!("nl: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for path in &args.files {
            let path_str = path.to_string_lossy();
            if path_str == "-" {
                let stdin = std::io::stdin();
                if let Err(e) = nl_reader(stdin, body_numbering, &separator, number_width) {
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

fn nl_reader<R: Read>(
    mut reader: R,
    body_numbering: cli::BodyNumbering,
    separator: &str,
    number_width: usize,
) -> Result<(), std::io::Error> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let mut output = String::new();
    let mut line_num = 0;
    let mut line_started = false;
    let mut line_content = String::new();

    let mut i = 0;
    let len = buffer.len();

    while i < len {
        let byte = buffer[i];

        if byte == b'\r' {
            i += 1;
            continue;
        }

        if byte == b'\n' {
            let is_empty = !line_started;

            match body_numbering {
                cli::BodyNumbering::All => {
                    line_num += 1;
                    output.push_str(&format!(
                        "{:width$}{}",
                        line_num,
                        separator,
                        width = number_width
                    ));
                }
                cli::BodyNumbering::NonBlank => {
                    if !is_empty {
                        line_num += 1;
                        output.push_str(&format!(
                            "{:width$}{}",
                            line_num,
                            separator,
                            width = number_width
                        ));
                    } else {
                        output.push_str(&" ".repeat(number_width));
                        output.push_str(separator);
                    }
                }
            }

            output.push_str(&line_content);
            output.push('\n');

            line_started = false;
            line_content.clear();
            i += 1;
        } else {
            if !line_started {
                line_started = true;
            }
            line_content.push(byte as char);
            i += 1;
        }
    }

    if !line_content.is_empty() {
        let is_empty = !line_started;

        match body_numbering {
            cli::BodyNumbering::All => {
                line_num += 1;
                output.push_str(&format!(
                    "{:width$}{}",
                    line_num,
                    separator,
                    width = number_width
                ));
            }
            cli::BodyNumbering::NonBlank => {
                if !is_empty {
                    line_num += 1;
                    output.push_str(&format!(
                        "{:width$}{}",
                        line_num,
                        separator,
                        width = number_width
                    ));
                } else {
                    output.push_str(&" ".repeat(number_width));
                    output.push_str(separator);
                }
            }
        }

        output.push_str(&line_content);
    }

    print!("{}", output);
    Ok(())
}
