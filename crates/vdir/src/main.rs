//! vdir — a Windows port of the GNU vdir command.
//!
//! vdir is ls with a long listing format by default: it delegates to the
//! ls crate's engine with an implicit `-l`, plus any user arguments.

use std::ffi::OsString;

fn main() {
    let mut args: Vec<OsString> = Vec::with_capacity(2);
    args.push(OsString::from("vdir")); // argv[0] placeholder, skipped by clap
    args.push(OsString::from("-l")); // vdir == ls -l by default
    args.extend(std::env::args_os().skip(1));
    std::process::exit(ls::run(args));
}
