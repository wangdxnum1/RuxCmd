use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "nohup",
    version,
    about = "Run COMMAND immune to hangups: detach it from the current job so it outlives this terminal"
)]
pub struct Cli {
    /// Command and arguments to run; everything after COMMAND is passed
    /// through unchanged, even when it starts with '-'
    #[arg(
        value_name = "COMMAND",
        num_args = 0..,
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub command: Vec<String>,
}
