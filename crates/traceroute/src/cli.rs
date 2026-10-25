use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "traceroute",
    version,
    about = "Trace the route packets take to a host",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    /// Maximum number of hops to probe (1-255, default 30).
    #[arg(
        short = 'm',
        long = "max-hops",
        default_value = "30",
        value_name = "N",
        value_parser = clap::value_parser!(u32).range(1..=255)
    )]
    pub max_hops: u32,

    /// Timeout in milliseconds to wait for each probe (default 4000).
    #[arg(
        short = 'w',
        long = "timeout",
        default_value = "4000",
        value_name = "MILLISECONDS",
        value_parser = clap::value_parser!(u64).range(1..=u32::MAX as u64)
    )]
    pub timeout_ms: u64,

    /// Number of probes per hop (default 3).
    #[arg(
        short = 'q',
        long = "nqueries",
        default_value = "3",
        value_name = "N",
        value_parser = clap::value_parser!(u32).range(1..)
    )]
    pub nqueries: u32,

    /// Use IPv4 only (IPv4 is the only supported family; accepted as a no-op).
    #[arg(short = '4')]
    pub ipv4_only: bool,

    /// Destination host, resolved to an IPv4 address.
    #[arg(value_name = "HOST")]
    pub host: Option<String>,
}
