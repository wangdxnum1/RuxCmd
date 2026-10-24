use clap::Parser;

#[derive(Parser)]
#[command(name = "sar", disable_version_flag = true)]
pub struct Args {
    #[arg(short, long)]
    pub version: bool,

    /// Report CPU utilization (sysstat's -u).
    #[arg(short = 'u', long = "cpu")]
    pub cpu: bool,

    /// Report memory utilization.
    #[arg(short = 'r', long = "memory")]
    pub memory: bool,

    /// Delay between samples in seconds (0 prints a single since-boot report).
    #[arg(default_value = "0")]
    pub delay: u32,

    /// Number of samples; omit for endless sampling when delay > 0.
    pub count: Option<u32>,
}

impl Args {
    pub fn get_version(&self) -> bool {
        self.version
    }
}
