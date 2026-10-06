mod cli;

use clap::Parser;
use std::net::Ipv4Addr;
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetTcpTable2, GetUdpTable, MIB_TCPROW2, MIB_UDPROW,
};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("netstat 0.1.0");
        return;
    }

    let proto = args.protocol.as_deref().unwrap_or("all");

    println!("Active Connections");
    println!();
    println!("  Proto  Local Address          Foreign Address        State");

    if proto == "all" || proto == "tcp" {
        print_tcp_connections();
    }

    if proto == "all" || proto == "udp" {
        print_udp_connections();
    }
}

fn print_tcp_connections() {
    match get_tcp_connections() {
        Ok(connections) => {
            for conn in connections {
                let state = match conn.state {
                    1 => "CLOSED",
                    2 => "LISTENING",
                    3 => "SYN_SENT",
                    4 => "SYN_RCVD",
                    5 => "ESTABLISHED",
                    6 => "FIN_WAIT1",
                    7 => "FIN_WAIT2",
                    8 => "CLOSE_WAIT",
                    9 => "CLOSING",
                    10 => "LAST_ACK",
                    11 => "TIME_WAIT",
                    12 => "DELETE_TCB",
                    _ => "UNKNOWN",
                };

                let local_addr = format_socket_addr(conn.local_addr, conn.local_port);
                let foreign_addr = format_socket_addr(conn.foreign_addr, conn.foreign_port);

                println!("  TCP    {:21} {:22} {}", local_addr, foreign_addr, state);
            }
        }
        Err(e) => {
            eprintln!("netstat: {}", e);
        }
    }
}

fn print_udp_connections() {
    match get_udp_connections() {
        Ok(connections) => {
            for conn in connections {
                let local_addr = format_socket_addr(conn.local_addr, conn.local_port);
                let foreign_addr = format_socket_addr(conn.foreign_addr, conn.foreign_port);

                println!("  UDP    {:21} {:22} *", local_addr, foreign_addr);
            }
        }
        Err(e) => {
            eprintln!("netstat: {}", e);
        }
    }
}

fn format_socket_addr(addr: u32, port: u32) -> String {
    let ip = Ipv4Addr::from(addr.to_be());
    let port = ((port >> 8) | (port << 8)) as u16;
    format!("{}:{}", ip, port)
}

struct TcpConnection {
    local_addr: u32,
    local_port: u32,
    foreign_addr: u32,
    foreign_port: u32,
    state: u32,
}

struct UdpConnection {
    local_addr: u32,
    local_port: u32,
    foreign_addr: u32,
    foreign_port: u32,
}

fn get_tcp_connections() -> Result<Vec<TcpConnection>, String> {
    let mut buffer_size: u32 = 0;

    let result = unsafe { GetTcpTable2(std::ptr::null_mut(), &mut buffer_size, 0) };

    if result != 122 {
        return Err(format!("failed to get TCP table size: {}", result));
    }

    let mut buffer = vec![0u8; buffer_size as usize];

    let result = unsafe { GetTcpTable2(buffer.as_mut_ptr() as *mut _, &mut buffer_size, 0) };

    if result != 0 {
        return Err(format!("failed to get TCP table: {}", result));
    }

    let table_ptr = buffer.as_ptr() as *const MIB_TCPTABLE2;
    let table = unsafe { &*table_ptr };
    let entries =
        unsafe { std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize) };

    let mut connections = Vec::new();

    for entry in entries {
        connections.push(TcpConnection {
            local_addr: entry.dwLocalAddr,
            local_port: entry.dwLocalPort,
            foreign_addr: entry.dwRemoteAddr,
            foreign_port: entry.dwRemotePort,
            state: entry.dwState,
        });
    }

    Ok(connections)
}

fn get_udp_connections() -> Result<Vec<UdpConnection>, String> {
    let mut buffer_size: u32 = 0;

    let result = unsafe { GetUdpTable(std::ptr::null_mut(), &mut buffer_size, 0) };

    if result != 122 {
        return Err(format!("failed to get UDP table size: {}", result));
    }

    let mut buffer = vec![0u8; buffer_size as usize];

    let result = unsafe { GetUdpTable(buffer.as_mut_ptr() as *mut _, &mut buffer_size, 0) };

    if result != 0 {
        return Err(format!("failed to get UDP table: {}", result));
    }

    let table_ptr = buffer.as_ptr() as *const MIB_UDPTABLE;
    let table = unsafe { &*table_ptr };
    let entries =
        unsafe { std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize) };

    let mut connections = Vec::new();

    for entry in entries {
        connections.push(UdpConnection {
            local_addr: entry.dwLocalAddr,
            local_port: entry.dwLocalPort,
            foreign_addr: 0,
            foreign_port: 0,
        });
    }

    Ok(connections)
}

#[repr(C)]
struct MIB_TCPTABLE2 {
    dwNumEntries: u32,
    table: [MIB_TCPROW2; 1],
}

#[repr(C)]
struct MIB_UDPTABLE {
    dwNumEntries: u32,
    table: [MIB_UDPROW; 1],
}
