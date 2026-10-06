use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "awk",
    version,
    about = "Pattern scanning and processing language",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'F', long = "field-separator", default_value = " ")]
    pub field_separator: String,

    #[arg(long = "assign", value_parser = parse_var_value, action = clap::ArgAction::Append)]
    pub vars: Vec<(String, String)>,

    #[arg(short = 'f', long = "file")]
    pub file: Option<PathBuf>,

    #[arg(value_name = "PROGRAM")]
    pub program: Option<String>,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

fn parse_var_value(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() == 2 {
        Ok((parts[0].to_string(), parts[1].to_string()))
    } else {
        Err(format!("invalid variable assignment: '{}'", s))
    }
}
