mod cli;

use clap::Parser;
use std::net::{IpAddr, ToSocketAddrs};
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::NetworkManagement::IpHelper::{
    IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho,
};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("ping 0.1.0");
        return;
    }

    let host = args.host.as_deref().unwrap_or_else(|| {
        eprintln!("ping: missing host");
        std::process::exit(1);
    });
    let count = args.count;
    let timeout = Duration::from_secs(args.timeout);
    let data_size = args.size;

    let ip = match resolve_host(host) {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!("ping: {}: {}", e, host);
            std::process::exit(1);
        }
    };

    let ip_addr = match ip {
        IpAddr::V4(ipv4) => u32::from(ipv4).to_be(),
        IpAddr::V6(_) => {
            eprintln!("ping: IPv6 not supported");
            std::process::exit(1);
        }
    };

    println!(
        "Pinging {} [{}] with {} bytes of data:",
        host, ip, data_size
    );

    let icmp_handle = unsafe { IcmpCreateFile() };
    if icmp_handle == INVALID_HANDLE_VALUE {
        eprintln!("ping: unable to create ICMP handle");
        std::process::exit(1);
    }

    let mut sent = 0;
    let mut received = 0;
    let mut min_ms = u64::MAX;
    let mut max_ms = 0;
    let mut total_ms = 0;

    for i in 0..count {
        sent += 1;

        match send_icmp_echo(icmp_handle, ip_addr, data_size, timeout) {
            Ok(elapsed) => {
                received += 1;
                let ms = elapsed.as_millis() as u64;
                min_ms = min_ms.min(ms);
                max_ms = max_ms.max(ms);
                total_ms += ms;
                println!("Reply from {}: bytes={} time={}ms", ip, data_size, ms);
            }
            Err(e) => {
                println!("{}", e);
            }
        }

        if i < count - 1 {
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    unsafe {
        IcmpCloseHandle(icmp_handle);
    }

    println!();
    println!("Ping statistics for {}:", ip);
    println!(
        "    Packets: Sent = {}, Received = {}, Lost = {} ({:.0}% loss)",
        sent,
        received,
        sent - received,
        if sent > 0 {
            (sent - received) as f64 / sent as f64 * 100.0
        } else {
            0.0
        }
    );

    if received > 0 {
        println!("Approximate round trip times in milli-seconds:");
        println!(
            "    Minimum = {}ms, Maximum = {}ms, Average = {}ms",
            min_ms,
            max_ms,
            total_ms / received
        );
    }
}

fn resolve_host(host: &str) -> Result<IpAddr, String> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ip);
    }
    match (host, 0).to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                Ok(addr.ip())
            } else {
                Err(format!("Could not resolve"))
            }
        }
        Err(_) => Err(format!("Could not resolve")),
    }
}

fn send_icmp_echo(
    icmp_handle: *mut std::ffi::c_void,
    ip_addr: u32,
    data_size: usize,
    timeout: Duration,
) -> Result<Duration, String> {
    let data = vec![0u8; data_size];
    let reply_size = std::mem::size_of::<ICMP_ECHO_REPLY>() + data_size;
    let mut reply = vec![0u8; reply_size];

    let start = Instant::now();

    let timeout_ms = timeout.as_millis() as u32;

    let result = unsafe {
        IcmpSendEcho(
            icmp_handle,
            ip_addr,
            data.as_ptr() as *const std::ffi::c_void,
            data_size as u16,
            std::ptr::null_mut(),
            reply.as_mut_ptr() as *mut std::ffi::c_void,
            reply_size as u32,
            timeout_ms,
        )
    };

    let elapsed = start.elapsed();

    if result == 0 {
        Err("Request timed out.".to_string())
    } else {
        let reply: &ICMP_ECHO_REPLY = unsafe { &*(reply.as_ptr() as *const ICMP_ECHO_REPLY) };
        if reply.Status == 0 {
            Ok(elapsed)
        } else {
            Err(format!("Request timed out."))
        }
    }
}

#[repr(C)]
struct ICMP_ECHO_REPLY {
    Address: u32,
    Status: u32,
    RoundTripTime: u32,
    DataSize: u16,
    Reserved: u16,
    Data: *mut u8,
    Options: IP_OPTION_INFORMATION,
}

#[repr(C)]
struct IP_OPTION_INFORMATION {
    Ttl: u8,
    Tos: u8,
    Flags: u8,
    OptionsSize: u8,
    OptionsData: *mut u8,
}
