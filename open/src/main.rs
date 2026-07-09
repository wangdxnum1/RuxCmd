mod cli;

use clap::Parser;
use std::fs;
use std::process::Command;
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    let paths = if args.paths.is_empty() {
        vec![std::path::PathBuf::from(".")]
    } else {
        args.paths.clone()
    };

    let mut exit_code = 0i32;

    for path in &paths {
        if let Err(e) = open_path(path, &args) {
            eprintln!("open: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

fn open_path(path: &Path, args: &cli::Args) -> Result<(), String> {
    let path_str = path.to_string_lossy();

    if args.reveal {
        return reveal_in_explorer(path);
    }

    let target_app = if args.edit {
        Some("notepad.exe".to_string())
    } else if let Some(app) = &args.application {
        Some(app.clone())
    } else {
        None
    };

    if let Some(app) = target_app {
        open_with_application(path, &app, args)
    } else if path_str.starts_with("http://") || path_str.starts_with("https://") || 
              path_str.starts_with("ftp://") || path_str.starts_with("file://") {
        open_url(path_str.as_ref())
    } else {
        open_default(path, args)
    }
}

fn open_default(path: &Path, args: &cli::Args) -> Result<(), String> {
    let path_str = path.to_string_lossy();
    let escaped_path = escape_path_for_powershell(&path_str);
    
    let mut cmd = if args.wait {
        Command::new("powershell")
    } else {
        Command::new("cmd")
    };

    if args.wait {
        cmd.arg("-Command")
           .arg(&format!("Start-Process -FilePath \"{}\" -Wait", escaped_path));
    } else {
        cmd.arg("/c")
           .arg("start")
           .arg("")
           .arg(&*path_str);
    }

    execute_command(cmd)
}

fn open_with_application(path: &Path, app: &str, args: &cli::Args) -> Result<(), String> {
    let path_str = path.to_string_lossy();
    let escaped_path = escape_path_for_powershell(&path_str);

    let mut cmd = if args.wait {
        Command::new("powershell")
    } else {
        Command::new("cmd")
    };

    if args.wait {
        cmd.arg("-Command")
           .arg(&format!("Start-Process -FilePath \"{}\" -ArgumentList \"{}\" -Wait", app, escaped_path));
    } else {
        cmd.arg("/c")
           .arg("start")
           .arg("")
           .arg(app)
           .arg(&*path_str);
    }

    execute_command(cmd)
}

fn open_url(url: &str) -> Result<(), String> {
    let mut cmd = Command::new("cmd");
    cmd.arg("/c")
       .arg("start")
       .arg("")
       .arg(url);

    execute_command(cmd)
}

fn reveal_in_explorer(path: &Path) -> Result<(), String> {
    let path_str = path.to_string_lossy();
    
    let is_dir = fs::metadata(path)
        .map(|m| m.is_dir())
        .unwrap_or(false);

    let mut cmd = Command::new("explorer.exe");
    if is_dir {
        cmd.arg(&*path_str);
    } else {
        cmd.arg("/select,")
           .arg(&*path_str);
    }

    execute_command(cmd)
}

fn escape_path_for_powershell(path: &str) -> String {
    path.replace('\'', "''")
}

fn execute_command(mut cmd: Command) -> Result<(), String> {
    let result = cmd.status();

    match result {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(format!("command exited with code {}", status.code().unwrap_or(-1)))
            }
        }
        Err(e) => {
            Err(format!("failed to execute command: {}", e))
        }
    }
}