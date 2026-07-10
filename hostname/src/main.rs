mod cli;

use clap::Parser;
use cli::Cli;
use std::env;

fn main() {
    let cli = Cli::parse();

    if cli.ip_address {
        if let Some(ip) = get_ip_address() {
            println!("{}", ip);
        }
    } else if cli.fqdn {
        if let Some(fqdn) = get_fqdn() {
            println!("{}", fqdn);
        }
    } else if cli.has_no_flags() {
        if let Some(hostname) = get_hostname() {
            println!("{}", hostname);
        }
    }
}

fn get_hostname() -> Option<String> {
    #[cfg(windows)]
    {
        env::var("COMPUTERNAME").ok()
    }
    #[cfg(not(windows))]
    {
        env::var("HOSTNAME").ok()
    }
}

fn get_ip_address() -> Option<String> {
    #[cfg(windows)]
    {
        use std::process::Command;
        let output = Command::new("powershell")
            .arg("-Command")
            .arg("(Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.InterfaceAlias -notlike 'Loopback*' }).IPAddress | Select-Object -First 1")
            .output()
            .ok()?;
        let ip = String::from_utf8_lossy(&output.stdout);
        let ip = ip.trim();
        if !ip.is_empty() {
            Some(ip.to_string())
        } else {
            None
        }
    }
    #[cfg(not(windows))]
    {
        use std::process::Command;
        let output = Command::new("hostname")
            .arg("-I")
            .output()
            .ok()?;
        let ip = String::from_utf8_lossy(&output.stdout);
        let ip = ip.trim().split_whitespace().next()?;
        Some(ip.to_string())
    }
}

fn get_fqdn() -> Option<String> {
    #[cfg(windows)]
    {
        use std::process::Command;
        let output = Command::new("powershell")
            .arg("-Command")
            .arg("[System.Net.Dns]::GetHostEntry($env:COMPUTERNAME).HostName")
            .output()
            .ok()?;
        let fqdn = String::from_utf8_lossy(&output.stdout);
        let fqdn = fqdn.trim();
        if !fqdn.is_empty() {
            Some(fqdn.to_string())
        } else {
            None
        }
    }
    #[cfg(not(windows))]
    {
        use std::process::Command;
        let output = Command::new("hostname")
            .arg("-f")
            .output()
            .ok()?;
        let fqdn = String::from_utf8_lossy(&output.stdout);
        let fqdn = fqdn.trim();
        if !fqdn.is_empty() {
            Some(fqdn.to_string())
        } else {
            None
        }
    }
}
