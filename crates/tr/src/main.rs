mod cli;

use clap::Parser;
use cli::Cli;
use std::collections::HashSet;
use std::io::{self, Read, Write};

/// One element of a set specification: a single character (escapes already
/// resolved) or a POSIX character class like `[:alpha:]`.
#[derive(Clone, PartialEq)]
enum Elem {
    Ch(char),
    Class(String),
}

/// Resolve the escape right after a backslash. Returns the character and how
/// many input characters were consumed. `\n` `\t` `\r` `\a` `\b` `\f` `\v`
/// `\\` plus 1-3 octal digits; any other escaped character stands for itself.
fn parse_escape(chars: &[char]) -> Result<(char, usize), String> {
    match chars.first() {
        None => Ok(('\\', 0)), // lone trailing backslash is literal
        Some('n') => Ok(('\n', 1)),
        Some('t') => Ok(('\t', 1)),
        Some('r') => Ok(('\r', 1)),
        Some('a') => Ok(('\u{7}', 1)),
        Some('b') => Ok(('\u{8}', 1)),
        Some('f') => Ok(('\u{c}', 1)),
        Some('v') => Ok(('\u{b}', 1)),
        Some('\\') => Ok(('\\', 1)),
        Some(c) if ('0'..='7').contains(c) => {
            let mut value: u32 = 0;
            let mut digits = 0usize;
            for &d in chars.iter().take(3) {
                if !('0'..='7').contains(&d) {
                    break;
                }
                value = value * 8 + d.to_digit(8).unwrap();
                digits += 1;
            }
            char::from_u32(value)
                .map(|c| (c, digits))
                .ok_or_else(|| "invalid octal escape".to_string())
        }
        Some(other) => Ok((*other, 1)),
    }
}

fn tokenize(pattern: &str) -> Vec<Elem> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut elems = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '[' && i + 1 < chars.len() && chars[i + 1] == ':' {
            if let Some(rel) = chars[i + 2..]
                .windows(2)
                .position(|w| w[0] == ':' && w[1] == ']')
            {
                elems.push(Elem::Class(chars[i + 2..i + 2 + rel].iter().collect()));
                i += 2 + rel + 2;
                continue;
            }
        }
        if chars[i] == '\\' {
            // Escapes resolve during tokenization so ranges and classes see
            // the final characters (`tr '\t-\r' ...` works).
            let (c, consumed) = parse_escape(&chars[i + 1..]).unwrap_or(('\\', 0));
            elems.push(Elem::Ch(c));
            i += 1 + consumed;
            continue;
        }
        elems.push(Elem::Ch(chars[i]));
        i += 1;
    }

    elems
}

fn class_chars(name: &str) -> Option<Vec<char>> {
    // ASCII locale expansion: predictable size and ordering keeps the
    // classic `tr '[:lower:]' '[:upper:]' positional mapping aligned.
    let pred: fn(char) -> bool = match name {
        "alpha" => |c| c.is_ascii_alphabetic(),
        "digit" => |c| c.is_ascii_digit(),
        "alnum" => |c| c.is_ascii_alphanumeric(),
        "space" => |c| c.is_ascii_whitespace() || c == '\u{b}',
        "upper" => |c| c.is_ascii_uppercase(),
        "lower" => |c| c.is_ascii_lowercase(),
        "punct" => |c| c.is_ascii_punctuation(),
        "print" => |c| c.is_ascii_graphic() || c == ' ',
        "graph" => |c| c.is_ascii_graphic(),
        "blank" => |c| c == ' ' || c == '\t',
        "cntrl" => |c| c.is_ascii_control(),
        "xdigit" => |c| c.is_ascii_hexdigit(),
        _ => return None,
    };
    Some(
        (0u32..=0x7f)
            .filter_map(char::from_u32)
            .filter(|c| pred(*c))
            .collect(),
    )
}

/// Expand a set specification (escapes, `c1-c2` ranges, `[:class:]`) into its
/// character sequence.
fn expand_set(pattern: &str) -> Result<Vec<char>, String> {
    let elems = tokenize(pattern);
    let mut out = Vec::new();
    let mut i = 0;

    while i < elems.len() {
        match &elems[i] {
            Elem::Class(name) => {
                let chars = class_chars(name)
                    .ok_or_else(|| format!("invalid character class '{}'", name))?;
                out.extend(chars);
                i += 1;
            }
            Elem::Ch(c) => {
                if i + 2 < elems.len() && elems[i + 1] == Elem::Ch('-') {
                    if let Elem::Ch(end) = elems[i + 2] {
                        if end < *c {
                            return Err(format!(
                                "range-endpoints of '{}-{}' are in reverse collating sequence order",
                                c, end
                            ));
                        }
                        let mut code = *c as u32;
                        while code <= end as u32 {
                            if let Some(ch) = char::from_u32(code) {
                                out.push(ch);
                            }
                            code += 1;
                        }
                        i += 3;
                        continue;
                    }
                }
                out.push(*c);
                i += 1;
            }
        }
    }

    Ok(out)
}

fn translate_char(c: char, set1: &[char], set1_sorted: &[char], set2: &[char], complement: bool) -> char {
    let last = set2[set2.len() - 1];

    if complement {
        // Characters in SET1 pass through; every other character maps
        // through SET2 at the index its position in the sorted complement
        // dictates, with SET2's last character repeating. The complement is
        // never materialized.
        if set1_sorted.binary_search(&c).is_ok() {
            return c;
        }
        let below = set1_sorted.partition_point(|&x| (x as u32) < (c as u32)) as u32;
        let index = (c as u32).saturating_sub(below) as usize;
        set2[index.min(set2.len() - 1)]
    } else {
        match set1.iter().position(|&x| x == c) {
            Some(idx) if idx < set2.len() => set2[idx],
            Some(_) => last,
            None => c,
        }
    }
}

fn squeeze<F: Fn(char) -> bool>(text: &mut String, in_squeeze_set: F) {
    let mut result = String::with_capacity(text.len());
    let mut prev: Option<char> = None;

    for c in text.chars() {
        if !in_squeeze_set(c) || prev != Some(c) {
            result.push(c);
        }
        prev = Some(c);
    }

    *text = result;
}

fn die(msg: &str) -> ! {
    eprintln!("tr: {}", msg);
    std::process::exit(1);
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if cli.version {
        println!("tr 0.1.0");
        return Ok(());
    }

    let set1_str = match cli.set1 {
        Some(s) => s,
        None => {
            eprintln!("tr: missing operand");
            eprintln!("Usage: tr [OPTION]... SET1 [SET2]");
            std::process::exit(1);
        }
    };

    if !cli.delete && cli.set2.is_none() && !cli.squeeze {
        // Translation without SET2 has nothing to translate to.
        die(&format!("missing operand after '{}'", set1_str));
    }

    if !cli.delete {
        if let Some(ref set2) = cli.set2 {
            if set2.is_empty() {
                die("when translating, SET2 must be non-empty");
            }
        }
    }

    let set1 = expand_set(&set1_str).unwrap_or_else(|e| die(&e));
    let set2 = if cli.delete {
        None
    } else {
        cli.set2.as_ref().map(|s| expand_set(s).unwrap_or_else(|e| die(&e)))
    };

    let set1_set: HashSet<char> = set1.iter().copied().collect();
    // `-c` flips membership of the effective first set; kept lazy so no
    // million-entry complement set is ever built.
    let in_set1 = |c: char| set1_set.contains(&c) != cli.complement;

    // tr runs on the raw stream (newlines included), so `tr '\n' ' '` works.
    let mut input = String::new();
    io::stdin().lock().read_to_string(&mut input)?;

    let mut output = String::new();

    if cli.delete {
        for c in input.chars() {
            if !in_set1(c) {
                output.push(c);
            }
        }
    } else if let Some(ref set2) = set2 {
        let mut set1_sorted = set1.clone();
        set1_sorted.sort();
        for c in input.chars() {
            output.push(translate_char(c, &set1, &set1_sorted, set2, cli.complement));
        }
    } else {
        output.push_str(&input);
    }

    if cli.squeeze {
        // GNU squeezes repeats of characters in the LAST specified set:
        // SET2 when translating, otherwise SET1 (complement applied).
        let squeeze_source: &Vec<char> = match &set2 {
            Some(v) => v,
            None => &set1,
        };
        let squeeze_set: HashSet<char> = squeeze_source.iter().copied().collect();
        squeeze(&mut output, |c| squeeze_set.contains(&c) != cli.complement);
    }

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(output.as_bytes())?;

    Ok(())
}
