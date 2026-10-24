use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "ping",
    version,
    about = "Send ICMP echo requests",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    /// Stop after sending COUNT probes (default 4).
    #[arg(short = 'c', long = "count", default_value = "4", value_name = "COUNT")]
    pub count: usize,

    /// Ping until stopped with Ctrl+C (Windows semantics); statistics are
    /// printed on shutdown. Overrides -c.
    #[arg(short = 't')]
    pub continuous: bool,

    /// Timeout in milliseconds to wait for each reply (default 4000).
    #[arg(short = 'w', default_value = "4000", value_name = "MILLISECONDS")]
    pub wait_ms: u64,

    /// Send SIZE bytes of data (0-65500, default 32).
    #[arg(short = 's', long = "size", default_value = "32", value_name = "SIZE")]
    pub size: usize,

    #[arg(value_name = "HOST")]
    pub host: Option<String>,
}
