use clap::Parser;

#[derive(Parser)]
#[command(name = "top", disable_version_flag = true)]
pub struct Args {
    #[arg(short, long)]
    pub version: bool,

    #[arg(short, long, default_value = "1")]
    pub iterations: u32,

    #[arg(short = 'd', long, default_value = "3")]
    pub delay: u32,
}

impl Args {
    pub fn get_version(&self) -> bool {
        self.version
    }
}
