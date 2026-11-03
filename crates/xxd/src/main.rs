use clap::Parser;
use std::io::{Read, Write};

/// Make a hexdump or do the reverse — a Windows port of Linux xxd.
#[derive(Parser, Debug)]
#[command(name = "xxd", version, about)]
pub struct Args {
    /// Reverse operation: convert hex dump back to binary
    #[arg(short = 'r', long = "revert")]
    pub revert: bool,

    /// Bytes per line (default 16; -i: 12, -p: 30, -b: 6; 0 keeps the default)
    #[arg(short = 'c', long = "cols", value_name = "COLS")]
    pub cols: Option<usize>,

    /// Stop after LEN octets (decimal or 0x hex)
    #[arg(short = 'l', long = "len", value_name = "LEN")]
    pub len: Option<String>,

    /// Start at SEEK octets; -N counts from the end, +N is relative
    #[arg(
        short = 's',
        long = "seek",
        value_name = "SEEK",
        allow_hyphen_values = true
    )]
    pub seek: Option<String>,

    /// Octets per group in normal output (default 2; -e: 4; 0 = no grouping)
    #[arg(short = 'g', long = "group", value_name = "BYTES")]
    pub group: Option<usize>,

    /// Plain hex dump without address or ASCII columns
    #[arg(short = 'p', long = "plain")]
    pub plain: bool,

    /// Output in C include file style
    #[arg(short = 'i', long = "include")]
    pub include: bool,

    /// Little-endian dump (reverse octets inside each group)
    #[arg(short = 'e', long = "little-endian")]
    pub little_endian: bool,

    /// Use upper case hex letters
    #[arg(short = 'u', long = "uppercase")]
    pub uppercase: bool,

    /// Binary digit dump
    #[arg(short = 'b', long = "binary")]
    pub binary: bool,

    /// Read from FILE instead of stdin
    #[arg(value_name = "FILE")]
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Style {
    Normal,
    Binary,
    LittleEndian,
    Plain,
    Include,
}

#[derive(Debug, Clone, Copy)]
struct Layout {
    style: Style,
    cols: usize,
    group: usize,
    uppercase: bool,
    /// `-b` also selects `0b` literals when combined with `-i`.
    bits: bool,
}

fn main() {
    let args = Args::parse();
    let mut input = Vec::new();

    for file in &args.files {
        if file == "-" {
            let _ = std::io::stdin().read_to_end(&mut input);
        } else {
            match std::fs::read(file) {
                Ok(mut data) => input.append(&mut data),
                Err(e) => {
                    eprintln!("xxd: {}: {}", file, e);
                    std::process::exit(1);
                }
            }
        }
    }
    if args.files.is_empty() {
        let _ = std::io::stdin().read_to_end(&mut input);
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    if args.revert {
        let offset = match args.seek.as_deref() {
            None => 0,
            Some(seek) => match parse_unsigned(seek.trim_start_matches('+')) {
                Some(value) => value,
                None => {
                    eprintln!("xxd: -r only supports a non-negative absolute -s offset");
                    std::process::exit(1);
                }
            },
        };
        let bytes = revert_dump(&input, offset);
        let _ = out.write_all(&bytes);
        let _ = out.flush();
        return;
    }

    let layout = match resolve_layout(&args) {
        Ok(layout) => layout,
        Err(message) => {
            eprintln!("xxd: {}", message);
            std::process::exit(1);
        }
    };
    let total = input.len();
    let start = match args.seek.as_deref() {
        None => 0,
        Some(seek) => match resolve_seek(seek, total) {
            Some(start) => start,
            None => {
                eprintln!("xxd: Sorry, cannot seek.");
                std::process::exit(4);
            }
        },
    };
    let limit = match args.len.as_deref() {
        None => total - start,
        Some(text) => match parse_unsigned(text) {
            Some(value) => value.min(total - start),
            None => {
                eprintln!("xxd: invalid length '{}'", text);
                std::process::exit(1);
            }
        },
    };
    let data = &input[start..start + limit];

    let include_name = if layout.style == Style::Include {
        args.files.first().filter(|file| *file != "-").map(|file| {
            file.chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect::<String>()
        })
    } else {
        None
    };

    let result = match layout.style {
        Style::Normal | Style::LittleEndian => dump_grouped(&mut out, data, start, layout),
        Style::Binary => dump_binary(&mut out, data, start, layout.cols),
        Style::Plain => dump_plain(&mut out, data, layout),
        Style::Include => dump_include(&mut out, data, layout, include_name.as_deref()),
    };
    if let Err(e) = result {
        eprintln!("xxd: {}", e);
        std::process::exit(1);
    }
    let _ = out.flush();
}

fn resolve_layout(args: &Args) -> Result<Layout, String> {
    // GNU xxd rejects -p with -b/-e/-i and -e with -b/-i. -u and -b+i are
    // accepted; -i takes precedence over -b.
    if args.plain && (args.binary || args.little_endian || args.include)
        || args.little_endian && (args.binary || args.include)
    {
        return Err("only one of -b, -e, -u, -p, -i can be used".to_string());
    }
    let style = if args.include {
        Style::Include
    } else if args.binary {
        Style::Binary
    } else if args.little_endian {
        Style::LittleEndian
    } else if args.plain {
        Style::Plain
    } else {
        Style::Normal
    };
    let default_cols = match style {
        Style::Include if args.binary => 6,
        Style::Include => 12,
        Style::Plain => 30,
        Style::Binary => 6,
        Style::Normal | Style::LittleEndian => 16,
    };
    let cols = args.cols.filter(|cols| *cols > 0).unwrap_or(default_cols);
    let default_group = if style == Style::LittleEndian { 4 } else { 2 };
    let group = args.group.unwrap_or(default_group);
    Ok(Layout {
        style,
        cols,
        group,
        uppercase: args.uppercase,
        bits: args.binary,
    })
}

/// Seek accepts decimal or 0x hex; `-N` counts back from the end, `+N` is
/// relative to the start.
fn resolve_seek(seek: &str, total: usize) -> Option<usize> {
    let text = seek.trim();
    if let Some(rest) = text.strip_prefix('-') {
        let offset = parse_unsigned(rest)?;
        total.checked_sub(offset)
    } else {
        let rest = text.strip_prefix('+').unwrap_or(text);
        let offset = parse_unsigned(rest)?;
        (offset <= total).then_some(offset)
    }
}

fn parse_unsigned(text: &str) -> Option<usize> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let (digits, radix) = match text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        Some(rest) => (rest, 16),
        None => (text, 10),
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_digit(radix)) {
        return None;
    }
    usize::from_str_radix(digits, radix).ok()
}

fn hex_pair(byte: u8, upper: bool) -> String {
    if upper {
        format!("{:02X}", byte)
    } else {
        format!("{:02x}", byte)
    }
}

fn ascii_column(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| {
            if (0x20..0x7f).contains(&b) {
                b as char
            } else {
                '.'
            }
        })
        .collect()
}

fn dump_grouped<W: Write>(
    out: &mut W,
    data: &[u8],
    start: usize,
    layout: Layout,
) -> std::io::Result<()> {
    let little = layout.style == Style::LittleEndian;
    for (index, chunk) in data.chunks(layout.cols).enumerate() {
        let address = start + index * layout.cols;
        write!(out, "{:08x}: ", address)?;
        if layout.group == 0 {
            for position in 0..layout.cols {
                match chunk.get(position) {
                    Some(byte) => write!(out, "{}", hex_pair(*byte, layout.uppercase))?,
                    None => write!(out, "  ")?,
                }
            }
        } else {
            let groups = layout.cols.div_ceil(layout.group);
            for group in 0..groups {
                if group > 0 {
                    write!(out, " ")?;
                }
                let first = group * layout.group;
                let count = layout.group.min(layout.cols - first);
                if little {
                    let available = chunk.len().saturating_sub(first).min(count);
                    let part = &chunk[first..first + available];
                    for _ in available..count {
                        write!(out, "  ")?;
                    }
                    for byte in part.iter().rev() {
                        write!(out, "{}", hex_pair(*byte, layout.uppercase))?;
                    }
                } else {
                    for position in first..first + count {
                        match chunk.get(position) {
                            Some(byte) => write!(out, "{}", hex_pair(*byte, layout.uppercase))?,
                            None => write!(out, "  ")?,
                        }
                    }
                }
            }
        }
        writeln!(out, "  {}", ascii_column(chunk))?;
    }
    Ok(())
}

fn dump_binary<W: Write>(
    out: &mut W,
    data: &[u8],
    start: usize,
    cols: usize,
) -> std::io::Result<()> {
    for (index, chunk) in data.chunks(cols).enumerate() {
        write!(out, "{:08x}: ", start + index * cols)?;
        for position in 0..cols {
            if position > 0 {
                write!(out, " ")?;
            }
            match chunk.get(position) {
                Some(byte) => write!(out, "{:08b}", byte)?,
                None => write!(out, "        ")?,
            }
        }
        writeln!(out, "  {}", ascii_column(chunk))?;
    }
    Ok(())
}

fn dump_plain<W: Write>(out: &mut W, data: &[u8], layout: Layout) -> std::io::Result<()> {
    for chunk in data.chunks(layout.cols) {
        let mut line = String::with_capacity(chunk.len() * 2);
        for byte in chunk {
            line.push_str(&hex_pair(*byte, layout.uppercase));
        }
        writeln!(out, "{}", line)?;
    }
    Ok(())
}

fn dump_include<W: Write>(
    out: &mut W,
    data: &[u8],
    layout: Layout,
    name: Option<&str>,
) -> std::io::Result<()> {
    // GNU keeps the 0b prefix lower case even with -u.
    let (prefix, radix) = if layout.bits {
        ("0b", "b")
    } else {
        (if layout.uppercase { "0X" } else { "0x" }, "x")
    };
    if let Some(name) = name {
        writeln!(out, "unsigned char {}[] = {{", name)?;
    }
    let chunks: Vec<&[u8]> = data.chunks(layout.cols).collect();
    for (index, chunk) in chunks.iter().enumerate() {
        write!(out, "  ")?;
        for (position, byte) in chunk.iter().enumerate() {
            if position > 0 {
                write!(out, ", ")?;
            }
            if radix == "b" {
                write!(out, "{}{:08b}", prefix, byte)?;
            } else {
                write!(out, "{}{}", prefix, hex_pair(*byte, layout.uppercase))?;
            }
        }
        if index + 1 < chunks.len() {
            write!(out, ",")?;
        }
        writeln!(out)?;
    }
    if let Some(name) = name {
        writeln!(out, "}};")?;
        writeln!(out, "unsigned int {}_len = {};", name, data.len())?;
    }
    Ok(())
}

/// Decode `xxd` output back to bytes. Only the hexadecimal part after the
/// address is data; the ASCII column is display-only and must never be
/// parsed. Addresses are honoured, so gaps are zero-filled like GNU xxd.
fn revert_dump(input: &[u8], offset: usize) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for raw in input.split(|&b| b == b'\n') {
        let line = raw.strip_suffix(b"\r").unwrap_or(raw);
        let trimmed = line
            .iter()
            .position(|&b| b != b' ')
            .map(|start| &line[start..])
            .unwrap_or(&[]);
        let Some(colon) = trimmed.iter().position(|&b| b == b':') else {
            // `*` fold lines and the trailing empty line carry no data.
            continue;
        };
        let Some(address) = parse_address(&trimmed[..colon]) else {
            continue;
        };
        let address = address.wrapping_add(offset);
        let body = &trimmed[colon + 1..];
        let nibbles = hex_nibbles(body);
        let mut bytes = Vec::with_capacity(nibbles.len() / 2);
        for pair in nibbles.chunks_exact(2) {
            bytes.push((pair[0] << 4) | pair[1]);
        }
        if address > out.len() {
            out.resize(address, 0);
        }
        for (index, byte) in bytes.into_iter().enumerate() {
            let position = address + index;
            if position < out.len() {
                out[position] = byte;
            } else {
                out.push(byte);
            }
        }
    }
    out
}

/// Hex digits of a dump line, stopping at the ASCII column separator (two
/// spaces). A single `-` acts as a group separator like GNU xxd.
fn hex_nibbles(body: &[u8]) -> Vec<u8> {
    let mut nibbles = Vec::new();
    let mut spaces = 0usize;
    let mut started = false;
    for &byte in body {
        match byte {
            b' ' => {
                spaces += 1;
                if started && spaces >= 2 {
                    break;
                }
            }
            b'-' => {
                spaces = 0;
            }
            _ => match hex_value(byte) {
                Some(value) => {
                    started = true;
                    spaces = 0;
                    nibbles.push(value);
                }
                None => break,
            },
        }
    }
    nibbles
}

fn parse_address(text: &[u8]) -> Option<usize> {
    if text.is_empty() {
        return None;
    }
    let mut value = 0usize;
    for &byte in text {
        value = value
            .checked_mul(16)?
            .checked_add(hex_value(byte)? as usize)?;
    }
    Some(value)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
