mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};

use cli::{AddressBase, DisplayType, FormatSpec};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("od 0.1.0");
        return;
    }

    let spec = match args.get_format_spec() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let address_base = args.get_address_base();
    let words_per_line = args.get_words_per_line(spec.size);
    let skip_bytes = args.get_skip_bytes();
    let read_bytes = args.get_read_bytes();

    let mut exit_code = 0i32;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    if let Some(path) = &args.file {
        let path_str = path.to_string_lossy();
        if path_str == "-" {
            let stdin = std::io::stdin();
            let mut reader = BufReader::new(stdin.lock());
            match skip_stdin(&mut reader, skip_bytes) {
                Ok(()) => {
                    if let Err(e) = dump_reader(
                        &mut out,
                        reader,
                        address_base,
                        &spec,
                        words_per_line,
                        skip_bytes,
                        read_bytes,
                    ) {
                        eprintln!("od: error reading stdin: {}", e);
                        exit_code = 1;
                    }
                }
                Err(SkipError::PastEnd) => {
                    eprintln!("od: cannot skip past end of combined input");
                    std::process::exit(1);
                }
                Err(SkipError::Io(e)) => {
                    eprintln!("od: error reading stdin: {}", e);
                    exit_code = 1;
                }
            }
        } else {
            match File::open(path) {
                Ok(mut file) => match seek_file(&mut file, skip_bytes) {
                    Ok(()) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = dump_reader(
                            &mut out,
                            reader,
                            address_base,
                            &spec,
                            words_per_line,
                            skip_bytes,
                            read_bytes,
                        ) {
                            eprintln!("od: {}: {}", path.display(), e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("od: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                },
                Err(e) => {
                    eprintln!("od: {}: {}", path.display(), e);
                    exit_code = 1;
                }
            }
        }
    } else {
        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        match skip_stdin(&mut reader, skip_bytes) {
            Ok(()) => {
                if let Err(e) = dump_reader(
                    &mut out,
                    reader,
                    address_base,
                    &spec,
                    words_per_line,
                    skip_bytes,
                    read_bytes,
                ) {
                    eprintln!("od: error reading stdin: {}", e);
                    exit_code = 1;
                }
            }
            Err(SkipError::PastEnd) => {
                eprintln!("od: cannot skip past end of combined input");
                std::process::exit(1);
            }
            Err(SkipError::Io(e)) => {
                eprintln!("od: error reading stdin: {}", e);
                exit_code = 1;
            }
        }
    }

    let _ = out.flush();
    std::process::exit(exit_code);
}

/// Why a `-j` skip over a non-seekable input could not be completed.
enum SkipError {
    /// The input ended before `skip_bytes` bytes were consumed.
    PastEnd,
    /// The underlying read failed.
    Io(std::io::Error),
}

/// Seek a regular file past `-j` bytes. Skipping past the end of the file is
/// rejected exactly like GNU od; other failures are ordinary I/O errors.
fn seek_file(file: &mut File, skip_bytes: usize) -> Result<(), std::io::Error> {
    let len = file.metadata()?.len() as usize;
    if skip_bytes > len {
        eprintln!("od: cannot skip past end of combined input");
        std::process::exit(1);
    }
    file.seek(SeekFrom::Start(skip_bytes as u64))?;
    Ok(())
}

/// Skip `-j` bytes on a non-seekable input by reading and discarding.
fn skip_stdin<R: Read>(reader: &mut R, skip_bytes: usize) -> Result<(), SkipError> {
    let mut discard = [0u8; 8192];
    let mut skipped = 0usize;
    while skipped < skip_bytes {
        let want = std::cmp::min(skip_bytes - skipped, discard.len());
        match read_full(reader, &mut discard[..want]) {
            Ok(0) => return Err(SkipError::PastEnd),
            Ok(n) => skipped += n,
            Err(e) => return Err(SkipError::Io(e)),
        }
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

fn format_address(addr: usize, base: &AddressBase) -> String {
    match base {
        AddressBase::Octal => format!("{addr:07o}"),
        AddressBase::Decimal => format!("{addr:07}"),
        AddressBase::Hexadecimal => format!("{addr:06x}"),
        AddressBase::None => String::new(),
    }
}

/// Maximum rendered width of one field for the given display type and size
/// (GNU od aligns fields to the widest possible value).
fn field_width(display: &DisplayType, size: usize) -> usize {
    match display {
        DisplayType::Octal => match size {
            1 => 3,
            2 => 6,
            4 => 11,
            8 => 22,
            _ => 6,
        },
        DisplayType::Hexadecimal => match size {
            1 => 2,
            2 => 4,
            4 => 8,
            8 => 16,
            _ => 4,
        },
        DisplayType::SignedDecimal => match size {
            1 => 4,  // "-128"
            2 => 6,  // "-32768"
            4 => 11, // "-2147483648"
            8 => 20,
            _ => 6,
        },
        DisplayType::UnsignedDecimal => match size {
            1 => 3,
            2 => 5,
            4 => 10,
            8 => 20,
            _ => 5,
        },
        DisplayType::Char | DisplayType::Named => 3,
        DisplayType::Float => {
            if size == 4 {
                15
            } else {
                24
            }
        }
        // The `s` pseudo-type renders without fixed-width alignment.
        DisplayType::String => 0,
    }
}

/// Interpret the given bytes as a little-endian unsigned integer; missing
/// high bytes (a short trailing word) count as zero, like GNU od.
fn le_uint(data: &[u8]) -> u64 {
    let mut v: u64 = 0;
    for (i, &b) in data.iter().take(8).enumerate() {
        v |= (b as u64) << (8 * i);
    }
    v
}

/// Little-endian two's-complement signed value; short trailing words are
/// treated as non-negative (zero-extended), like GNU od.
fn le_signed(data: &[u8], size: usize) -> i64 {
    let v = le_uint(data);
    if data.len() == size && size < 8 {
        let bits = size * 8;
        if v & (1u64 << (bits - 1)) != 0 {
            return (v | (u64::MAX << bits)) as i64;
        }
    }
    v as i64
}

/// GNU `-t c` rendering: C-style escapes for the usual suspects, plain
/// three-digit octal (no backslash) for everything else non-printable.
fn char_escape(b: u8) -> String {
    match b {
        0x00 => "\\0".to_string(),
        0x07 => "\\a".to_string(),
        0x08 => "\\b".to_string(),
        0x09 => "\\t".to_string(),
        0x0a => "\\n".to_string(),
        0x0b => "\\v".to_string(),
        0x0c => "\\f".to_string(),
        0x0d => "\\r".to_string(),
        _ => {
            let c = b as char;
            if c.is_ascii_graphic() || b == b' ' {
                c.to_string()
            } else {
                format!("{b:03o}")
            }
        }
    }
}

const CHAR_NAMES: [&str; 33] = [
    "nul", "soh", "stx", "etx", "eot", "enq", "ack", "bel", "bs", "ht", "nl", "vt", "ff", "cr",
    "so", "si", "dle", "dc1", "dc2", "dc3", "dc4", "nak", "syn", "etb", "can", "em", "sub", "esc",
    "fs", "gs", "rs", "us", "sp",
];

/// GNU `-t a` rendering: the high bit is masked off, 0-32 use the name table,
/// 127 is "del", everything else prints the (masked) printable character.
fn named_char(b: u8) -> String {
    let masked = b & 0x7f;
    if masked < 33 {
        CHAR_NAMES[masked as usize].to_string()
    } else if masked == 127 {
        "del".to_string()
    } else {
        (masked as char).to_string()
    }
}

fn strip_fraction_zeros(s: &str) -> String {
    match s.find('.') {
        None => s.to_string(),
        Some(dot) => {
            let frac = s[dot + 1..].trim_end_matches('0');
            if frac.is_empty() {
                s[..dot].to_string()
            } else {
                format!("{}.{}", &s[..dot], frac)
            }
        }
    }
}

/// Emulate C's `%.Ng` for N = 1..=max_precision and return the shortest
/// representation that round-trips (what gnulib's ftoastr does for GNU od).
fn g_format(value: f64, precision: usize) -> String {
    let sci = format!("{:.*e}", precision.saturating_sub(1), value);
    let (mantissa, exp) = match sci.split_once('e') {
        Some(parts) => parts,
        None => return sci,
    };
    let exp10: i32 = exp.parse().unwrap_or(0);
    if exp10 < -4 || exp10 >= precision as i32 {
        let mantissa = strip_fraction_zeros(mantissa);
        let sign = if exp10 < 0 { '-' } else { '+' };
        format!("{mantissa}e{sign}{:02}", exp10.abs())
    } else {
        let decimals = (precision as i32 - 1 - exp10).max(0) as usize;
        strip_fraction_zeros(&format!("{value:.decimals$}"))
    }
}

fn format_float(value: f64, is_f32: bool) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value < 0.0 { "-inf" } else { "inf" }.to_string();
    }
    let max_precision = if is_f32 { 9 } else { 17 };
    for p in 1..=max_precision {
        let candidate = g_format(value, p);
        let round_trips = if is_f32 {
            candidate
                .parse::<f32>()
                .map(|f| f as f64 == value)
                .unwrap_or(false)
        } else {
            candidate.parse::<f64>() == Ok(value)
        };
        if round_trips {
            return candidate;
        }
    }
    g_format(value, max_precision)
}

fn format_word(data: &[u8], spec: &FormatSpec) -> String {
    let width = field_width(&spec.display, spec.size);
    match spec.display {
        DisplayType::Octal => format!("{:0w$o}", le_uint(data), w = width),
        DisplayType::Hexadecimal => format!("{:0w$x}", le_uint(data), w = width),
        DisplayType::SignedDecimal => format!("{:>w$}", le_signed(data, spec.size), w = width),
        DisplayType::UnsignedDecimal => format!("{:>w$}", le_uint(data), w = width),
        DisplayType::Char => data.iter().map(|&b| char_escape(b)).collect(),
        DisplayType::Named => data.iter().map(|&b| named_char(b)).collect(),
        DisplayType::Float => {
            if spec.size == 4 {
                let mut bits: u32 = 0;
                for (i, &b) in data.iter().enumerate() {
                    bits |= (b as u32) << (8 * i);
                }
                format_float(f32::from_bits(bits) as f64, true)
            } else {
                let mut bits: u64 = 0;
                for (i, &b) in data.iter().enumerate() {
                    bits |= (b as u64) << (8 * i);
                }
                format_float(f64::from_bits(bits), false)
            }
        }
        DisplayType::String => data
            .iter()
            .map(|&b| {
                let c = b as char;
                if c.is_ascii_graphic() || b == b' ' {
                    c.to_string()
                } else {
                    format!("\\{b:03o}")
                }
            })
            .collect(),
    }
}

/// Emit one output line for `line_data` (at most one full `-w` row of the
/// chosen format; a short final row is legal). Rendering is byte-identical
/// to the previous whole-buffer implementation.
fn dump_line<W: Write>(
    out: &mut W,
    line_data: &[u8],
    address_base: &AddressBase,
    addr: usize,
    spec: &FormatSpec,
    width: usize,
    words_per_line: usize,
) -> Result<(), std::io::Error> {
    let size = spec.size;

    let addr_str = format_address(addr, address_base);
    if !addr_str.is_empty() {
        write!(out, "{addr_str}")?;
    }

    // GNU od prints every field right-aligned to the type's width with a
    // single leading space; with -A n that space is all that remains of
    // the address column.
    let mut used = 0usize;
    let mut word_offset = 0;
    while word_offset < line_data.len() {
        let word_end = std::cmp::min(word_offset + size, line_data.len());
        let word_data = &line_data[word_offset..word_end];
        let content = format_word(word_data, spec);
        let pad = width.saturating_sub(content.len());
        write!(out, " {}{}", " ".repeat(pad), content)?;
        used += 1 + pad + content.len();
        word_offset += size;
    }

    // `z` suffix: a `>ascii<` trailer column, as in GNU od -t x1z.
    if spec.z {
        let full_width = words_per_line * (width + 1);
        let pad = full_width.saturating_sub(used) + 2;
        write!(out, "{}>", " ".repeat(pad))?;
        for &b in line_data {
            let c = b as char;
            if c.is_ascii_graphic() || b == b' ' {
                write!(out, "{c}")?;
            } else {
                write!(out, ".")?;
            }
        }
        write!(out, "<")?;
    }

    writeln!(out)?;
    Ok(())
}

/// Dump the already-skipped input one output row at a time: at most
/// `words_per_line * size` bytes (bounded by the remaining `-N` quota) are
/// in memory at any moment.
fn dump_reader<W: Write, R: Read>(
    out: &mut W,
    mut reader: R,
    address_base: AddressBase,
    spec: &FormatSpec,
    words_per_line: usize,
    skip_bytes: usize,
    read_bytes: Option<usize>,
) -> Result<(), std::io::Error> {
    let width = field_width(&spec.display, spec.size);
    let size = spec.size;
    let line_size = words_per_line * size;

    let mut chunk = vec![0u8; line_size];
    let mut total = 0usize;

    loop {
        let want = match read_bytes {
            Some(n) => std::cmp::min(line_size, n - total),
            None => line_size,
        };
        if want == 0 {
            break;
        }
        let got = read_full(&mut reader, &mut chunk[..want])?;
        if got == 0 {
            break;
        }
        dump_line(
            out,
            &chunk[..got],
            &address_base,
            skip_bytes + total,
            spec,
            width,
            words_per_line,
        )?;
        total += got;
        if got < want {
            break; // EOF reached mid-row; the tail row was just emitted.
        }
    }

    // GNU od finishes with a final offset line (suppressed by -A n).
    if !matches!(address_base, AddressBase::None) {
        let end_addr = format_address(skip_bytes + total, &address_base);
        writeln!(out, "{end_addr}")?;
    }

    Ok(())
}
