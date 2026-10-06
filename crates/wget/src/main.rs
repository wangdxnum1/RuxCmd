mod cli;

use clap::Parser;
use reqwest::blocking::Client;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("wget 0.1.0");
        return;
    }

    let url = args.url.as_deref().unwrap_or_else(|| {
        eprintln!("wget: missing URL");
        std::process::exit(1);
    });

    let output_path = resolve_output_path(&args, url);

    if !args.quiet {
        println!("--2026-07-18 12:00:00--  {}", url);
    }

    let client = Client::new();

    if !args.quiet {
        println!("Connecting to {}...", url);
    }

    let response = match client.get(url).send() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("wget: connection failed: {}", e);
            std::process::exit(1);
        }
    };

    let status = response.status();
    if !status.is_success() {
        eprintln!(
            "wget: HTTP error {}: {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("unknown")
        );
        std::process::exit(1);
    }

    let content = match response.bytes() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("wget: read error: {}", e);
            std::process::exit(1);
        }
    };

    if !args.quiet {
        println!(
            "HTTP request sent, awaiting response... {} OK",
            status.as_u16()
        );
        println!("Length: {} [binary]", content.len());
        println!("Saving to: '{}'", output_path.display());
        println!();
    }

    if let Err(e) = fs::write(&output_path, content) {
        eprintln!("wget: cannot write to {}: {}", output_path.display(), e);
        std::process::exit(1);
    }

    if !args.quiet {
        println!("{} saved", output_path.display());
    }
}

fn resolve_output_path(args: &cli::Args, url: &str) -> PathBuf {
    if let Some(o) = &args.output {
        return o.clone();
    }

    let filename = url.split('/').last().unwrap_or("index.html");
    let mut result = PathBuf::from(filename);

    if let Some(d) = &args.directory {
        result = d.join(result);
        if let Err(e) = fs::create_dir_all(d) {
            eprintln!("wget: cannot create directory {}: {}", d.display(), e);
            std::process::exit(1);
        }
    }

    result
}
