//! GNU printf formatting core, shared with the `awk` printf/sprintf
//! implementation. The reuse loop lives in [`format_format`] callers; this
//! module exposes one formatting pass over a format string.
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conv {
    Dec,
    Unsigned,
    Octal,
    HexLower,
    HexUpper,
    Float,
    SciLower,
    SciUpper,
    GeneralLower,
    GeneralUpper,
    Char,
    Str,
    EscapedArg,
    Quoted,
    Percent,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Flags {
    pub minus: bool,
    pub zero: bool,
    pub plus: bool,
    pub space: bool,
    pub hash: bool,
}

#[derive(Debug)]
pub struct Spec {
    pub flags: Flags,
    pub width: Option<usize>,
    pub width_from_arg: bool,
    pub prec: Option<usize>,
    pub prec_from_arg: bool,
    pub conv: Conv,
}

/// Whether the format string contains at least one conversion directive;
/// GNU reuses the format across arguments only when it does.
pub fn scan_has_conversion(format: &str) -> bool {
    let mut chars = format.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            continue;
        }
        match chars.peek() {
            Some('%') => {
                chars.next();
            }
            Some(_) => return true,
            None => {}
        }
    }
    false
}

/// Run one pass over `format`, writing to `out`. Returns the number of
/// arguments consumed and whether an invalid specification was hit.
pub fn run_format(
    out: &mut impl Write,
    format: &str,
    args: &[String],
    stop: &mut bool,
) -> (usize, bool) {
    let mut used = 0usize;
    let mut error = false;
    let mut chars = format.chars().peekable();

    macro_rules! next_arg {
        () => {{
            let a = args.get(used).cloned();
            if a.is_some() {
                used += 1;
            }
            a
        }};
    }

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                expand_escape(&mut chars, out, stop);
                if *stop {
                    return (used, false);
                }
            }
            '%' => {
                if chars.peek() == Some(&'%') {
                    chars.next();
                    let _ = out.write_all(b"%");
                    continue;
                }
                let spec = match parse_spec(&mut chars) {
                    Ok(Some(s)) => s,
                    Ok(None) => continue,
                    Err(bad) => {
                        eprintln!("printf: {}: invalid conversion specification", bad);
                        return (used, true);
                    }
                };
                let mut spec = spec;
                if spec.width_from_arg {
                    let raw = next_arg!().unwrap_or_default();
                    let n: i64 = raw.trim().parse().unwrap_or(0);
                    if n < 0 {
                        spec.flags.minus = true;
                        spec.width = Some(n.unsigned_abs() as usize);
                    } else {
                        spec.width = Some(n as usize);
                    }
                }
                if spec.prec_from_arg {
                    let raw = next_arg!().unwrap_or_default();
                    let n: i64 = raw.trim().parse().unwrap_or(0);
                    spec.prec = Some(n.max(0) as usize);
                }
                let mut buf: Vec<u8> = Vec::new();
                match spec.conv {
                    Conv::Percent => buf.push(b'%'),
                    Conv::Str => {
                        let raw = next_arg!().unwrap_or_default();
                        push_string(&mut buf, &raw, &spec);
                    }
                    Conv::Char => {
                        let raw = next_arg!().unwrap_or_default();
                        if let Some(ch) = raw.chars().next() {
                            let mut b = [0u8; 4];
                            buf.extend_from_slice(ch.encode_utf8(&mut b).as_bytes());
                        }
                    }
                    Conv::EscapedArg => {
                        let raw = next_arg!().unwrap_or_default();
                        let mut inner = raw.chars().peekable();
                        while let Some(c) = inner.next() {
                            if c == '\\' {
                                expand_escape(&mut inner, &mut buf, stop);
                                if *stop {
                                    break;
                                }
                            } else {
                                let mut b = [0u8; 4];
                                buf.extend_from_slice(c.encode_utf8(&mut b).as_bytes());
                            }
                        }
                        if *stop {
                            let _ = out.write_all(&pad(&buf, &spec));
                            return (used, false);
                        }
                    }
                    Conv::Quoted => {
                        let raw = next_arg!().unwrap_or_default();
                        buf.extend_from_slice(shell_quote(&raw).as_bytes());
                    }
                    Conv::Dec | Conv::Unsigned | Conv::Octal | Conv::HexLower | Conv::HexUpper => {
                        let raw = next_arg!();
                        let parsed = match raw {
                            Some(ref a) => parse_integer(a),
                            None => Ok((0i64, false)),
                        };
                        match parsed {
                            Ok((value, neg)) => {
                                push_integer(&mut buf, value, neg, &spec);
                            }
                            Err(a) => {
                                eprintln!("printf: '{}': expected a numeric value", a);
                                error = true;
                                push_integer(&mut buf, 0, false, &spec);
                            }
                        }
                        // GNU keeps processing the rest of the format after
                        // an invalid numeric; the error only sets the exit
                        // status.
                    }
                    Conv::Float
                    | Conv::SciLower
                    | Conv::SciUpper
                    | Conv::GeneralLower
                    | Conv::GeneralUpper => {
                        let raw = next_arg!();
                        let parsed = match raw {
                            Some(ref a) => a.trim().parse::<f64>().map_err(|_| a.clone()),
                            None => Ok(0.0),
                        };
                        match parsed {
                            Ok(v) => push_float(&mut buf, v, &spec),
                            Err(a) => {
                                eprintln!("printf: '{}': expected a numeric value", a);
                                error = true;
                                push_float(&mut buf, 0.0, &spec);
                            }
                        }
                    }
                }
                let _ = out.write_all(&pad(&buf, &spec));
            }
            other => {
                let mut b = [0u8; 4];
                let _ = out.write_all(other.encode_utf8(&mut b).as_bytes());
            }
        }
    }
    (used, error)
}

pub fn parse_spec(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Result<Option<Spec>, String> {
    let mut spec = Spec {
        flags: Flags::default(),
        width: None,
        width_from_arg: false,
        prec: None,
        prec_from_arg: false,
        conv: Conv::Percent,
    };
    let mut saw_any = false;
    loop {
        match chars.peek() {
            Some('-') => spec.flags.minus = true,
            Some('0') => spec.flags.zero = true,
            Some('+') => spec.flags.plus = true,
            Some(' ') => spec.flags.space = true,
            Some('#') => spec.flags.hash = true,
            _ => break,
        }
        chars.next();
        saw_any = true;
    }
    if chars.peek().is_some_and(|c| c.is_ascii_digit()) {
        saw_any = true;
        let mut w = 0usize;
        while let Some(c) = chars.peek().and_then(|c| c.to_digit(10)) {
            w = w.saturating_mul(10).saturating_add(c as usize);
            chars.next();
        }
        spec.width = Some(w);
    } else if chars.peek() == Some(&'*') {
        chars.next();
        saw_any = true;
        spec.width_from_arg = true;
    }
    if chars.peek() == Some(&'.') {
        chars.next();
        saw_any = true;
        if chars.peek() == Some(&'*') {
            chars.next();
            spec.prec_from_arg = true;
        } else {
            let mut p = 0usize;
            while let Some(c) = chars.peek().and_then(|c| c.to_digit(10)) {
                p = p.saturating_mul(10).saturating_add(c as usize);
                chars.next();
            }
            spec.prec = Some(p);
        }
    }
    let conv = match chars.next() {
        Some('d') | Some('i') => Conv::Dec,
        Some('u') => Conv::Unsigned,
        Some('o') => Conv::Octal,
        Some('x') => Conv::HexLower,
        Some('X') => Conv::HexUpper,
        Some('f') => Conv::Float,
        Some('e') => Conv::SciLower,
        Some('E') => Conv::SciUpper,
        Some('g') => Conv::GeneralLower,
        Some('G') => Conv::GeneralUpper,
        Some('c') => Conv::Char,
        Some('s') => Conv::Str,
        Some('b') => Conv::EscapedArg,
        Some('q') => Conv::Quoted,
        Some('%') => Conv::Percent,
        Some(other) => {
            return Err(format!("%{}", other));
        }
        None => {
            if saw_any {
                // A dangling "%5" style fragment: GNU reports it as invalid.
                return Err("<end of format>".to_string());
            }
            return Ok(None);
        }
    };
    spec.conv = conv;
    Ok(Some(spec))
}

/// Parse a GNU printf integer: leading whitespace, optional sign, then
/// base-0 style (0x hex, 0 octal, decimal). Returns (value, is_negative).
pub fn parse_integer(raw: &str) -> Result<(i64, bool), String> {
    let t = raw.trim_start();
    let (neg, rest) = match t.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let value = if let Some(hex) = rest.strip_prefix("0x").or(rest.strip_prefix("0X")) {
        i64::from_str_radix(hex, 16)
    } else if rest.len() > 1 && rest.starts_with('0') {
        i64::from_str_radix(&rest[1..], 8)
    } else {
        rest.parse::<i64>()
    };
    match value {
        Ok(v) => {
            if neg {
                Ok((v.wrapping_neg(), v != 0 || true))
            } else {
                Ok((v, false))
            }
        }
        Err(_) => Err(raw.to_string()),
    }
}

pub fn push_integer(buf: &mut Vec<u8>, value: i64, neg: bool, spec: &Spec) {
    // Only %d/%i render a sign; the unsigned/octal/hex conversions show the
    // wrapped u64 representation, so -1 becomes 18446744073709551615 etc.
    let neg = neg && spec.conv == Conv::Dec;
    let digits: String = match spec.conv {
        Conv::Dec => value.unsigned_abs().to_string(),
        Conv::Unsigned => (value as u64).to_string(),
        Conv::Octal => format!("{:o}", value as u64),
        Conv::HexLower => format!("{:x}", value as u64),
        Conv::HexUpper => format!("{:X}", value as u64),
        _ => value.to_string(),
    };
    let mut digits = digits;
    if let Some(p) = spec.prec {
        while digits.len() < p {
            digits.insert(0, '0');
        }
    }
    let mut prefix = String::new();
    if neg && value != 0 {
        prefix.push('-');
    } else if spec.flags.plus {
        prefix.push('+');
    } else if spec.flags.space {
        prefix.push(' ');
    }
    if spec.flags.hash {
        match spec.conv {
            Conv::Octal if !digits.starts_with('0') => prefix.insert(0, '0'),
            Conv::HexLower if value != 0 => prefix.push_str("0x"),
            Conv::HexUpper if value != 0 => prefix.push_str("0X"),
            _ => {}
        }
    }
    finish_number(buf, prefix, digits, spec, spec.prec.is_none());
}

pub fn push_float(buf: &mut Vec<u8>, v: f64, spec: &Spec) {
    let neg = v.is_sign_negative() && (v != 0.0 || spec.flags.plus || spec.flags.space);
    let body: String = match spec.conv {
        Conv::Float => {
            let p = spec.prec.unwrap_or(6);
            format!("{:.*}", p, v.abs())
        }
        Conv::SciLower | Conv::SciUpper => {
            let p = spec.prec.unwrap_or(6);
            format_sci(v.abs(), p, matches!(spec.conv, Conv::SciUpper))
        }
        _ => format_general(v.abs(), spec.prec, matches!(spec.conv, Conv::GeneralUpper)),
    };
    let mut prefix = String::new();
    if neg {
        prefix.push('-');
    } else if spec.flags.plus {
        prefix.push('+');
    } else if spec.flags.space {
        prefix.push(' ');
    }
    finish_number(buf, prefix, body, spec, true);
}

/// C-style %e: mantissa with `p` fraction digits, exponent with sign and at
/// least two digits.
pub fn format_sci(v: f64, p: usize, upper: bool) -> String {
    let raw = format!("{:.*e}", p, v);
    let (mant, exp) = raw.split_once('e').unwrap_or((raw.as_str(), "0"));
    let exp_num: i32 = exp.parse().unwrap_or(0);
    let sign = if exp_num < 0 { '-' } else { '+' };
    let mag = exp_num.unsigned_abs().to_string();
    let mag = if mag.len() < 2 {
        format!("0{mag}")
    } else {
        mag
    };
    let e = if upper { 'E' } else { 'e' };
    format!("{mant}{e}{sign}{mag}")
}

/// C-style %g: significant-digit formatting that picks fixed or scientific
/// form and strips trailing zeros.
pub fn format_general(v: f64, prec: Option<usize>, upper: bool) -> String {
    if v == 0.0 {
        return "0".to_string();
    }
    let mut p = prec.unwrap_or(6);
    if p == 0 {
        p = 1;
    }
    let exp_text = format!("{:e}", v);
    let exp_num: i32 = exp_text
        .split_once('e')
        .and_then(|(_, e)| e.parse().ok())
        .unwrap_or(0);
    if exp_num < -4 || exp_num >= p as i32 {
        let s = format_sci(v, p - 1, upper);
        strip_fraction_zeros(s, |c| c == 'e' || c == 'E')
    } else {
        let frac = (p as i32 - 1 - exp_num).max(0) as usize;
        strip_fraction_zeros(format!("{:.*}", frac, v), |_| false)
    }
}

pub fn strip_fraction_zeros(s: String, is_exp_start: fn(char) -> bool) -> String {
    if !s.contains('.') {
        return s;
    }
    // Remove trailing zeros before the exponent marker (if any).
    let split_at = s
        .char_indices()
        .find(|(_, c)| is_exp_start(*c))
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    let (mant, tail) = s.split_at(split_at);
    let mant = mant.trim_end_matches('0').trim_end_matches('.');
    format!("{mant}{tail}")
}

/// Assemble sign/prefix, zero padding and width alignment like C printf.
pub fn finish_number(
    buf: &mut Vec<u8>,
    prefix: String,
    digits: String,
    spec: &Spec,
    zero_ok: bool,
) {
    let width = spec.width.unwrap_or(0);
    let mut s = String::with_capacity(width + digits.len() + prefix.len());
    if spec.flags.zero && zero_ok && !spec.flags.minus && width > prefix.len() + digits.len() {
        let zeros = "0".repeat(width - prefix.len() - digits.len());
        s.push_str(&prefix);
        s.push_str(&zeros);
        s.push_str(&digits);
    } else {
        s.push_str(&prefix);
        s.push_str(&digits);
    }
    if s.len() < width {
        let pad = " ".repeat(width - s.len());
        if spec.flags.minus {
            s.push_str(&pad);
        } else {
            s.insert_str(0, &pad);
        }
    }
    buf.extend_from_slice(s.as_bytes());
}

pub fn push_string(buf: &mut Vec<u8>, raw: &str, spec: &Spec) {
    let mut text = raw.to_string();
    if let Some(p) = spec.prec {
        text = text.chars().take(p).collect();
    }
    buf.extend_from_slice(text.as_bytes());
}

/// Pad a rendered directive to the field width (strings and chars).
pub fn pad(body: &[u8], spec: &Spec) -> Vec<u8> {
    let width = spec.width.unwrap_or(0);
    if body.len() >= width {
        return body.to_vec();
    }
    let mut out = Vec::with_capacity(width);
    if spec.flags.minus {
        out.extend_from_slice(body);
        out.extend(std::iter::repeat_n(b' ', width - body.len()));
    } else {
        out.extend(std::iter::repeat_n(b' ', width - body.len()));
        out.extend_from_slice(body);
    }
    out
}

/// POSIX shell quoting used by GNU printf's %q extension.
pub fn shell_quote(raw: &str) -> String {
    format!("'{}'", raw.replace('\'', "'\\''"))
}

/// Expand one backslash escape (the echo -e family). Unknown escapes keep
/// the backslash and the character verbatim; \c stops all further output.
pub fn expand_escape(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    out: &mut impl Write,
    stop: &mut bool,
) {
    macro_rules! push {
        ($b:expr) => {{
            let _ = out.write_all(&[$b]);
        }};
    }
    match chars.next() {
        Some('a') => push!(0x07),
        Some('b') => push!(0x08),
        Some('e') => push!(0x1B),
        Some('f') => push!(0x0C),
        Some('n') => push!(b'\n'),
        Some('r') => push!(b'\r'),
        Some('t') => push!(b'\t'),
        Some('v') => push!(0x0B),
        Some('\\') => push!(b'\\'),
        Some('"') => push!(b'"'),
        Some('\'') => push!(b'\''),
        Some('?') => push!(b'?'),
        Some('c') => {
            *stop = true;
        }
        Some('x') => {
            let mut value: u32 = 0;
            let mut digits = 0;
            while digits < 2 {
                match chars.peek().and_then(|c| c.to_digit(16)) {
                    Some(d) => {
                        value = value * 16 + d;
                        chars.next();
                        digits += 1;
                    }
                    None => break,
                }
            }
            if digits == 0 {
                let _ = out.write_all(b"\\x");
            } else {
                push!((value & 0xFF) as u8);
            }
        }
        Some(d) if ('0'..='7').contains(&d) => {
            // coreutils printf: the first octal digit, then up to three
            // more (so \0101 parses as \0 + "101" = 0o101).
            let mut value: u32 = d.to_digit(8).unwrap();
            let mut count = 0;
            while count < 3 {
                match chars.peek().and_then(|c| c.to_digit(8)) {
                    Some(next) => {
                        value = value * 8 + next;
                        chars.next();
                        count += 1;
                    }
                    None => break,
                }
            }
            push!((value & 0xFF) as u8);
        }
        Some(other) => {
            let _ = out.write_all(b"\\");
            let mut b = [0u8; 4];
            let _ = out.write_all(other.encode_utf8(&mut b).as_bytes());
        }
        None => {
            let _ = out.write_all(b"\\");
        }
    }
}
