mod cli;
mod color;
mod display;
mod entry;
mod perm;
mod sort;

fn main() {
    let args = cli::Args::parse_or_exit();
    let paths = if args.files.is_empty() {
        vec![std::path::PathBuf::from(".")]
    } else {
        args.files.clone()
    };

    let mut exit_code = 0i32;
    let show_total = paths.len() > 1 && args.long;
    let has_multiple_sources = paths.len() > 1;

    for (idx, path) in paths.iter().enumerate() {
        match entry::collect_entries(path, &args) {
            Ok(entries) => {
                if has_multiple_sources {
                    if idx > 0 {
                        println!();
                    }
                    println!("{}:", path.display());
                }
                if show_total {
                    let total_blocks: u64 = entries
                        .iter()
                        .filter(|e| !e.name.starts_with('.') || args.all)
                        .map(|e| e.block_count() * 512 / 1024)
                        .sum();
                    println!("total {}", total_blocks);
                }
                display::print_entries(&entries, &args);
            }
            Err(e) => {
                eprintln!("ls: cannot access '{}': {}", path.display(), e);
                exit_code = 2;
            }
        }
    }

    std::process::exit(exit_code);
}
