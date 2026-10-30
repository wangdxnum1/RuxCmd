use clap::Parser;

/// Check path name validity - a Windows port of Linux pathchk (subset).
///
/// Default checks: no empty components, no component longer than 255 bytes,
/// total path no longer than 4096 bytes. With -p, components must also use
/// only portable POSIX filename characters.
#[derive(Parser, Debug)]
#[command(name = "pathchk", version, about)]
pub struct Args {
    /// Check for POSIX portability (portable character set, 255-byte names)
    #[arg(short = 'p', long = "portability")]
    pub portability: bool,

    /// Additionally reject empty path names and leading '-'
    #[arg(short = 'P', long = "all")]
    pub all: bool,

    /// Paths to check
    #[arg(value_name = "PATH", allow_hyphen_values = true)]
    pub paths: Vec<String>,
}

fn main() {
    let args = Args::parse();
    if args.paths.is_empty() {
        eprintln!("pathchk: missing operand");
        std::process::exit(1);
    }

    let mut failed = false;
    for path in &args.paths {
        if let Err(message) = check(path, args.portability, args.all) {
            eprintln!("pathchk: '{}': {}", path, message);
            failed = true;
        }
    }
    std::process::exit(if failed { 1 } else { 0 });
}

fn check(path: &str, portability: bool, all: bool) -> Result<(), String> {
    if path.is_empty() {
        return Err(if all {
            "empty path name".to_string()
        } else {
            "contains empty component".to_string()
        });
    }

    if all && path.starts_with('-') {
        return Err("leading '-' in path name".to_string());
    }

    let total = path.len();
    if total > 4096 {
        return Err(format!("name contains {} bytes, exceeding 4096", total));
    }

    // A Windows drive prefix (C:) is structural, not a name component.
    let body = if path.len() >= 2
        && path.as_bytes()[1] == b':'
        && path.as_bytes()[0].is_ascii_alphabetic()
    {
        &path[2..]
    } else {
        path
    };

    for component in body.split(['/', '\\']) {
        if component.is_empty() {
            continue;
        }
        if component.len() > 255 {
            return Err(format!(
                "name '{}' has length {} bytes, exceeding 255",
                component,
                component.len()
            ));
        }
        if portability && !component.chars().all(is_portable) {
            return Err(format!(
                "name '{}' contains characters outside the portable filename set",
                component
            ));
        }
    }
    Ok(())
}

fn is_portable(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-'
}
