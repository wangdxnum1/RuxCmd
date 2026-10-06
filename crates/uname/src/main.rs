mod cli;

use clap::Parser;
use cli::Cli;
use std::env::consts;
use std::process::Command;

fn main() {
    let cli = Cli::parse();

    let kernel_name = get_kernel_name();
    let nodename = get_nodename();
    let kernel_release = get_kernel_release();
    let kernel_version = get_kernel_version();
    let machine = get_machine();
    let operating_system = get_operating_system();

    let output = if cli.all {
        format!(
            "{} {} {} {} {} {}",
            kernel_name, nodename, kernel_release, kernel_version, machine, operating_system
        )
    } else if cli.has_no_flags() {
        kernel_name
    } else {
        let mut parts = Vec::new();
        if cli.kernel_name {
            parts.push(kernel_name);
        }
        if cli.nodename {
            parts.push(nodename);
        }
        if cli.kernel_release {
            parts.push(kernel_release);
        }
        if cli.kernel_version {
            parts.push(kernel_version);
        }
        if cli.machine {
            parts.push(machine);
        }
        if cli.operating_system {
            parts.push(operating_system);
        }
        parts.join(" ")
    };

    println!("{}", output);
}

fn get_kernel_name() -> String {
    #[cfg(windows)]
    {
        "Windows_NT".to_string()
    }
    #[cfg(not(windows))]
    {
        consts::OS.to_string()
    }
}

fn get_nodename() -> String {
    #[cfg(windows)]
    {
        let output = Command::new("hostname").output().ok();
        if let Some(out) = output {
            let name = String::from_utf8_lossy(&out.stdout);
            name.trim().to_string()
        } else {
            "unknown".to_string()
        }
    }
    #[cfg(not(windows))]
    {
        let output = Command::new("hostname").output().ok();
        if let Some(out) = output {
            let name = String::from_utf8_lossy(&out.stdout);
            name.trim().to_string()
        } else {
            "unknown".to_string()
        }
    }
}

fn get_kernel_release() -> String {
    #[cfg(windows)]
    {
        let output = Command::new("cmd").arg("/c").arg("ver").output().ok();
        if let Some(out) = output {
            let ver = String::from_utf8_lossy(&out.stdout);
            if let Some(start) = ver.find('[') {
                if let Some(end) = ver.find(']') {
                    let version_str = &ver[start + 1..end];
                    let parts: Vec<&str> = version_str.split_whitespace().collect();
                    if let Some(version) = parts.last() {
                        return version.to_string();
                    }
                }
            }
            "10.0".to_string()
        } else {
            "10.0".to_string()
        }
    }
    #[cfg(not(windows))]
    {
        consts::OS.to_string()
    }
}

fn get_kernel_version() -> String {
    #[cfg(windows)]
    {
        let output = Command::new("powershell")
            .arg("-Command")
            .arg("(Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion').CurrentBuild")
            .output()
            .ok();
        if let Some(out) = output {
            let build = String::from_utf8_lossy(&out.stdout);
            let build = build.trim();
            if !build.is_empty() {
                return format!("Build {}", build);
            }
        }
        "unknown".to_string()
    }
    #[cfg(not(windows))]
    {
        "unknown".to_string()
    }
}

fn get_machine() -> String {
    consts::ARCH.to_string()
}

fn get_operating_system() -> String {
    #[cfg(windows)]
    {
        let output = Command::new("cmd").arg("/c").arg("ver").output().ok();
        if let Some(out) = output {
            let ver = String::from_utf8_lossy(&out.stdout);
            let parts: Vec<&str> = ver.split_whitespace().collect();
            if parts.len() >= 2 {
                parts[1].to_string()
            } else {
                "Windows".to_string()
            }
        } else {
            "Windows".to_string()
        }
    }
    #[cfg(not(windows))]
    {
        consts::OS.to_string()
    }
}
