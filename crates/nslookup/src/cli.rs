use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "nslookup",
    version,
    about = "Query DNS records for a name (types: A, AAAA, CNAME, MX, TXT)",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 't', long = "type", default_value = "A")]
    pub qtype: String,

    #[arg(value_name = "NAME")]
    pub name: Option<String>,
}
