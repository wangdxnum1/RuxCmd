mod cli;

use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process;

fn main() {
    let args = cli::Args::parse_or_exit();

    let path = Path::new(&args.path);

    let result =
        fs::canonicalize(path).and_then(|canon| strip_components(canon, args.strip_components));

    match result {
        Ok(p) => {
            println!("{}", strip_verbatim(&p));
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

    // Prefix (drive letter / UNC) and RootDir are structural parts of the
    // path, not name components: filter them out, then strip `count` leading
    // components from what remains.
    let mut result = PathBuf::new();
    let mut available = 0usize;
    for comp in path.components() {
        match comp {
            Component::Prefix(_) | Component::RootDir => {}
            _ => {
                available += 1;
                if available > count {
                    result.push(comp);
                }
            }
        }
    }

    if available <= count {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("cannot strip {} components from path", count),
        ));
    }

    Ok(result)
}

/// Strip the `\\?\` verbatim prefix (mapping `\\?\UNC\` to `//`) and normalize
/// backslashes to forward slashes so the output is directly reusable.
fn strip_verbatim(path: &Path) -> String {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        return format!("//{}", rest.replace('\\', "/"));
    }
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s.replace('\\', "/")
}
