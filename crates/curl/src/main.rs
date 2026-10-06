mod cli;

use clap::Parser;
use reqwest::blocking::Client;
use std::fs;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("curl 0.1.0");
        return;
    }

    let url = args.url.as_deref().unwrap_or_else(|| {
        eprintln!("curl: missing URL");
        std::process::exit(1);
    });

    let client = Client::new();

    let mut request = match args.method.as_str().to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        _ => {
            eprintln!("curl: unsupported method '{}'", args.method);
            std::process::exit(1);
        }
    };

    for header in &args.headers {
        let parts: Vec<&str> = header.splitn(2, ':').collect();
        if parts.len() == 2 {
            request = request.header(parts[0].trim(), parts[1].trim());
        }
    }

    if let Some(data) = &args.data {
        request = request.body(data.clone());
    }

    let response = match request.send() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("curl: connection failed: {}", e);
            std::process::exit(1);
        }
    };

    let status = response.status();

    if !args.silent {
        println!(
            "HTTP/1.1 {} {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        );
        for (key, value) in response.headers() {
            println!("{}: {}", key, value.to_str().unwrap_or(""));
        }
        println!();
    }

    let body = match response.text() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("curl: read error: {}", e);
            std::process::exit(1);
        }
    };

    if let Some(output) = &args.output {
        if let Err(e) = fs::write(output, &body) {
            eprintln!("curl: cannot write to {}: {}", output.display(), e);
            std::process::exit(1);
        }
    } else {
        print!("{}", body);
    }
}
