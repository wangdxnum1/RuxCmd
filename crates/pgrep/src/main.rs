mod cli;

use clap::Parser;
use cli::Args;
use ps_sys::{collect_with, CollectOptions};

fn main() {
    let args = Args::parse();
    let self_pid = std::process::id();

    let opts = if args.full {
        CollectOptions::FULL
    } else {
        CollectOptions::FAST
    };
    let procs = match collect_with(opts) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("pgrep: cannot enumerate processes: {}", e);
            std::process::exit(1);
        }
    };

    let mut matched = 0usize;
    for p in &procs {
        if p.pid == self_pid {
            continue;
        }
        if !matches(&args, p) {
            continue;
        }
        matched += 1;
        if args.list_name {
            println!("{} {}", p.pid, p.name);
        } else {
            println!("{}", p.pid);
        }
    }

    if matched == 0 {
        std::process::exit(1);
    }
}

/// Match against the image name and (optionally) the full command line.
/// Image-name comparison is ASCII case-insensitive: Toolhelp32 reports its
/// own casing, which need not match the on-disk spelling. -f cmdline matching
/// stays case-sensitive unless -i.
fn matches(args: &Args, p: &ps_sys::ProcessRaw) -> bool {
    let pat = args.pattern.to_lowercase();
    let hay_name = p.name.to_lowercase();
    // Windows image names carry ".exe"; accept matches against both forms.
    let hay_stem = hay_name
        .strip_suffix(".exe")
        .unwrap_or(&hay_name)
        .to_string();

    let name_hit = if args.exact {
        pat == hay_name || pat == hay_stem
    } else {
        hay_name.contains(&pat) || hay_stem.contains(&pat)
    };
    if name_hit {
        return true;
    }
    if args.full {
        if let Some(cmd) = &p.cmdline {
            let hay_cmd = if args.ignore_case {
                cmd.to_lowercase()
            } else {
                cmd.clone()
            };
            return hay_cmd.contains(&args.pattern);
        }
    }
    false
}
