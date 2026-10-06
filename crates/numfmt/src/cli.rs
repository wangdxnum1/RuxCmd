use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "numfmt",
    version,
    about = "Reformat NUMBER(s), or the numbers from FILE",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(long = "to", value_enum, default_value_t = To::None)]
    pub to: To,

    #[arg(long = "from", value_enum, default_value_t = From::None)]
    pub from: From,

    #[arg(long = "to-unit", default_value_t = 1f64)]
    pub to_unit: f64,

    #[arg(long = "from-unit", default_value_t = 1f64)]
    pub from_unit: f64,

    #[arg(short = 'd', long = "delimiter", default_value = " \t")]
    pub delimiter: String,

    #[arg(long = "field", value_name = "N", default_value_t = 1usize)]
    pub field: usize,

    #[arg(short = 'H', long = "header", default_value_t = 0usize)]
    pub header: usize,

    #[arg(
        short = 'p',
        long = "padding",
        value_name = "N",
        default_value_t = 0isize
    )]
    pub padding: isize,

    #[arg(short = 'S', long = "suffix", value_name = "SUF")]
    pub suffix_opt: Option<String>,

    #[arg(long = "round", value_enum, default_value_t = Round::Up)]
    pub round: Round,

    #[arg(long = "grouping", action = clap::ArgAction::SetTrue)]
    pub grouping: bool,

    #[arg(value_name = "NUMBER_OR_FILE")]
    pub rest: Vec<String>,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum To {
    None,
    Auto,
    Si,
    Iec,
    #[value(name = "iec-i")]
    IecI,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum From {
    None,
    Auto,
    Si,
    Iec,
    #[value(name = "iec-i")]
    IecI,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum Round {
    Up,
    Down,
    FromZero,
    TowardsZero,
    Nearest,
}
