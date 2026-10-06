use clap::Parser;

#[derive(Parser)]
#[command(name = "iostat", disable_version_flag = true)]
pub struct Args {
    #[arg(short, long)]
    pub version: bool,

    #[arg(short, long)]
    pub disk_only: bool,

    #[arg(short, long)]
    pub extended: bool,

    #[arg(default_value = "0")]
    pub delay: u32,

    #[arg(default_value = "1")]
    pub count: u32,
}

impl Args {
    pub fn get_version(&self) -> bool {
        self.version
    }
}
