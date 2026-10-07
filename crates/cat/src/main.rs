mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};

/// Size of each input chunk; matches the classic GNU cat buffer size.
const CHUNK_SIZE: usize = 8 * 1024;

/// Display options derived from the command line (immutable per run).
#[derive(Clone, Copy)]
struct Display {
    show_ends: bool,
    show_tabs: bool,
    show_nonprinting: bool,
    number: bool,
    number_nonblank: bool,
    squeeze_blank: bool,
}

/// Per-input-stream line state (line numbering resets for each file/stdin).
struct LineState {
    line_num: u64,
    at_line_start: bool,
    prev_line_empty: bool,
}

impl LineState {
    fn new() -> Self {
        Self {
            line_num: 0,
            at_line_start: true,
            prev_line_empty: false,
        }
    }

    /// Transform one chunk of input bytes into output bytes.
    ///
    /// Bytes that need no transformation are copied verbatim (including CR,
    /// NUL and high bytes); only -v/-E/-T substitutions rewrite bytes.
    fn process_chunk(&mut self, d: &Display, chunk: &[u8], out: &mut Vec<u8>) {
        for &byte in chunk {
            if byte == b'\n' {
                let is_empty = self.at_line_start;
                if d.squeeze_blank && self.prev_line_empty && is_empty {
                    continue;
                }
                if is_empty && d.number {
                    self.line_num += 1;
                    write_line_number(out, self.line_num);
                }
                if d.show_ends {
                    out.push(b'$');
                }
                out.push(b'\n');
                self.at_line_start = true;
                self.prev_line_empty = is_empty;
            } else {
                if self.at_line_start {
                    if d.number || d.number_nonblank {
                        self.line_num += 1;
                        write_line_number(out, self.line_num);
                    }
                    self.at_line_start = false;
                }
                if d.show_tabs && byte == b'\t' {
                    out.extend_from_slice(b"^I");
                } else if d.show_nonprinting {
                    write_nonprinting(out, byte);
                } else {
                    out.push(byte);
                }
            }
        }
    }
}

fn write_line_number(out: &mut Vec<u8>, n: u64) {
    out.extend_from_slice(format!("{:6}\t", n).as_bytes());
}

/// Write a byte using ^ and M- notation (GNU `cat -v` byte semantics);
/// TAB and LF pass through unchanged. High bytes (0x80-0xFF) are shown as
/// `M-` followed by the low seven bits in caret notation.
fn write_nonprinting(out: &mut Vec<u8>, byte: u8) {
    match byte {
        b'\t' | b'\n' => out.push(byte),
        0x20..=0x7E => out.push(byte),
        0x7F => out.extend_from_slice(b"^?"),
        0x00..=0x1F => {
            out.push(b'^');
            out.push(byte + 0x40);
        }
        _ => {
            let lo = byte & 0x7F;
            out.extend_from_slice(b"M-");
            if lo < 0x20 {
                out.push(b'^');
                out.push(lo + 0x40);
            } else if lo == 0x7F {
                out.extend_from_slice(b"^?");
            } else {
                out.push(lo);
            }
        }
    }
}

fn main() {
    let args = cli::Args::parse();

    let display = Display {
        show_ends: args.show_ends || args.show_all,
        show_tabs: args.show_tabs || args.show_all,
        show_nonprinting: args.show_nonprinting || args.show_all,
        number: args.number,
        number_nonblank: args.number_nonblank,
        squeeze_blank: args.squeeze_blank,
    };

    let mut exit_code = 0i32;

    let stdout = io::stdout();
    let mut writer = BufWriter::with_capacity(CHUNK_SIZE * 2, stdout.lock());

    if args.files.is_empty() {
        let stdin = io::stdin();
        if let Err(e) = cat_reader(stdin.lock(), &display, &mut writer) {
            eprintln!("cat: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for path in &args.files {
            if path.to_string_lossy() == "-" {
                let stdin = io::stdin();
                if let Err(e) = cat_reader(stdin.lock(), &display, &mut writer) {
                    eprintln!("cat: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = cat_reader(reader, &display, &mut writer) {
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

    if let Err(e) = writer.flush() {
        eprintln!("cat: error writing to stdout: {}", e);
        exit_code = 1;
    }

    std::process::exit(exit_code);
}

/// Stream `reader` through the byte-level display pipeline to `writer`
/// in fixed-size chunks; nothing is buffered whole.
fn cat_reader<R: Read>(
    mut reader: R,
    display: &Display,
    writer: &mut impl Write,
) -> Result<(), std::io::Error> {
    let mut state = LineState::new();
    let mut chunk = [0u8; CHUNK_SIZE];
    let mut out: Vec<u8> = Vec::with_capacity(CHUNK_SIZE * 2);
    loop {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        out.clear();
        state.process_chunk(display, &chunk[..n], &mut out);
        writer.write_all(&out)?;
    }
    Ok(())
}
