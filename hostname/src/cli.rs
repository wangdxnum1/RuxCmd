use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "hostname", author, version, about = "Print hostname", disable_version_flag = true)]
pub struct Cli {
    #[arg(short = 'i', long = "ip-address", help = "Print IP address")]
    pub ip_address: bool,

    #[arg(short = 'f', long = "fqdn", help = "Print fully qualified domain name")]
    pub fqdn: bool,

    #[arg(short = 'v', long = "version", help = "Print version information", action = clap::ArgAction::Version)]
    pub version: (),
}

impl Cli {
    pub fn has_no_flags(&self) -> bool {
        !self.ip_address && !self.fqdn
    }
}
