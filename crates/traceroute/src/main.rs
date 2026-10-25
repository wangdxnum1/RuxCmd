mod cli;

use clap::{CommandFactory, Parser};
use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};
use std::time::Duration;
use windows_sys::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    ICMP_ECHO_REPLY, IP_DEST_HOST_UNREACHABLE, IP_DEST_NET_UNREACHABLE, IP_DEST_PORT_UNREACHABLE,
    IP_DEST_PROT_UNREACHABLE, IP_OPTION_INFORMATION, IP_REQ_TIMED_OUT, IP_TTL_EXPIRED_REASSEM,
    IP_TTL_EXPIRED_TRANSIT, IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho,
};

/// Payload bytes carried by each probe, matching tracert.exe's default.
const PROBE_SIZE: usize = 32;

/// Number of consecutive hops where every probe times out before the trace
/// gives up early instead of walking all -m hops.
const MAX_SILENT_HOPS: usize = 3;

/// Outcome of a single probe.
enum Probe {
    /// A reply arrived: `addr` is the responder (a router on TTL expiry, or
    /// the target itself), `rtt_ms` its reported round trip time, `arrived`
    /// whether the reply came from the target with a success status.
    Reply {
        addr: u32,
        rtt_ms: u32,
        arrived: bool,
    },
    /// No answer within the per-probe timeout.
    TimedOut,
    /// The destination reported itself unreachable; tracing ends after this.
    DestUnreachable(&'static str),
    /// Any other failure, printed verbatim in the probe column.
    Failed(&'static str),
}

/// Result of probing one TTL value.
struct Hop {
    /// One display cell per probe: "N ms", `*`, or a status label.
    probes: Vec<String>,
    /// Address of the first responder on this hop, if any.
    addr: Option<String>,
    /// The target answered; the trace is complete.
    arrived: bool,
    /// Every probe timed out; feeds the silent-hop early exit.
    all_timed_out: bool,
    /// The destination reported itself unreachable; tracing ends.
    dest_unreachable: bool,
}

fn main() {
    // Clap errors (missing host, bad values) must exit 1 rather than clap's
    // default 2; --help/--version still exit 0.
    let args = match cli::Args::try_parse() {
        Ok(args) => args,
        Err(e) => {
            let code = if e.use_stderr() { 1 } else { 0 };
            let _ = e.print();
            std::process::exit(code);
        }
    };

    if args.version {
        println!("traceroute 0.1.0");
        return;
    }

    // host stays optional so `-v` parses without it; a missing host then
    // reports usage and exits 1 instead of clap's default exit code 2.
    let Some(host) = args.host.as_deref() else {
        eprintln!("{}", cli::Args::command().render_usage());
        std::process::exit(1);
    };

    let target = match resolve_ipv4(host) {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!("traceroute: {}: {}", e, host);
            std::process::exit(1);
        }
    };
    // Network-byte-order form for the ICMP API and reply comparisons.
    let target_net = u32::from(target).to_be();
    let timeout = Duration::from_millis(args.timeout_ms);

    println!(
        "Tracing route to {} [{}] over a maximum of {} hops:",
        host, target, args.max_hops
    );
    println!();

    let icmp_handle: HANDLE = unsafe { IcmpCreateFile() };
    if icmp_handle == INVALID_HANDLE_VALUE {
        eprintln!("traceroute: unable to create ICMP handle");
        std::process::exit(1);
    }

    let mut consecutive_silent = 0usize;
    for ttl in 1..=args.max_hops {
        let hop = probe_hop(icmp_handle, target_net, ttl, args.nqueries, timeout);
        print_hop(ttl, &hop);

        if hop.arrived || hop.dest_unreachable {
            break;
        }
        if hop.all_timed_out {
            consecutive_silent += 1;
            if consecutive_silent >= MAX_SILENT_HOPS {
                break;
            }
        } else {
            consecutive_silent = 0;
        }
    }

    unsafe {
        IcmpCloseHandle(icmp_handle);
    }

    println!();
    println!("Trace complete.");
}

/// Resolve a host to IPv4, preferring A records when a name has several
/// addresses. IPv6 targets are rejected: only the -4 family is supported.
fn resolve_ipv4(host: &str) -> Result<Ipv4Addr, String> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return match ip {
            IpAddr::V4(v4) => Ok(v4),
            IpAddr::V6(_) => Err("IPv6 not supported".to_string()),
        };
    }
    match (host, 0).to_socket_addrs() {
        Ok(addrs) => {
            let mut only_v6 = false;
            for addr in addrs {
                match addr.ip() {
                    IpAddr::V4(v4) => return Ok(v4),
                    IpAddr::V6(_) => only_v6 = true,
                }
            }
            if only_v6 {
                Err("IPv6 not supported".to_string())
            } else {
                Err("Could not resolve".to_string())
            }
        }
        Err(_) => Err("Could not resolve".to_string()),
    }
}

/// Send one ICMP echo with the hop's TTL in the IP options.
fn send_probe(icmp_handle: HANDLE, target_net: u32, ttl: u32, timeout: Duration) -> Probe {
    let options = IP_OPTION_INFORMATION {
        Ttl: ttl as u8,
        Tos: 0,
        Flags: 0,
        OptionsSize: 0,
        OptionsData: std::ptr::null_mut(),
    };
    let data = [0u8; PROBE_SIZE];
    let reply_size = (std::mem::size_of::<ICMP_ECHO_REPLY>() + PROBE_SIZE + 8) as u32;
    let mut reply_buf = vec![0u8; reply_size as usize];

    let replies = unsafe {
        IcmpSendEcho(
            icmp_handle,
            target_net,
            data.as_ptr() as *const std::ffi::c_void,
            data.len() as u16,
            &options,
            reply_buf.as_mut_ptr() as *mut std::ffi::c_void,
            reply_size,
            timeout.as_millis() as u32,
        )
    };

    let reply = unsafe { &*(reply_buf.as_ptr() as *const ICMP_ECHO_REPLY) };

    if replies == 0 {
        // IcmpSendEcho reports the IP_STATUS through GetLastError; on TTL
        // expiry the buffer still carries the responder's address.
        let status = std::io::Error::last_os_error().raw_os_error().unwrap_or(0) as u32;
        classify(
            if status == 0 {
                IP_REQ_TIMED_OUT
            } else {
                status
            },
            reply,
            target_net,
        )
    } else {
        classify(reply.Status, reply, target_net)
    }
}

fn classify(status: u32, reply: &ICMP_ECHO_REPLY, target_net: u32) -> Probe {
    match status {
        0 => Probe::Reply {
            addr: reply.Address,
            rtt_ms: reply.RoundTripTime,
            arrived: reply.Address == target_net,
        },
        IP_TTL_EXPIRED_TRANSIT | IP_TTL_EXPIRED_REASSEM => {
            if reply.Address == 0 {
                Probe::Failed("TTL expired in transit.")
            } else {
                Probe::Reply {
                    addr: reply.Address,
                    rtt_ms: reply.RoundTripTime,
                    arrived: false,
                }
            }
        }
        IP_REQ_TIMED_OUT => Probe::TimedOut,
        IP_DEST_NET_UNREACHABLE
        | IP_DEST_HOST_UNREACHABLE
        | IP_DEST_PROT_UNREACHABLE
        | IP_DEST_PORT_UNREACHABLE => Probe::DestUnreachable(status_label(status)),
        _ => Probe::Failed(status_label(status)),
    }
}

fn status_label(status: u32) -> &'static str {
    match status {
        IP_DEST_NET_UNREACHABLE => "Destination net unreachable.",
        IP_DEST_HOST_UNREACHABLE => "Destination host unreachable.",
        IP_DEST_PROT_UNREACHABLE => "Destination protocol unreachable.",
        IP_DEST_PORT_UNREACHABLE => "Destination port unreachable.",
        IP_TTL_EXPIRED_TRANSIT | IP_TTL_EXPIRED_REASSEM => "TTL expired in transit.",
        _ => "Request failed.",
    }
}

/// Probe one TTL value `-q` times and fold the probes into a hop line.
fn probe_hop(
    icmp_handle: HANDLE,
    target_net: u32,
    ttl: u32,
    nqueries: u32,
    timeout: Duration,
) -> Hop {
    let mut probes = Vec::new();
    let mut addr: Option<u32> = None;
    let mut arrived = false;
    let mut dest_unreachable = false;
    let mut all_timed_out = true;

    for _ in 0..nqueries {
        match send_probe(icmp_handle, target_net, ttl, timeout) {
            Probe::Reply {
                addr: from,
                rtt_ms,
                arrived: is_target,
            } => {
                all_timed_out = false;
                if addr.is_none() {
                    addr = Some(from);
                }
                probes.push(format!("{} ms", rtt_ms));
                if is_target {
                    arrived = true;
                }
            }
            Probe::TimedOut => probes.push("*".to_string()),
            Probe::DestUnreachable(label) => {
                all_timed_out = false;
                dest_unreachable = true;
                probes.push(label.to_string());
            }
            Probe::Failed(label) => {
                all_timed_out = false;
                probes.push(label.to_string());
            }
        }
    }

    Hop {
        probes,
        // reply.Address shares the API's network-byte-order convention with
        // target_net, so the same to_be() round trip restores the octets.
        addr: addr.map(|a| Ipv4Addr::from(a.to_be()).to_string()),
        arrived,
        all_timed_out,
        dest_unreachable,
    }
}

fn print_hop(ttl: u32, hop: &Hop) {
    match &hop.addr {
        Some(addr) => println!("  {:>2}  {:<15}  {}", ttl, addr, hop.probes.join("  ")),
        None => println!("  {:>2}  {}", ttl, hop.probes.join("  ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_labels_match_windows_tracert() {
        assert_eq!(
            status_label(IP_DEST_NET_UNREACHABLE),
            "Destination net unreachable."
        );
        assert_eq!(
            status_label(IP_DEST_HOST_UNREACHABLE),
            "Destination host unreachable."
        );
        assert_eq!(
            status_label(IP_DEST_PROT_UNREACHABLE),
            "Destination protocol unreachable."
        );
        assert_eq!(
            status_label(IP_DEST_PORT_UNREACHABLE),
            "Destination port unreachable."
        );
        assert_eq!(
            status_label(IP_TTL_EXPIRED_TRANSIT),
            "TTL expired in transit."
        );
        assert_eq!(
            status_label(IP_TTL_EXPIRED_REASSEM),
            "TTL expired in transit."
        );
        assert_eq!(status_label(11050), "Request failed.");
    }

    #[test]
    fn reply_addresses_are_rendered_in_network_byte_order() {
        // The ICMP API hands out addresses in network byte order (the same
        // convention as target_net); 0x0100007f must render as 127.0.0.1.
        let addr = Ipv4Addr::from(0x0100007fu32.to_be()).to_string();
        assert_eq!(addr, "127.0.0.1");
    }
}
