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
        eprintln!("groups: identity information is not available on this platform");
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn run_windows(args: &cli::Args) {
    let identity = match token::current_identity() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("groups: {e}");
            std::process::exit(1);
        }
    };

    // GNU `groups [USER]` queries another user; on Windows we can only read the
    // current process token, so any other user is rejected explicitly.
    if let Some(requested) = &args.user
        && !is_current_user(requested, &identity)
    {
        eprintln!("groups: cannot query other users on windows");
        std::process::exit(1);
    }

    let names: Vec<String> = identity
        .groups
        .iter()
        .map(|e| match &e.name {
            Some(n) => n.clone(),
            None => e.rid.to_string(),
        })
        .collect();
    println!("{}", names.join(" "));
}

/// Case-insensitive match against the token account name (with optional
/// DOMAIN\ prefix). When the token account name cannot be resolved we cannot
/// confirm the request refers to the current user, so it is rejected.
#[cfg(windows)]
fn is_current_user(requested: &str, identity: &token::Identity) -> bool {
    let Some(name) = &identity.user.name else {
        return false;
    };
    let requested = requested.to_lowercase();
    let name = name.to_lowercase();
    if identity.domain.is_empty() {
        return requested == name;
    }
    let full = format!("{}\\{}", identity.domain.to_lowercase(), name);
    requested == name || requested == full
}
