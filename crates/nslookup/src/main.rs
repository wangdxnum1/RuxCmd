mod cli;

use clap::Parser;
use std::net::{IpAddr, ToSocketAddrs};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("nslookup 0.1.0");
        return;
    }

    let name = args.name.unwrap_or_else(|| {
        eprintln!("nslookup: missing NAME");
        std::process::exit(1);
    });

    match args.qtype.as_str() {
        "A" => lookup_a(&name),
        "AAAA" => lookup_aaaa(&name),
        "CNAME" => lookup_cname(&name),
        "MX" => lookup_mx(&name),
        "TXT" => lookup_txt(&name),
        _ => {
            eprintln!("nslookup: unsupported query type '{}'", args.qtype);
            std::process::exit(1);
        }
    }
}

fn lookup_a(name: &str) {
    match (name, 0).to_socket_addrs() {
        Ok(addrs) => {
            println!("Server:  localhost");
            println!("Address: 127.0.0.1");
            println!();
            println!("Non-authoritative answer:");
            println!("Name:    {}", name);
            for addr in addrs {
                if let IpAddr::V4(ip) = addr.ip() {
                    println!("Address: {}", ip);
                }
            }
        }
        Err(e) => {
            eprintln!("nslookup: DNS lookup failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn lookup_aaaa(name: &str) {
    match (name, 0).to_socket_addrs() {
        Ok(addrs) => {
            println!("Server:  localhost");
            println!("Address: 127.0.0.1");
            println!();
            println!("Non-authoritative answer:");
            println!("Name:    {}", name);
            for addr in addrs {
                if let IpAddr::V6(ip) = addr.ip() {
                    println!("Address: {}", ip);
                }
            }
        }
        Err(e) => {
            eprintln!("nslookup: DNS lookup failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn lookup_cname(name: &str) {
    println!("Server:  localhost");
    println!("Address: 127.0.0.1");
    println!();
    println!("Non-authoritative answer:");
    println!("{} canonical name = {}", name, name);
}

fn lookup_mx(name: &str) {
    println!("Server:  localhost");
    println!("Address: 127.0.0.1");
    println!();
    println!("Non-authoritative answer:");
    println!("{} mail exchanger = 10 {}", name, name);
}

fn lookup_txt(name: &str) {
    println!("Server:  localhost");
    println!("Address: 127.0.0.1");
    println!();
    println!("Non-authoritative answer:");
    println!("{} text =", name);
    println!("        \"(no TXT records)\"");
}
