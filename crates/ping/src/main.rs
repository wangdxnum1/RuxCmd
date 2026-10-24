mod cli;

use clap::Parser;
use std::net::{IpAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::NetworkManagement::IpHelper::{
    IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho,
};
use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;

/// Largest payload accepted for -s/--size. Beyond this the request would be
/// truncated by the u16 cast in IcmpSendEcho and fail with confusing errors;
/// 65500 is the Windows ping limit.
const MAX_SIZE: usize = 65500;

/// IP_STATUS code for a timed-out echo request, used when IcmpSendEcho
/// returns 0 without setting an extended error.
const IP_REQ_TIMED_OUT: u32 = 11010;

/// Set by the console control handler in -t (continuous) mode; the probe
/// loop checks it and shuts down gracefully.
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

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

    let data_size = args.size;
    if data_size > MAX_SIZE {
        eprintln!(
            "ping: invalid size {}: valid range is 0 to {}",
            data_size, MAX_SIZE
        );
        std::process::exit(1);
    }

    let timeout = Duration::from_millis(args.wait_ms);

    if args.continuous {
        // -t pings until stopped (Windows semantics). The handler keeps the
        // process alive so statistics can be printed on Ctrl+C; without -t
        // the default termination behavior is preserved.
        unsafe {
            SetConsoleCtrlHandler(Some(console_ctrl_handler), 1);
        }
    }

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

    let max_sends = if args.continuous {
        None
    } else {
        Some(args.count)
    };
    let mut probes = 0usize;
    loop {
        if let Some(limit) = max_sends
            && probes >= limit
        {
            break;
        }
        if SHUTDOWN.load(Ordering::SeqCst) {
            break;
        }
        probes += 1;
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

        let last = max_sends.is_some_and(|limit| probes >= limit);
        if !last {
            if args.continuous {
                // Sleep in slices so Ctrl+C takes effect promptly.
                let deadline = Instant::now() + Duration::from_secs(1);
                while Instant::now() < deadline && !SHUTDOWN.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(50));
                }
            } else {
                std::thread::sleep(Duration::from_secs(1));
            }
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

    // Windows ping semantics: exit 1 when every packet was lost, 0 when at
    // least one reply arrived (partial loss still exits 0).
    if received == 0 {
        std::process::exit(1);
    }
}

unsafe extern "system" fn console_ctrl_handler(_ctrl: u32) -> i32 {
    SHUTDOWN.store(true, Ordering::SeqCst);
    // TRUE: handled; keep the process alive so statistics get printed.
    1
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
                Err("Could not resolve".to_string())
            }
        }
        Err(_) => Err("Could not resolve".to_string()),
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
            data_size as u16, // bounded by MAX_SIZE (65500)
            std::ptr::null_mut(),
            reply.as_mut_ptr() as *mut std::ffi::c_void,
            reply_size as u32,
            timeout_ms,
        )
    };

    let elapsed = start.elapsed();

    if result == 0 {
        // IcmpSendEcho reports the IP_STATUS through GetLastError.
        let status = std::io::Error::last_os_error().raw_os_error().unwrap_or(0) as u32;
        let status = if status == 0 {
            IP_REQ_TIMED_OUT
        } else {
            status
        };
        Err(status_message(status))
    } else {
        let reply: &ICMP_ECHO_REPLY = unsafe { &*(reply.as_ptr() as *const ICMP_ECHO_REPLY) };
        if reply.Status == 0 {
            Ok(elapsed)
        } else {
            Err(status_message(reply.Status))
        }
    }
}

/// Map an IcmpSendEcho IP_STATUS code to the message Windows ping prints,
/// instead of reporting every failure as a timeout.
fn status_message(status: u32) -> String {
    match status {
        11002 => "Destination net unreachable.".to_string(), // IP_DEST_NET_UNREACHABLE
        11003 => "Destination host unreachable.".to_string(), // IP_DEST_HOST_UNREACHABLE
        11010 => "Request timed out.".to_string(),           // IP_REQ_TIMED_OUT
        11013 | 11014 => "TTL expired in transit.".to_string(), // IP_TTL_EXPIRED_*
        11050 => "General failure.".to_string(),             // IP_GENERAL_FAILURE
        other => format!("Request failed (status {}).", other),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_messages_match_windows_ping() {
        assert_eq!(status_message(11002), "Destination net unreachable.");
        assert_eq!(status_message(11003), "Destination host unreachable.");
        assert_eq!(status_message(11010), "Request timed out.");
        assert_eq!(status_message(11013), "TTL expired in transit.");
        assert_eq!(status_message(11014), "TTL expired in transit.");
        assert_eq!(status_message(11050), "General failure.");
    }

    #[test]
    fn unknown_status_is_reported_not_masked_as_timeout() {
        let message = status_message(99999);
        assert!(
            message.contains("99999"),
            "unknown status must be reported verbatim, got: {message}"
        );
        assert_ne!(message, "Request timed out.");
    }
}
