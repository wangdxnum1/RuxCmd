mod cli;

use clap::Parser;

fn main() {
    let args = cli::Args::parse();

    if args.hostname {
        if let Ok(hostname) = std::env::var("COMPUTERNAME") {
            println!("{}", hostname);
        } else {
            println!("localhost");
        }
    } else if args.username {
        if let Ok(username) = std::env::var("USERNAME") {
            println!("{}", username);
        } else {
            println!("unknown");
        }
    } else {
        if let Ok(username) = std::env::var("USERNAME") {
            println!("{}", username);
        } else {
            println!("unknown");
        }
    }
}
