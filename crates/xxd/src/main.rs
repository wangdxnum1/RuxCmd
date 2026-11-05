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

    /// Toggle autoskip: a single '*' replaces runs of zero lines
    #[arg(short = 'a', long = "autoskip", action = clap::ArgAction::Count)]
    pub autoskip: u8,

    /// Variable name used by -i (non-alphanumerics become '_')
    #[arg(short = 'n', long = "name", value_name = "NAME")]
    pub name: Option<String>,

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
    /// Effective `-a` state; GNU's flag toggles, so two `-a` turn it back off.
    autoskip: bool,
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
        match args.name.as_deref() {
            Some(name) => Some(sanitize_c_name(name)),
            None => args
                .files
                .first()
                .filter(|file| file.as_str() != "-")
                .map(|file| sanitize_c_name(file)),
        }
    } else {
        None
    };

    let result = match layout.style {
        Style::Normal | Style::LittleEndian => dump_grouped(&mut out, data, start, layout),
        Style::Binary => dump_binary(&mut out, data, start, layout),
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
        autoskip: args.autoskip % 2 == 1,
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

/// GNU replaces every non-alphanumeric byte with `_` and prefixes names that
/// start with a digit with `__`. It never validates or rejects the name.
fn sanitize_c_name(name: &str) -> String {
    let mut sanitized = String::new();
    if name
        .as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_digit())
    {
        sanitized.push_str("__");
    }
    for &byte in name.as_bytes() {
        if byte.is_ascii_alphanumeric() {
            sanitized.push(byte as char);
        } else {
            sanitized.push('_');
        }
    }
    sanitized
}

/// Port of GNU xxd's `xxdline` zero-run tracker. `nz` is the number of non-zero
/// bytes in a full line, 1 for lines that are always shown (partial lines and
/// dumps without `-a`), and -1 for the end-of-input flush. GNU keeps the first
/// zero line, folds the middle of a run into `*`, and always prints the last
/// zero line at end of input, so the quirky transitions below are deliberate.
struct AutoSkip {
    zero_seen: i32,
    saved: String,
}

impl AutoSkip {
    fn new() -> Self {
        Self {
            zero_seen: 0,
            saved: String::new(),
        }
    }

    fn write_line<W: Write>(&mut self, out: &mut W, line: &str, nz: i32) -> std::io::Result<()> {
        if nz == 0 && self.zero_seen == 1 {
            self.saved = line.to_string();
        }
        let body = if nz != 0 {
            true
        } else {
            let previous = self.zero_seen;
            self.zero_seen += 1;
            previous == 0
        };
        if body {
            if nz != 0 {
                if nz < 0 {
                    self.zero_seen -= 1;
                }
                if self.zero_seen == 2 {
                    writeln!(out, "{}", self.saved)?;
                }
                if self.zero_seen > 2 {
                    writeln!(out, "*")?;
                }
            }
            if nz >= 0 || self.zero_seen > 0 {
                writeln!(out, "{}", line)?;
            }
            if nz != 0 {
                self.zero_seen = 0;
            }
        }
        if self.zero_seen >= 0x7f {
            self.zero_seen = 4;
        }
        Ok(())
    }
}

/// Non-zero byte count for a full line; partial lines are always displayed.
fn non_zero_bytes(chunk: &[u8], cols: usize) -> i32 {
    if chunk.len() == cols {
        chunk.iter().filter(|byte| **byte != 0).count() as i32
    } else {
        1
    }
}

fn grouped_line(chunk: &[u8], address: usize, layout: Layout) -> String {
    let mut line = format!("{:08x}: ", address);
    if layout.group == 0 {
        for position in 0..layout.cols {
            match chunk.get(position) {
                Some(byte) => line.push_str(&hex_pair(*byte, layout.uppercase)),
                None => line.push_str("  "),
            }
        }
    } else {
        let groups = layout.cols.div_ceil(layout.group);
        let little = layout.style == Style::LittleEndian;
        for group in 0..groups {
            if group > 0 {
                line.push(' ');
            }
            let first = group * layout.group;
            let count = layout.group.min(layout.cols - first);
            if little {
                let available = chunk.len().saturating_sub(first).min(count);
                let part = &chunk[first..first + available];
                for _ in available..count {
                    line.push_str("  ");
                }
                for byte in part.iter().rev() {
                    line.push_str(&hex_pair(*byte, layout.uppercase));
                }
            } else {
                for position in first..first + count {
                    match chunk.get(position) {
                        Some(byte) => line.push_str(&hex_pair(*byte, layout.uppercase)),
                        None => line.push_str("  "),
                    }
                }
            }
        }
    }
    line.push_str("  ");
    line.push_str(&ascii_column(chunk));
    line
}

fn dump_grouped<W: Write>(
    out: &mut W,
    data: &[u8],
    start: usize,
    layout: Layout,
) -> std::io::Result<()> {
    let mut skip = AutoSkip::new();
    let mut last = String::new();
    for (index, chunk) in data.chunks(layout.cols).enumerate() {
        let line = grouped_line(chunk, start + index * layout.cols, layout);
        if layout.autoskip {
            skip.write_line(out, &line, non_zero_bytes(chunk, layout.cols))?;
        } else {
            writeln!(out, "{}", line)?;
        }
        last = line;
    }
    if layout.autoskip && data.len().is_multiple_of(layout.cols) {
        skip.write_line(out, &last, -1)?;
    }
    Ok(())
}

fn binary_line(chunk: &[u8], address: usize, cols: usize) -> String {
    let mut line = format!("{:08x}: ", address);
    for position in 0..cols {
        if position > 0 {
            line.push(' ');
        }
        match chunk.get(position) {
            Some(byte) => line.push_str(&format!("{:08b}", byte)),
            None => line.push_str("        "),
        }
    }
    line.push_str("  ");
    line.push_str(&ascii_column(chunk));
    line
}

fn dump_binary<W: Write>(
    out: &mut W,
    data: &[u8],
    start: usize,
    layout: Layout,
) -> std::io::Result<()> {
    let cols = layout.cols;
    let mut skip = AutoSkip::new();
    let mut last = String::new();
    for (index, chunk) in data.chunks(cols).enumerate() {
        let line = binary_line(chunk, start + index * cols, cols);
        if layout.autoskip {
            skip.write_line(out, &line, non_zero_bytes(chunk, cols))?;
        } else {
            writeln!(out, "{}", line)?;
        }
        last = line;
    }
    if layout.autoskip && data.len().is_multiple_of(cols) {
        skip.write_line(out, &last, -1)?;
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
        for pair in nibbles.as_chunks::<2>().0 {
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
