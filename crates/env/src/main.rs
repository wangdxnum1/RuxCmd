use clap::Parser;
use std::env;
use std::process::{exit, Command};

mod cli;

fn main() {
    let args = cli::Cli::parse();

    // Split the operands into leading NAME=VALUE assignments and the command.
    let mut assignments: Vec<(String, String)> = Vec::new();
    let mut rest: &[String] = &args.command;
    for arg in &args.command {
        match split_assignment(arg) {
            Some((name, value)) => {
                assignments.push((name, value));
                rest = &rest[1..];
            }
            None => break,
        }
    }

    if args.verbose && args.ignore_environment {
        eprintln!("env: ignoring system environment");
    }

    if rest.is_empty() {
        print_environment(&args, &assignments);
        return;
    }

    let mut cmd = Command::new(&rest[0]);
    if args.ignore_environment {
        cmd.env_clear();
    }
    for name in &args.unset {
        cmd.env_remove(name);
    }
    for (name, value) in &assignments {
        cmd.env(name, value);
    }
    cmd.args(&rest[1..]);

    match cmd.status() {
        Ok(status) => exit(status.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("env: '{}': {}", rest[0], e);
            exit(127);
        }
    }
}

/// `NAME=VALUE` with a non-empty name; anything else (including values that
/// start with '-') starts the command instead.
fn split_assignment(arg: &str) -> Option<(String, String)> {
    if arg.starts_with('-') {
        return None;
    }
    let (name, value) = arg.split_once('=')?;
    if name.is_empty() {
        return None;
    }
    Some((name.to_string(), value.to_string()))
}

fn print_environment(args: &cli::Cli, assignments: &[(String, String)]) {
    let mut pairs: Vec<(String, String)> = if args.ignore_environment {
        Vec::new()
    } else {
        env::vars().collect()
    };

    for name in &args.unset {
        if args.verbose {
            eprintln!("env: removing variable: {}", name);
        }
        pairs.retain(|(k, _)| k != name);
    }

    for (name, value) in assignments {
        if let Some(pair) = pairs.iter_mut().find(|(k, _)| k == name) {
            pair.1 = value.clone();
        } else {
            pairs.push((name.clone(), value.clone()));
        }
    }

    for (key, value) in pairs {
        println!("{}={}", key, value);
    }
}
