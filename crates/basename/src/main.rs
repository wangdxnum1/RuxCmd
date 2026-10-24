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
    if let Some(name) = Path::new(path).file_name() {
        return name.to_string_lossy().to_string();
    }
    // No final component: trailing `.`/`..` segments and roots. POSIX strips
    // trailing separators first; the last non-empty segment is the basename.
    let trimmed = path.trim_end_matches(['/', '\\']);
    match trimmed.rsplit(['/', '\\']).next() {
        Some(last) if !last.is_empty() => last.to_string(),
        _ if trimmed.is_empty() && !path.is_empty() => "/".to_string(),
        _ => path.to_string(),
    }
}

fn remove_suffix(basename: &str, suffix: &str) -> String {
    if basename.ends_with(suffix) && basename.len() > suffix.len() {
        basename[..basename.len() - suffix.len()].to_string()
    } else {
        basename.to_string()
    }
}
