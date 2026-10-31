use clap::Parser;
use std::path::Path;

/// List contents of directories in a tree-like format.
#[derive(Parser, Debug)]
#[command(name = "tree", version, about)]
pub struct Args {
    /// Show hidden files
    #[arg(short = 'a')]
    pub all: bool,

    /// Descend only level deep
    #[arg(short = 'L', long = "level", value_name = "LEVEL")]
    pub level: Option<usize>,

    /// Use ASCII characters instead of Unicode
    #[arg(long = "charset", value_name = "CHARSET", default_value = "unicode")]
    pub charset: String,

    /// Directory to list (default: current)
    #[arg(value_name = "DIRECTORY")]
    pub directory: Option<String>,
}

fn main() {
    let args = Args::parse();
    let root = args.directory.clone().unwrap_or_else(|| ".".to_string());
    let max_depth = args.level.unwrap_or(usize::MAX);
    let ascii = args.charset == "ascii";

    let root_path = Path::new(&root);
    if !root_path.is_dir() {
        eprintln!("tree: {}: Not a directory", root);
        std::process::exit(1);
    }

    println!("{}", root);
    let mut dirs = 0usize;
    let mut files = 0usize;
    list_dir(
        root_path, "", 1, max_depth, args.all, ascii, &mut dirs, &mut files,
    );
    println!("\n{} directories, {} files", dirs, files);
}

fn list_dir(
    dir: &Path,
    prefix: &str,
    depth: usize,
    max_depth: usize,
    all: bool,
    ascii: bool,
    dirs: &mut usize,
    files: &mut usize,
) {
    if depth > max_depth {
        return;
    }
    let (branch, last_branch, pipe, end) = if ascii {
        ("|-- ", "`-- ", "|   ", "    ")
    } else {
        ("├── ", "└── ", "│   ", "    ")
    };

    let entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| {
                if all {
                    return true;
                }
                !e.file_name().to_string_lossy().starts_with('.')
            })
            .collect(),
        Err(e) => {
            eprintln!("tree: cannot read {}: {}", dir.display(), e);
            return;
        }
    };

    let mut sorted: Vec<_> = entries;
    sorted.sort_by(|a, b| {
        let a_dir = a.path().is_dir();
        let b_dir = b.path().is_dir();
        b_dir.cmp(&a_dir).then(a.file_name().cmp(&b.file_name()))
    });

    let count = sorted.len();
    for (idx, entry) in sorted.iter().enumerate() {
        let is_last = idx == count - 1;
        let connector = if is_last { last_branch } else { branch };
        let name = entry.file_name().to_string_lossy().into_owned();
        println!("{}{}{}", prefix, connector, name);

        let child_prefix = if is_last {
            format!("{}{}", prefix, end)
        } else {
            format!("{}{}", prefix, pipe)
        };

        if entry.path().is_dir() {
            *dirs += 1;
            list_dir(
                &entry.path(),
                &child_prefix,
                depth + 1,
                max_depth,
                all,
                ascii,
                dirs,
                files,
            );
        } else {
            *files += 1;
        }
    }
}
