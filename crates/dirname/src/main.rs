mod cli;

use std::path::Path;

fn main() {
    let args = cli::Args::parse_or_exit();

    for path in &args.path {
        println!("{}", dirname(path));
    }
}

fn dirname(path: &str) -> String {
    let p = Path::new(path);

    if p.components().count() == 0 {
        return ".".to_string();
    }

    let parent = p.parent();

    match parent {
        Some(parent) => {
            if parent.components().count() == 0 {
                if p.is_absolute() {
                    let root = p
                        .components()
                        .next()
                        .unwrap()
                        .as_os_str()
                        .to_string_lossy()
                        .to_string();
                    root
                } else {
                    ".".to_string()
                }
            } else {
                format_path(parent)
            }
        }
        None => {
            if p.is_absolute() {
                let root = p
                    .components()
                    .next()
                    .unwrap()
                    .as_os_str()
                    .to_string_lossy()
                    .to_string();
                root
            } else {
                ".".to_string()
            }
        }
    }
}

fn format_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s.replace('\\', "/")
}
