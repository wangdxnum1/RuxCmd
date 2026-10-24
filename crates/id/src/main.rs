mod cli;

#[cfg(windows)]
mod token;

use clap::Parser;

fn main() {
    let args = cli::Args::parse();

    #[cfg(windows)]
    run_windows(&args);

    #[cfg(not(windows))]
    {
        let _ = args;
        eprintln!("id: identity information is not available on this platform");
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn run_windows(args: &cli::Args) {
    let mut only_flags = 0;
    if args.user {
        only_flags += 1;
    }
    if args.group {
        only_flags += 1;
    }
    if args.groups {
        only_flags += 1;
    }
    if only_flags > 1 {
        eprintln!("id: cannot print \"only\" of more than one choice");
        std::process::exit(1);
    }
    if args.name && only_flags == 0 {
        eprintln!("id: cannot print only names or real IDs in default format");
        std::process::exit(1);
    }

    let identity = match token::current_identity() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("id: {e}");
            std::process::exit(1);
        }
    };

    if args.user {
        println!("{}", one(&identity.user, args.name));
    } else if args.group {
        println!("{}", one(&identity.primary_group, args.name));
    } else if args.groups {
        let parts: Vec<String> = identity.groups.iter().map(|e| one(e, args.name)).collect();
        println!("{}", parts.join(" "));
    } else {
        let groups: Vec<String> = identity.groups.iter().map(|e| e.display()).collect();
        println!(
            "uid={} gid={} groups={}",
            identity.user.display(),
            identity.primary_group.display(),
            groups.join(",")
        );
    }
}

/// With `-n` print the account name; otherwise the numeric RID. Falls back to
/// the RID when no name can be resolved (GNU does the same).
#[cfg(windows)]
fn one(entry: &token::Entry, name: bool) -> String {
    match (name, &entry.name) {
        (true, Some(n)) => n.clone(),
        _ => entry.rid.to_string(),
    }
}
