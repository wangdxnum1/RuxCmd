use clap::Parser;
use std::env;

mod cli;

fn main() {
    let args = cli::Cli::parse();

    if args.verbose {
        println!("Parsing arguments: {:?}", args);
    }

    if args.ignore_environment {
        if args.verbose {
            println!("Ignoring system environment");
        }
        return;
    }

    let unset_set: std::collections::HashSet<_> = args.unset.iter().collect();

    for (key, value) in env::vars() {
        if unset_set.contains(&key) {
            if args.verbose {
                println!("Skipping unset variable: {}", key);
            }
            continue;
        }

        if args.verbose {
            println!("{}={}", key, value);
        } else {
            println!("{}={}", key, value);
        }
    }
}
