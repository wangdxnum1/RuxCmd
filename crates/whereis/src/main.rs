mod cli;

use clap::Parser;
use cli::Args;
use std::path::Path;

fn main() {
    let args = Args::parse();

    if args.commands.is_empty() {
        eprintln!("whereis: missing operand");
        eprintln!("Try 'whereis --help' for more information.");
        std::process::exit(1);
    }

    // GNU whereis searches a fixed list of system directories; on Windows
    // the PATH entries cover the same ground without hardcoded paths.
    let path_env = std::env::var("PATH").unwrap_or_default();
    let dirs: Vec<&str> = path_env.split(';').collect();

    // util-linux whereis exits 0 when at least one requested command had
    // any match, 1 when none did.
    let mut any_found = false;

    for cmd in &args.commands {
        let mut hits: Vec<String> = Vec::new();
        if args.binaries || (!args.binaries && !args.manuals && !args.sources) {
            for dir in &dirs {
                for ext in ["", ".exe"] {
                    let candidate = Path::new(dir).join(format!("{}{}", cmd, ext));
                    if candidate.is_file() {
                        let shown = ruxcmd_core::paths::display(&candidate);
                        if !hits.contains(&shown) {
                            hits.push(shown);
                        }
                    }
                }
            }
        }
        // -m (manuals) and -s (sources) have no standard Windows locations,
        // so those sections report nothing, like a Linux host without docs.

        print!("{}:", cmd);
        for hit in &hits {
            print!(" {}", hit);
        }
        println!();
        if !hits.is_empty() {
            any_found = true;
        }
    }

    std::process::exit(if any_found { 0 } else { 1 });
}
