mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("hexdump 0.1.0");
        return;
    }

    let start = args.skip.unwrap_or(0);

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    if let Some(path) = &args.file {
        let path_str = path.to_string_lossy();
        if path_str.as_ref() == "-" {
            let stdin = std::io::stdin();
            let mut reader = BufReader::new(stdin.lock());
            if let Err(e) = discard(&mut reader, start)
                .and_then(|()| dump_stream(&mut out, &mut reader, &args, start))
            {
                eprintln!("hexdump: error reading stdin: {}", e);
                std::process::exit(1);
            }
        } else {
            match File::open(path) {
                Ok(mut file) => match seek_file(&mut file, start) {
                    Ok(()) => {
                        let mut reader = BufReader::new(file);
                        if let Err(e) = dump_stream(&mut out, &mut reader, &args, start) {
                            eprintln!("hexdump: {}: {}", path.display(), e);
                            std::process::exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("hexdump: {}: {}", path.display(), e);
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!("hexdump: {}: {}", path.display(), e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        if let Err(e) = discard(&mut reader, start)
            .and_then(|()| dump_stream(&mut out, &mut reader, &args, start))
        {
            eprintln!("hexdump: error reading stdin: {}", e);
            std::process::exit(1);
        }
    }

    if let Err(e) = out.flush() {
        eprintln!("hexdump: write error: {}", e);
        std::process::exit(1);
    }
}

/// Seek a regular file past `-s` bytes. A skip at or beyond the end of the
/// file simply leaves nothing to dump (the old implementation returned
/// early in that case without printing anything, including the trailing
/// offset line of -C).
fn seek_file(file: &mut File, start: usize) -> Result<(), std::io::Error> {
    let len = file.metadata()?.len() as usize;
    file.seek(SeekFrom::Start(start.min(len) as u64))?;
    Ok(())
}

/// Skip `count` bytes of a non-seekable input by reading and discarding
/// them; a short input simply leaves nothing to dump.
fn discard<R: Read>(reader: &mut R, count: usize) -> Result<(), std::io::Error> {
    let mut buf = [0u8; 8192];
    let mut done = 0usize;
    while done < count {
        let want = std::cmp::min(count - done, buf.len());
        let n = read_full(reader, &mut buf[..want])?;
        if n == 0 {
            return Ok(());
        }
        done += n;
    }
    Ok(())
}

/// Read into `buf` until it is full or EOF; returns how many bytes were
/// read (`Ok(0)` only at EOF).
fn read_full<R: Read>(reader: &mut R, mut buf: &mut [u8]) -> Result<usize, std::io::Error> {
    let mut filled = 0usize;
    while !buf.is_empty() {
        match reader.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                filled += n;
                buf = &mut buf[n..];
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(filled)
}

/// Dump the (already skipped) input one 16-byte row at a time; at most one
/// row (bounded by the remaining `-n` quota) is in memory at any moment.
fn dump_stream<W: Write, R: Read>(
    out: &mut W,
    reader: &mut R,
    args: &cli::Args,
    start: usize,
) -> Result<(), std::io::Error> {
    let mut remaining = args.length;
    let mut chunk = [0u8; 16];
    let mut total = 0usize;

    loop {
        let want = remaining.map_or(16, |n| n.min(16));
        if want == 0 {
            break;
        }
        let got = read_full(reader, &mut chunk[..want])?;
        if got == 0 {
            break;
        }
        let line = &chunk[..got];
        let offset = start + total;
        if args.canonical {
            dump_canonical_line(out, line, offset)?;
        } else if args.decimal {
            dump_decimal_line(out, line, offset)?;
        } else if args.octal {
            dump_octal_line(out, line, offset)?;
        } else if args.ascii {
            dump_ascii_line(out, line, offset)?;
        } else {
            dump_hexadecimal_line(out, line, offset)?;
        }
        if let Some(n) = remaining.as_mut() {
            *n -= got;
        }
        total += got;
        if got < want {
            break; // EOF reached mid-row; the tail row was just emitted.
        }
    }

    if args.canonical && total > 0 {
        writeln!(out, "{:08x}", start + total)?;
    }
    Ok(())
}

fn dump_canonical_line<W: Write>(
    out: &mut W,
    line_data: &[u8],
    offset: usize,
) -> Result<(), std::io::Error> {
    write!(out, "{:08x}  ", offset)?;

    for i in 0..16 {
        if i < line_data.len() {
            write!(out, "{:02x} ", line_data[i])?;
        } else {
            write!(out, "   ")?;
        }
        if i == 7 {
            write!(out, " ")?;
        }
    }

    write!(out, " |")?;
    for &byte in line_data {
        if byte.is_ascii_graphic() || byte == b' ' {
            write!(out, "{}", byte as char)?;
        } else {
            write!(out, ".")?;
        }
    }
    writeln!(out, "|")?;

    Ok(())
}

fn dump_hexadecimal_line<W: Write>(
    out: &mut W,
    line_data: &[u8],
    offset: usize,
) -> Result<(), std::io::Error> {
    write!(out, "{:08x}  ", offset)?;

    let mut i = 0;
    while i < line_data.len() {
        let size = std::cmp::min(2, line_data.len() - i);
        if size == 2 {
            write!(
                out,
                "{:04x} ",
                u16::from_be_bytes([line_data[i], line_data[i + 1]])
            )?;
        } else {
            write!(out, "{:02x}   ", line_data[i])?;
        }
        i += 2;
        if i % 8 == 0 {
            write!(out, " ")?;
        }
    }

    writeln!(out)?;
    Ok(())
}

fn dump_decimal_line<W: Write>(
    out: &mut W,
    line_data: &[u8],
    offset: usize,
) -> Result<(), std::io::Error> {
    write!(out, "{:08x}  ", offset)?;

    let mut i = 0;
    while i < line_data.len() {
        let size = std::cmp::min(2, line_data.len() - i);
        if size == 2 {
            write!(
                out,
                "{:05} ",
                u16::from_be_bytes([line_data[i], line_data[i + 1]])
            )?;
        } else {
            write!(out, "{:03}   ", line_data[i])?;
        }
        i += 2;
        if i % 8 == 0 {
            write!(out, " ")?;
        }
    }

    writeln!(out)?;
    Ok(())
}

fn dump_octal_line<W: Write>(
    out: &mut W,
    line_data: &[u8],
    offset: usize,
) -> Result<(), std::io::Error> {
    write!(out, "{:08x}  ", offset)?;

    let mut i = 0;
    while i < line_data.len() {
        let size = std::cmp::min(2, line_data.len() - i);
        if size == 2 {
            write!(
                out,
                "{:06o} ",
                u16::from_be_bytes([line_data[i], line_data[i + 1]])
            )?;
        } else {
            write!(out, "{:03o}   ", line_data[i])?;
        }
        i += 2;
        if i % 8 == 0 {
            write!(out, " ")?;
        }
    }

    writeln!(out)?;
    Ok(())
}

fn dump_ascii_line<W: Write>(
    out: &mut W,
    line_data: &[u8],
    offset: usize,
) -> Result<(), std::io::Error> {
    write!(out, "{:08x}  ", offset)?;

    for &byte in line_data {
        if byte.is_ascii_graphic() || byte == b' ' {
            write!(out, "{} ", byte as char)?;
        } else if byte == b'\n' {
            write!(out, "\\n ")?;
        } else if byte == b'\r' {
            write!(out, "\\r ")?;
        } else if byte == b'\t' {
            write!(out, "\\t ")?;
        } else {
            write!(out, "{:03o} ", byte)?;
        }
    }

    writeln!(out)?;
    Ok(())
}
