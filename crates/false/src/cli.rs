use clap::Parser;

/// Return a failure exit code
#[derive(Parser, Debug)]
#[command(name = "false", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: (),

    /// Operands and unknown options, all ignored (POSIX)
    #[arg(
        value_name = "ARGS",
        num_args = 0..,
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub ignored: Vec<String>,
}
