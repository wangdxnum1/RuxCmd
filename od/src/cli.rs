use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum AddressBase {
    Octal,
    Decimal,
    Hexadecimal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DisplayType {
    Octal,
    Hexadecimal,
    Decimal,
    Char,
    String,
    Float,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WordSize {
    Byte,
    TwoBytes,
    FourBytes,
    EightBytes,
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
            _ => AddressBase::Octal,
        }
    }

    pub fn get_display_type(&self) -> DisplayType {
        if self.ascii {
            return DisplayType::Char;
        }
        if self.hexadecimal {
            return DisplayType::Hexadecimal;
        }
        if let Some(f) = &self.format {
            match f.chars().next().unwrap() {
                'o' => DisplayType::Octal,
                'x' => DisplayType::Hexadecimal,
                'd' => DisplayType::Decimal,
                'c' => DisplayType::Char,
                's' => DisplayType::String,
                'f' => DisplayType::Float,
                _ => DisplayType::Octal,
            }
        } else {
            DisplayType::Octal
        }
    }

    pub fn get_word_size(&self) -> WordSize {
        if self.ascii || (self.format.as_deref() == Some("c")) {
            return WordSize::Byte;
        }
        if let Some(f) = &self.format {
            let len = f.len();
            if len > 1 {
                match &f[1..] {
                    "2" => WordSize::TwoBytes,
                    "4" => WordSize::FourBytes,
                    "8" => WordSize::EightBytes,
                    _ => WordSize::TwoBytes,
                }
            } else {
                WordSize::TwoBytes
            }
        } else {
            WordSize::TwoBytes
        }
    }

    pub fn get_words_per_line(&self) -> usize {
        match self.get_word_size() {
            WordSize::Byte => 16,
            WordSize::TwoBytes => 8,
            WordSize::FourBytes => 4,
            WordSize::EightBytes => 2,
        }
    }
}
