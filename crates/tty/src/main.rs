mod cli;

use clap::Parser;
use std::io::{self, IsTerminal};

/// Placeholder terminal device name for stdin, printed by `tty -a` (see
/// docs/compatibility.md — Windows has no /dev/tty equivalent).
const CONIN_NAME: &str = "CONIN$";
/// Placeholder terminal device name for stdout/stderr, printed by `tty -a`.
const CONOUT_NAME: &str = "CONOUT$";
const NOT_A_TTY: &str = "not a tty";

/// GNU tty reports success iff the stream it inspects (stdin) is a terminal.
fn stdin_exit_code(stdin_is_tty: bool) -> i32 {
    if stdin_is_tty {
        0
    } else {
        1
    }
}

/// `tty -a` succeeds when any of the three standard streams is a terminal.
fn all_streams_exit_code(streams_are_tty: [bool; 3]) -> i32 {
    if streams_are_tty.iter().any(|&t| t) {
        0
    } else {
        1
    }
}

/// Per-stream label for `tty -a` output (placeholder device names, see
/// docs/compatibility.md).
fn stream_label(is_tty: bool, is_input: bool) -> &'static str {
    if is_tty {
        if is_input {
            CONIN_NAME
        } else {
            CONOUT_NAME
        }
    } else {
        NOT_A_TTY
    }
}

fn run() -> i32 {
    let args = cli::Args::parse();

    if args.all {
        let streams = [
            io::stdin().is_terminal(),
            io::stdout().is_terminal(),
            io::stderr().is_terminal(),
        ];
        if !args.silent {
            for (i, tty) in streams.iter().enumerate() {
                println!("{}", stream_label(*tty, i == 0));
            }
        }
        return all_streams_exit_code(streams);
    }

    let stdin_is_tty = io::stdin().is_terminal();
    if !args.silent {
        println!("{}", stream_label(stdin_is_tty, true));
    }
    stdin_exit_code(stdin_is_tty)
}

fn main() {
    std::process::exit(run());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdin_being_a_tty_yields_success_exit_code() {
        // "stdin is a tty -> exit 0" cannot run under CI (no tty there), so the
        // decision logic is extracted and asserted here instead.
        assert_eq!(stdin_exit_code(true), 0);
    }

    #[test]
    fn non_tty_stdin_yields_failure_exit_code() {
        assert_eq!(stdin_exit_code(false), 1);
    }

    #[test]
    fn all_streams_exit_code_succeeds_if_any_stream_is_a_tty() {
        assert_eq!(all_streams_exit_code([true, true, true]), 0);
        assert_eq!(all_streams_exit_code([true, false, false]), 0);
        assert_eq!(all_streams_exit_code([false, true, false]), 0);
        assert_eq!(all_streams_exit_code([false, false, true]), 0);
        assert_eq!(all_streams_exit_code([false, false, false]), 1);
    }

    #[test]
    fn stream_labels_use_console_device_placeholders() {
        assert_eq!(stream_label(true, true), "CONIN$");
        assert_eq!(stream_label(true, false), "CONOUT$");
        assert_eq!(stream_label(false, true), "not a tty");
        assert_eq!(stream_label(false, false), "not a tty");
    }
}
