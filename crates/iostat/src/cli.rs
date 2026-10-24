use clap::Parser;

#[derive(Parser)]
#[command(name = "iostat", disable_version_flag = true)]
pub struct Args {
    #[arg(short, long)]
    pub version: bool,

    /// Delay between reports in seconds (0 prints a single since-boot report).
    #[arg(default_value = "0")]
    pub delay: u32,

    /// Number of reports; omit for endless sampling when delay > 0.
    pub count: Option<u32>,
}

impl Args {
    pub fn get_version(&self) -> bool {
        self.version
    }
}
