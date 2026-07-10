mod cli;

use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    let args = cli::Args::parse_or_exit();

    let path = Path::new(&args.path);

    let result = fs::canonicalize(path)
        .and_then(|canon| strip_components(canon, args.strip_components));

    match result {
        Ok(p) => {
            println!("{}", format_path(&p));
        }
        Err(e) => {
            eprintln!("realpath: {}: {}", path.display(), e);
            process::exit(1);
        }
    }
}

fn strip_components(path: PathBuf, count: usize) -> Result<PathBuf, std::io::Error> {
    if count == 0 {
        return Ok(path);
    }

    let components: Vec<_> = path.components().collect();

    if components.len() <= count {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("cannot strip {} components from path", count),
        ));
    }

    let remaining: Vec<_> = components.into_iter().skip(count).collect();

    let mut result = PathBuf::new();
    for comp in remaining {
        result.push(comp);
    }

    Ok(result)
}

fn format_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s.replace('\\', "/")
}