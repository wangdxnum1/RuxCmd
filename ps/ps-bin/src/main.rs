#![cfg(windows)]

mod cli;
mod render;

use std::io::Write;
use std::process::ExitCode;

use clap::Parser;
use ps_core::column::BoxCol;
use ps_core::filter::Selector;
use ps_core::{assemble, filter, sort, tree, CollectOptions};

use cli::Cli;
use render::{render_forest, render_table};

type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ps: error: {e}");
            ExitCode::from(2)
        }
    }
}

fn run() -> AppResult<()> {
    let mut cli = Cli::parse();

    if cli.list_columns {
        for (k, _) in ps_core::registry() {
            println!("{k}");
        }
        return Ok(());
    }

    let args: Vec<String> = std::env::args().collect();
    let has_u_flag = args.iter().any(|arg| arg == "-u" || (arg.starts_with('-') && arg.contains('u') && !arg.starts_with("--")));
    let is_aux = args.iter().any(|arg| arg == "-aux");
    let is_u_without_arg = has_u_flag && cli.user.is_empty();
    
    if is_aux || is_u_without_arg {
        cli.user_format = true;
        cli.select_all_tty = true;
        cli.include_no_tty = true;
        if is_aux && !cli.user.is_empty() {
            cli.user.clear();
        }
    }

    let cols: Vec<BoxCol> = resolve_columns(&cli)?;

    let opts = if cli.no_cmdline {
        CollectOptions::FAST
    } else {
        CollectOptions::FULL
    };
    let mut procs = assemble::snapshot_with(opts);

    if cli.select_all
        || cli.select_all_tty
        || cli.include_no_tty
        || !cli.pid.is_empty()
        || !cli.user.is_empty()
        || !cli.ruser.is_empty()
        || !cli.comm.is_empty()
    {
        let sel = Selector {
            pids: cli.pid.clone(),
            users: cli.user.clone(),
            group: Vec::new(),
            session: Vec::new(),
            comm_patterns: cli.comm.clone(),
            include_all: cli.select_all || cli.select_all_tty,
            include_no_ttys: cli.include_no_tty,
            explicit: true,
        };
        procs.retain(|p| filter::passes(p, &sel));
    }

    if procs.is_empty() {
        return Ok(());
    }

    let sort_spec = sort::parse_sort(&cli.sort);
    procs = sort::apply_sort(procs, &sort_spec, &cols);

    let mut out: Box<dyn Write> = match &cli.write {
        Some(p) => Box::new(std::fs::File::create(p)?),
        None => Box::new(std::io::stdout().lock()),
    };

    if cli.forest {
        let forest = tree::build_forest(procs);
        render_forest(&mut out, &forest, &cols, cli.no_headers, cli.ascii)?;
    } else {
        render_table(&mut out, &procs, &cols, cli.no_headers)?;
    }
    out.flush()?;
    Ok(())
}

fn resolve_columns(cli: &Cli) -> AppResult<Vec<BoxCol>> {
    let names = cli.columns_to_render();
    let mut cols = Vec::with_capacity(names.len());
    for n in &names {
        if let Some(f) = ps_core::lookup(n) {
            cols.push(f());
        } else {
            eprintln!("ps: warning: unknown column '{n}', skipped");
        }
    }
    if cols.is_empty() {
        return Err("no valid columns selected".into());
    }
    Ok(cols)
}
