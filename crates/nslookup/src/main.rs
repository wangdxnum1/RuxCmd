mod cli;

use clap::Parser;
use std::net::{IpAddr, ToSocketAddrs};
use windows::Win32::NetworkManagement::Dns::{
    DNS_QUERY_STANDARD, DNS_RECORDA, DNS_RECORDW, DNS_TYPE, DNS_TYPE_CNAME, DNS_TYPE_MX,
    DNS_TYPE_TEXT, DnsFree, DnsFreeRecordList, DnsQuery_W,
};
use windows::core::{PCWSTR, PWSTR};

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

    // Record type names are case-insensitive: `-t a` and `-t A` are equivalent.
    match args.qtype.to_ascii_uppercase().as_str() {
        "A" => lookup_a(&name),
        "AAAA" => lookup_aaaa(&name),
        "CNAME" => lookup_record(&name, DNS_TYPE_CNAME, "CNAME"),
        "MX" => lookup_record(&name, DNS_TYPE_MX, "MX"),
        "TXT" => lookup_record(&name, DNS_TYPE_TEXT, "TXT"),
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

/// Answer data extracted from a DNS_RECORDW of the requested type.
struct DnsAnswer {
    lines: Vec<String>,
}

fn lookup_record(name: &str, qtype: DNS_TYPE, type_label: &str) {
    match dns_query(name, qtype) {
        Ok(answers) if !answers.is_empty() => {
            println!("Non-authoritative answer:");
            for answer in answers {
                for line in answer.lines {
                    println!("{}", line);
                }
            }
        }
        Ok(_) => {
            eprintln!("nslookup: no {} records found for '{}'", type_label, name);
            std::process::exit(1);
        }
        Err(status) => {
            eprintln!(
                "nslookup: DNS query for '{}' ({}) failed: {}",
                name,
                type_label,
                dns_status_message(status)
            );
            std::process::exit(1);
        }
    }
}

/// Query the system resolver via dnsapi's DnsQuery_W and collect the records
/// of the requested type as owned strings.
fn dns_query(name: &str, qtype: DNS_TYPE) -> Result<Vec<DnsAnswer>, u32> {
    let mut wide: Vec<u16> = name.encode_utf16().collect();
    wide.push(0);

    // windows-0.58 types the out-slot as *mut DNS_RECORDA because the SDK
    // aliases PDNS_RECORD to PDNS_RECORDA; DnsQuery_W actually hands back
    // wide DNS_RECORDW entries with a layout identical to DNS_RECORDA, so the
    // slot is reinterpreted below.
    let mut results: *mut DNS_RECORDA = std::ptr::null_mut();
    let status = unsafe {
        DnsQuery_W(
            PCWSTR(wide.as_ptr()),
            qtype,
            DNS_QUERY_STANDARD,
            None,
            &mut results,
            None,
        )
    };
    if status.0 != 0 {
        return Err(status.0);
    }

    let mut answers = Vec::new();
    let mut record: *mut DNS_RECORDW = results.cast();
    while !record.is_null() {
        let entry = unsafe { &*record };
        if entry.wType == qtype.0 {
            let owner = wide_string(entry.pName);
            let lines = match qtype {
                DNS_TYPE_MX => {
                    let mx = unsafe { entry.Data.Mx };
                    vec![format!(
                        "{} mail exchanger = {} {}",
                        owner,
                        mx.wPreference,
                        wide_string(mx.pNameExchange)
                    )]
                }
                DNS_TYPE_TEXT => {
                    let txt = unsafe { entry.Data.Txt };
                    let strings = unsafe {
                        std::slice::from_raw_parts(
                            txt.pStringArray.as_ptr(),
                            txt.dwStringCount as usize,
                        )
                    };
                    strings
                        .iter()
                        .map(|s| format!("{} text = \"{}\"", owner, wide_string(*s)))
                        .collect()
                }
                DNS_TYPE_CNAME => {
                    let cname = unsafe { entry.Data.Cname };
                    vec![format!(
                        "{} canonical name = {}",
                        owner,
                        wide_string(cname.pNameHost)
                    )]
                }
                _ => Vec::new(),
            };
            answers.push(DnsAnswer { lines });
        }
        record = entry.pNext;
    }

    unsafe { DnsFree(Some(results.cast()), DnsFreeRecordList) };
    Ok(answers)
}

/// Decode a NUL-terminated UTF-16 string from the resolver.
fn wide_string(p: PWSTR) -> String {
    unsafe { p.to_string() }.unwrap_or_default()
}

/// English description for the DNS_STATUS codes the resolver commonly
/// returns (FormatMessage output would be locale-dependent).
fn dns_status_message(status: u32) -> String {
    let text = match status {
        146 => "DNS query timed out",                    // ERROR_TIMEOUT
        9001 => "DNS server unable to interpret format", // DNS_ERROR_RCODE_FORMAT_ERROR
        9002 => "DNS server failure",                    // DNS_ERROR_RCODE_SERVER_FAILURE
        9003 => "DNS name does not exist",               // DNS_ERROR_RCODE_NAME_ERROR
        9004 => "DNS query type not implemented",        // DNS_ERROR_RCODE_NOT_IMPLEMENTED
        9005 => "DNS query refused",                     // DNS_ERROR_RCODE_REFUSED
        9501 => "no records of the requested type",      // DNS_INFO_NO_RECORDS
        9852 => "no DNS servers configured",             // DNS_ERROR_NO_DNS_SERVERS
        _ => return format!("status {}", status),
    };
    format!("{} (status {})", text, status)
}
