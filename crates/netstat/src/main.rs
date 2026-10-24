mod cli;

use clap::Parser;
use std::net::Ipv4Addr;
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, GetTcpTable2, GetUdpTable, MIB_TCPROW_OWNER_PID,
    MIB_TCPROW2, MIB_UDPROW, MIB_UDPROW_OWNER_PID,
};
use windows_sys::Win32::Networking::WinSock::AF_INET;

const TCP_TABLE_OWNER_PID_ALL: u32 = 5;
const UDP_TABLE_OWNER_PID: u32 = 1;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("netstat 0.1.0");
        return;
    }

    let proto = args.protocol.as_deref().unwrap_or("all");

    let mut failed = false;

    println!("Active Connections");
    println!();
    if args.own_pid {
        println!("  Proto  Local Address          Foreign Address        State           PID");
    } else {
        println!("  Proto  Local Address          Foreign Address        State");
    }

    if proto == "all" || proto == "tcp" {
        failed |= print_tcp_connections(&args);
    }

    if proto == "all" || proto == "udp" {
        failed |= print_udp_connections(&args);
    }

    if failed {
        std::process::exit(1);
    }
}

type Row = (u32, u32, u32, u32, u32, Option<u32>);

fn print_row(
    args: &cli::Args,
    proto: &str,
    local: String,
    foreign: String,
    state: &str,
    pid: Option<u32>,
) {
    if args.own_pid {
        println!(
            "  {:<6} {:21} {:22} {:<15} {}",
            proto,
            local,
            foreign,
            state,
            pid.map(|p| p.to_string()).unwrap_or_else(|| "-".into())
        );
    } else {
        println!("  {:<6} {:21} {:22} {}", proto, local, foreign, state);
    }
}

fn print_tcp_connections(args: &cli::Args) -> bool {
    // With -o the PID table is fetched instead of the plain one.
    if args.own_pid {
        match get_tcp_owner_rows() {
            Ok(rows) => {
                for (local_addr, local_port, foreign_addr, foreign_port, state, pid) in rows {
                    if !args.all && state == 2 {
                        continue; // LISTENING: only with -a (Windows-style default)
                    }
                    let state_name = tcp_state_name(state);
                    print_row(
                        args,
                        "TCP",
                        format_socket_addr(local_addr, local_port),
                        format_socket_addr(foreign_addr, foreign_port),
                        state_name,
                        pid,
                    );
                }
                false
            }
            Err(e) => {
                eprintln!("netstat: {}", e);
                true
            }
        }
    } else {
        match get_tcp_connections() {
            Ok(connections) => {
                for conn in connections {
                    if !args.all && conn.state == 2 {
                        continue; // LISTENING: only with -a
                    }
                    print_row(
                        args,
                        "TCP",
                        format_socket_addr(conn.local_addr, conn.local_port),
                        format_socket_addr(conn.foreign_addr, conn.foreign_port),
                        tcp_state_name(conn.state),
                        None,
                    );
                }
                false
            }
            Err(e) => {
                eprintln!("netstat: {}", e);
                true
            }
        }
    }
}

fn tcp_state_name(state: u32) -> &'static str {
    match state {
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
    }
}

fn print_udp_connections(args: &cli::Args) -> bool {
    if args.own_pid {
        match get_udp_owner_rows() {
            Ok(rows) => {
                for (local_addr, local_port, pid) in rows {
                    if !args.all && local_port == 0 {
                        continue; // unbound sockets only with -a
                    }
                    print_row(
                        args,
                        "UDP",
                        format_socket_addr(local_addr, local_port),
                        "*:*".into(),
                        "",
                        Some(pid),
                    );
                }
                false
            }
            Err(e) => {
                eprintln!("netstat: {}", e);
                true
            }
        }
    } else {
        match get_udp_connections() {
            Ok(connections) => {
                for (local_addr, local_port) in connections {
                    if !args.all && local_port == 0 {
                        continue;
                    }
                    print_row(
                        args,
                        "UDP",
                        format_socket_addr(local_addr, local_port),
                        "*:*".into(),
                        "",
                        None,
                    );
                }
                false
            }
            Err(e) => {
                eprintln!("netstat: {}", e);
                true
            }
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

    Ok(entries
        .iter()
        .map(|entry| TcpConnection {
            local_addr: entry.dwLocalAddr,
            local_port: entry.dwLocalPort,
            foreign_addr: entry.dwRemoteAddr,
            foreign_port: entry.dwRemotePort,
            state: entry.dwState,
        })
        .collect())
}

/// (local_addr, local_port, foreign_addr, foreign_port, state, pid)
fn get_tcp_owner_rows() -> Result<Vec<Row>, String> {
    let mut buffer_size: u32 = 0;
    let result = unsafe {
        GetExtendedTcpTable(
            std::ptr::null_mut(),
            &mut buffer_size,
            0,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_ALL as i32,
            0,
        )
    };
    if result != 122 {
        return Err(format!("failed to get TCP table size: {}", result));
    }
    let mut buffer = vec![0u8; buffer_size as usize];
    let result = unsafe {
        GetExtendedTcpTable(
            buffer.as_mut_ptr() as *mut _,
            &mut buffer_size,
            0,
            AF_INET as u32,
            TCP_TABLE_OWNER_PID_ALL as i32,
            0,
        )
    };
    if result != 0 {
        return Err(format!("failed to get TCP table: {}", result));
    }
    #[repr(C)]
    struct Table {
        num_entries: u32,
        rows: [MIB_TCPROW_OWNER_PID; 1],
    }
    let table = unsafe { &*(buffer.as_ptr() as *const Table) };
    let entries =
        unsafe { std::slice::from_raw_parts(table.rows.as_ptr(), table.num_entries as usize) };
    Ok(entries
        .iter()
        .map(|r| {
            (
                r.dwLocalAddr,
                r.dwLocalPort,
                r.dwRemoteAddr,
                r.dwRemotePort,
                r.dwState,
                Some(r.dwOwningPid),
            )
        })
        .collect())
}

/// (local_addr, local_port, pid)
fn get_udp_owner_rows() -> Result<Vec<(u32, u32, u32)>, String> {
    let mut buffer_size: u32 = 0;
    let result = unsafe {
        GetExtendedUdpTable(
            std::ptr::null_mut(),
            &mut buffer_size,
            0,
            AF_INET as u32,
            UDP_TABLE_OWNER_PID as i32,
            0,
        )
    };
    if result != 122 {
        return Err(format!("failed to get UDP table size: {}", result));
    }
    let mut buffer = vec![0u8; buffer_size as usize];
    let result = unsafe {
        GetExtendedUdpTable(
            buffer.as_mut_ptr() as *mut _,
            &mut buffer_size,
            0,
            AF_INET as u32,
            UDP_TABLE_OWNER_PID as i32,
            0,
        )
    };
    if result != 0 {
        return Err(format!("failed to get UDP table: {}", result));
    }
    #[repr(C)]
    struct Table {
        num_entries: u32,
        rows: [MIB_UDPROW_OWNER_PID; 1],
    }
    let table = unsafe { &*(buffer.as_ptr() as *const Table) };
    let entries =
        unsafe { std::slice::from_raw_parts(table.rows.as_ptr(), table.num_entries as usize) };
    Ok(entries
        .iter()
        .map(|r| (r.dwLocalAddr, r.dwLocalPort, r.dwOwningPid))
        .collect())
}

fn get_udp_connections() -> Result<Vec<(u32, u32)>, String> {
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

    Ok(entries
        .iter()
        .map(|entry| (entry.dwLocalAddr, entry.dwLocalPort))
        .collect())
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
