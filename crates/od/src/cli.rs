use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum AddressBase {
    Octal,
    Decimal,
    Hexadecimal,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DisplayType {
    Octal,
    Hexadecimal,
    SignedDecimal,
    UnsignedDecimal,
    Char,
    Named,
    Float,
    String,
}

/// A parsed `-t` type specification (or the equivalent short-flag form).
#[derive(Debug, Clone, PartialEq)]
pub struct FormatSpec {
    pub display: DisplayType,
    pub size: usize,
    /// `z` suffix: append a `>ascii<` trailer column after each line.
    pub z: bool,
}

#[derive(Parser, Debug)]
#[command(name = "od", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'A', long = "address-radix", default_value = "o")]
    pub address_radix: String,

    #[arg(short = 't', long = "format")]
    pub format: Option<String>,

    #[arg(short = 'x', long = "hexadecimal", conflicts_with = "format")]
    pub hexadecimal: bool,

    #[arg(short = 'c', long = "ascii", conflicts_with = "format")]
    pub ascii: bool,

    #[arg(short = 'd', long = "decimal", conflicts_with = "format")]
    pub decimal: bool,

    #[arg(short = 'o', long = "octal", conflicts_with = "format")]
    pub octal: bool,

    #[arg(short = 'j', long = "skip-bytes")]
    pub skip_bytes: Option<usize>,

    #[arg(short = 'N', long = "read-bytes")]
    pub read_bytes: Option<usize>,

    #[arg(short = 'w', long = "width")]
    pub width: Option<usize>,

    #[arg(short = 'v', long = "version")]
    pub version: bool,

    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}

impl Args {
    pub fn get_address_base(&self) -> AddressBase {
        match self.address_radix.as_str() {
            "d" => AddressBase::Decimal,
            "x" => AddressBase::Hexadecimal,
            "n" => AddressBase::None,
            _ => AddressBase::Octal,
        }
    }

    /// Resolve the display specification from the short flags or `-t`.
    pub fn get_format_spec(&self) -> Result<FormatSpec, String> {
        if self.ascii {
            return Ok(FormatSpec { display: DisplayType::Char, size: 1, z: false });
        }
        if self.decimal {
            return Ok(FormatSpec {
                display: DisplayType::SignedDecimal,
                size: 2,
                z: false,
            });
        }
        if self.octal {
            return Ok(FormatSpec { display: DisplayType::Octal, size: 2, z: false });
        }
        if self.hexadecimal {
            return Ok(FormatSpec {
                display: DisplayType::Hexadecimal,
                size: 2,
                z: false,
            });
        }
        match &self.format {
            Some(f) => parse_format(f),
            // GNU default: "oS" — octal shorts, no ASCII column.
            None => Ok(FormatSpec { display: DisplayType::Octal, size: 2, z: false }),
        }
    }

    pub fn get_words_per_line(&self, size: usize) -> usize {
        if let Some(w) = self.width {
            let per_line = w / size.max(1);
            return per_line.max(1);
        }
        match size {
            1 => 16,
            2 => 8,
            4 => 4,
            8 => 2,
            _ => 8,
        }
    }

    pub fn get_skip_bytes(&self) -> usize {
        self.skip_bytes.unwrap_or(0)
    }

    pub fn get_read_bytes(&self) -> Option<usize> {
        self.read_bytes
    }
}

fn invalid_char(c: char, spec: &str) -> String {
    format!("od: invalid character '{c}' in type string '{spec}'")
}

/// Parse a GNU od type string such as `x1`, `d2`, `o4`, `u1`, `a`, `c`,
/// `f4`, `f8`, `x1z`.
pub fn parse_format(s: &str) -> Result<FormatSpec, String> {
    let mut chars = s.chars();
    let type_char = chars
        .next()
        .ok_or_else(|| format!("od: invalid type string '{s}'"))?;

    let mut size_digits = String::new();
    let mut z = false;
    for c in chars {
        match c {
            '0'..='9' => {
                if z {
                    return Err(invalid_char(c, s));
                }
                size_digits.push(c);
            }
            'z' => {
                if z {
                    return Err(invalid_char(c, s));
                }
                z = true;
            }
            _ => return Err(invalid_char(c, s)),
        }
    }

    let (display, default_size, allowed): (DisplayType, usize, &[usize]) = match type_char {
        // Bare `o`/`x`/`d`/`u` mean sizeof(int) words in GNU od (4 bytes);
        // only the plain default format is "short" (2 bytes).
        'o' => (DisplayType::Octal, 4, &[1, 2, 4, 8]),
        'x' => (DisplayType::Hexadecimal, 4, &[1, 2, 4, 8]),
        'd' => (DisplayType::SignedDecimal, 4, &[1, 2, 4, 8]),
        'u' => (DisplayType::UnsignedDecimal, 4, &[1, 2, 4, 8]),
        'c' => (DisplayType::Char, 1, &[]),
        'a' => (DisplayType::Named, 1, &[]),
        'f' => (DisplayType::Float, 8, &[4, 8]),
        's' => (DisplayType::String, 2, &[1, 2, 4, 8]),
        _ => return Err(invalid_char(type_char, s)),
    };

    let size = if allowed.is_empty() {
        if !size_digits.is_empty() {
            // `c` and `a` take no size at all: GNU rejects "-t c1".
            let c = size_digits.chars().next().unwrap_or(type_char);
            return Err(invalid_char(c, s));
        }
        default_size
    } else if size_digits.is_empty() {
        default_size
    } else {
        match size_digits.parse::<usize>() {
            Ok(n) if allowed.contains(&n) => n,
            Ok(n) => {
                let kind = if matches!(display, DisplayType::Float) {
                    "floating point"
                } else {
                    "integral"
                };
                return Err(format!(
                    "od: invalid type string '{s}';\nthis system doesn't provide a {n}-byte {kind} type"
                ));
            }
            Err(_) => {
                let c = size_digits.chars().next().unwrap_or(type_char);
                return Err(invalid_char(c, s));
            }
        }
    };

    Ok(FormatSpec { display, size, z })
}
