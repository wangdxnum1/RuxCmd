use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "test", version, about, disable_version_flag = true)]
pub struct Args {
    /// Expression operands, evaluated following the POSIX test grammar
    #[arg(
        value_name = "OPERAND",
        num_args = 0..,
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub operand: Vec<String>,

    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),
}
