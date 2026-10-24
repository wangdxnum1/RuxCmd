use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "curl",
    version,
    about = "Transfer data from or to a server",
    disable_version_flag = true
)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    /// HTTP method (GET, POST, PUT, DELETE, HEAD). Overrides the POST
    /// implied by -d.
    #[arg(short = 'X', long = "request", value_name = "METHOD")]
    pub method: Option<String>,

    /// Extra request header, e.g. -H "Content-Type: application/json".
    #[arg(short = 'H', long = "header", value_name = "HEADER", action = clap::ArgAction::Append)]
    pub headers: Vec<String>,

    /// Request body; implies POST unless -X overrides the method.
    #[arg(short = 'd', long = "data", value_name = "DATA")]
    pub data: Option<String>,

    /// Include the HTTP status line and response headers in the output
    /// (GNU curl prints the body only by default).
    #[arg(short = 'i', long = "include", action = clap::ArgAction::SetTrue)]
    pub include: bool,

    /// Issue a HEAD request instead of GET.
    #[arg(short = 'I', long = "head", action = clap::ArgAction::SetTrue)]
    pub head: bool,

    /// Return exit code 22 and no output on HTTP errors >= 400.
    #[arg(short = 'f', long = "fail", action = clap::ArgAction::SetTrue)]
    pub fail: bool,

    /// Follow HTTP redirects (up to 10 hops). Redirects are not followed
    /// by default, matching GNU curl.
    #[arg(short = 'L', long = "location", action = clap::ArgAction::SetTrue)]
    pub location: bool,

    /// Silent mode: suppress progress and error messages.
    #[arg(short = 's', long = "silent", action = clap::ArgAction::SetTrue)]
    pub silent: bool,

    /// Write the response body to FILE instead of stdout.
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    pub output: Option<PathBuf>,

    #[arg(value_name = "URL")]
    pub url: Option<String>,
}
