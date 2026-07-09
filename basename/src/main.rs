mod cli;

use clap::Parser;
use cli::Cli;
use std::path::Path;

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("basename 1.0.0");
        return;
    }

    for name in &cli.names {
        let basename = get_basename(name);
        let result = if let Some(suffix) = &cli.suffix {
            remove_suffix(&basename, suffix)
        } else {
            basename
        };
        println!("{}", result);
    }
}

fn get_basename(path: &str) -> String {
    let p = Path::new(path);
    p.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

fn remove_suffix(basename: &str, suffix: &str) -> String {
    if basename.ends_with(suffix) && basename.len() > suffix.len() {
        basename[..basename.len() - suffix.len()].to_string()
    } else {
        basename.to_string()
    }
}
