mod cli;

use std::path::{Component, Path};

fn main() {
    let args = cli::Args::parse_or_exit();

    for path in &args.path {
        print!("{}{}", dirname(path), if args.zero { '\0' } else { '\n' });
    }
}

fn dirname(path: &str) -> String {
    let p = Path::new(path);

    if let Some(root) = single_root_component(p) {
        return root;
    }

    match p.parent() {
        Some(parent) => {
            if parent.as_os_str().is_empty() {
                ".".to_string()
            } else {
                format_path(parent)
            }
        }
        None => format_path(p),
    }
}

// Root-only forms: "/" and "//" print "/" (Windows does not report them as
// absolute, so the parent-based logic would lose them); a bare drive prefix
// ("C:") has no directory part and falls back to ".".
fn single_root_component(p: &Path) -> Option<String> {
    let mut comps = p.components();
    let first = comps.next()?;
    if comps.next().is_some() {
        return None;
    }
    match first {
        Component::RootDir => Some("/".to_string()),
        Component::Prefix(_) => Some(".".to_string()),
        _ => None,
    }
}

fn format_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s.replace('\\', "/")
}
