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
            println!("{}", ruxcmd_core::paths::display(&p));
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
