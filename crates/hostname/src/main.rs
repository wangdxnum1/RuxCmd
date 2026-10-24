mod cli;

use clap::Parser;
use cli::Cli;
use std::env;
use std::process;

fn main() {
    let cli = Cli::parse();

    let result = if cli.ip_address {
        get_ip_address().map_err(|_| "cannot resolve IP address".to_string())
    } else if cli.fqdn {
        get_fqdn().map_err(|_| "cannot resolve fully qualified domain name".to_string())
    } else {
        get_hostname().map_err(|_| "cannot determine hostname".to_string())
    };

    match result {
        Ok(value) => println!("{}", value),
        Err(msg) => {
            eprintln!("hostname: {}", msg);
            process::exit(1);
        }
    }
}

fn get_hostname() -> Result<String, ()> {
    #[cfg(windows)]
    {
        env::var("COMPUTERNAME").map_err(|_| ())
    }
    #[cfg(not(windows))]
    {
        env::var("HOSTNAME").map_err(|_| ())
    }
}

fn get_ip_address() -> Result<String, ()> {
    #[cfg(windows)]
    {
        use std::process::Command;
        let output = Command::new("powershell")
            .arg("-Command")
            .arg("(Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.InterfaceAlias -notlike 'Loopback*' }).IPAddress | Select-Object -First 1")
            .output()
            .map_err(|_| ())?;
        if !output.status.success() {
            return Err(());
        }
        let ip = String::from_utf8_lossy(&output.stdout);
        let ip = ip.trim();
        if ip.is_empty() {
            return Err(());
        }
        Ok(ip.to_string())
    }
    #[cfg(not(windows))]
    {
        use std::process::Command;
        let output = Command::new("hostname")
            .arg("-I")
            .output()
            .map_err(|_| ())?;
        if !output.status.success() {
            return Err(());
        }
        let ip = String::from_utf8_lossy(&output.stdout);
        let ip = ip.trim().split_whitespace().next().ok_or(())?;
        Ok(ip.to_string())
    }
}

fn get_fqdn() -> Result<String, ()> {
    #[cfg(windows)]
    {
        use std::process::Command;
        let output = Command::new("powershell")
            .arg("-Command")
            .arg("[System.Net.Dns]::GetHostEntry($env:COMPUTERNAME).HostName")
            .output()
            .map_err(|_| ())?;
        if !output.status.success() {
            return Err(());
        }
        let fqdn = String::from_utf8_lossy(&output.stdout);
        let fqdn = fqdn.trim();
        if fqdn.is_empty() {
            return Err(());
        }
        Ok(fqdn.to_string())
    }
    #[cfg(not(windows))]
    {
        use std::process::Command;
        let output = Command::new("hostname")
            .arg("-f")
            .output()
            .map_err(|_| ())?;
        if !output.status.success() {
            return Err(());
        }
        let fqdn = String::from_utf8_lossy(&output.stdout);
        let fqdn = fqdn.trim();
        if fqdn.is_empty() {
            return Err(());
        }
        Ok(fqdn.to_string())
    }
}
