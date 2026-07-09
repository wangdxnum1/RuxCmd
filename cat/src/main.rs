mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read};

fn main() {
    let args = cli::Args::parse();

    let show_all = args.show_all;
    let show_ends = args.show_ends || show_all;
    let show_tabs = args.show_tabs || show_all;
    let show_nonprinting = args.show_nonprinting || show_all;

    let number = args.number;
    let number_nonblank = args.number_nonblank;
    let squeeze_blank = args.squeeze_blank;

    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = cat_reader(stdin, show_ends, show_tabs, show_nonprinting, number, number_nonblank, squeeze_blank) {
            eprintln!("cat: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for path in &args.files {
            let path_str = path.to_string_lossy();
            if path_str == "-" {
                let stdin = std::io::stdin();
                if let Err(e) = cat_reader(stdin, show_ends, show_tabs, show_nonprinting, number, number_nonblank, squeeze_blank) {
                    eprintln!("cat: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = cat_reader(reader, show_ends, show_tabs, show_nonprinting, number, number_nonblank, squeeze_blank) {
                            eprintln!("cat: {}: {}", path.display(), e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("cat: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    std::process::exit(exit_code);
}

fn process_byte(byte: u8, show_tabs: bool, show_nonprinting: bool) -> String {
    if show_tabs && byte == b'\t' {
        "^I".to_string()
    } else if show_nonprinting {
        if byte < 0x20 && byte != b'\n' && byte != b'\r' && byte != b'\t' {
            format!("^{}", (byte + 0x40) as char)
        } else if byte == 0x7F {
            "^?".to_string()
        } else if byte > 0x7F {
            let lo = byte & 0x7F;
            if lo < 0x20 {
                format!("M-^{}", (lo + 0x40) as char)
            } else if lo == 0x7F {
                "M-^?".to_string()
            } else {
                format!("M-{}", lo as char)
            }
        } else {
            (byte as char).to_string()
        }
    } else {
        (byte as char).to_string()
    }
}

fn cat_reader<R: Read>(
    mut reader: R,
    show_ends: bool,
    show_tabs: bool,
    show_nonprinting: bool,
    number: bool,
    number_nonblank: bool,
    squeeze_blank: bool,
) -> Result<(), std::io::Error> {
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let mut output = String::new();
    let mut line_num = 0;
    let mut prev_line_empty = false;
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

            if squeeze_blank && prev_line_empty && is_empty {
                i += 1;
                line_started = false;
                line_content.clear();
                continue;
            }

            if number {
                line_num += 1;
                output.push_str(&format!("{:6}\t", line_num));
            } else if number_nonblank && !is_empty {
                line_num += 1;
                output.push_str(&format!("{:6}\t", line_num));
            }

            output.push_str(&line_content);

            if show_ends {
                output.push('$');
            }
            output.push('\n');

            prev_line_empty = is_empty;
            line_started = false;
            line_content.clear();
            i += 1;
        } else {
            if !line_started {
                line_started = true;
            }
            line_content.push_str(&process_byte(byte, show_tabs, show_nonprinting));
            i += 1;
        }
    }

    if !line_content.is_empty() {
        let is_empty = !line_started;

        if number {
            line_num += 1;
            output.push_str(&format!("{:6}\t", line_num));
        } else if number_nonblank && !is_empty {
            line_num += 1;
            output.push_str(&format!("{:6}\t", line_num));
        }

        output.push_str(&line_content);

        if show_ends {
            output.push('$');
        }
    }

    print!("{}", output);
    Ok(())
}