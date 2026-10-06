mod cli;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    let args = cli::Args::parse_or_exit();

    let path = if args.is_physical() {
        physical_path()
    } else {
        logical_path()
    };

    match path {
        Ok(p) => {
            println!("{}", format_path(&p));
        }
        Err(e) => {
            eprintln!("pwd: error retrieving current directory: {}", e);
            process::exit(1);
        }
    }
}

/// Resolve the physical (canonical) path, stripping all symlinks/junctions.
fn physical_path() -> Result<PathBuf, std::io::Error> {
    fs::canonicalize(".")
}

/// Resolve the logical path: prefer $PWD if it matches the real CWD.
fn logical_path() -> Result<PathBuf, std::io::Error> {
    let cwd = env::current_dir()?;

    if let Ok(pwd) = env::var("PWD") {
        let pwd_path = Path::new(&pwd);
        // Validate: both must canonicalize to the same physical directory
        if let (Ok(pwd_canon), Ok(cwd_canon)) = (fs::canonicalize(pwd_path), fs::canonicalize(&cwd))
            && pwd_canon == cwd_canon
        {
            return Ok(pwd_path.to_path_buf());
        }
    }

    Ok(cwd)
}

/// Format a path for display: normalize separators and strip Windows extended prefix.
fn format_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s.replace('\\', "/")
}
