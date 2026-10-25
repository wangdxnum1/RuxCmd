mod cli;

use chrono::{DateTime, Local};
use clap::Parser;
use cli::Args;
use std::io::{self, Write};
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Screen home + clear, matching the sequence clear/reset emit; `\x1B[3J`
/// also drops the scrollback buffer so every iteration starts from scratch.
const CLEAR_SCREEN: &str = "\x1B[H\x1B[2J\x1B[3J";

/// Assumed terminal width used to right-justify the clock in the header. The
/// real width is not queried because the output is often redirected.
const SCREEN_WIDTH: usize = 80;

fn main() {
    let args = Args::parse();

    if args.command.is_empty() {
        eprintln!("watch: missing command");
        eprintln!("Try 'watch --help' for more information.");
        std::process::exit(1);
    }

    let interval = match parse_interval(&args.interval) {
        Ok(seconds) => seconds,
        Err(raw) => {
            eprintln!("watch: invalid time interval '{}'", raw);
            std::process::exit(1);
        }
    };

    let program = args.command[0].clone();
    let command_text = args.command.join(" ");
    // Previous iteration's rendered lines (for -d) and raw text (for -g).
    let mut previous: Option<(Vec<String>, String)> = None;

    loop {
        let (text, code, note) = run_once(&program, &args.command[1..]);
        let lines: Vec<&str> = text.lines().collect();
        let first = previous.is_none();

        let mut screen = String::from(CLEAR_SCREEN);
        if !args.no_title {
            screen.push_str(&title_line(interval, &command_text, Local::now()));
            screen.push('\n');
            screen.push_str(&"-".repeat(SCREEN_WIDTH));
            screen.push('\n');
        }
        screen.push_str(&render_output(
            &lines,
            previous.as_ref().map(|(lines, _)| lines.as_slice()),
            args.differences,
        ));
        if let Some(note) = &note {
            screen.push_str(note);
            screen.push('\n');
        }
        write_screen(&screen);

        // -e: end the loop with the command's own status. procps watch exits
        // with the command status too, after waiting for a key press that a
        // headless port cannot block on.
        if args.errexit {
            let exit_code = code.unwrap_or(127); // 0x7f: the command could not be started
            if exit_code != 0 {
                std::process::exit(exit_code);
            }
        }
        // -g: the first run is the baseline; a later run whose output differs
        // from the previous one ends the loop successfully.
        if args.chgexit && !first {
            let (_, prev_text) = previous.as_ref().expect("first run tracked");
            if prev_text != &text {
                std::process::exit(0);
            }
        }

        previous = Some((lines.iter().map(|line| (*line).to_string()).collect(), text));
        thread::sleep(Duration::from_secs_f64(interval));
    }
}

/// Writes one screenful to stdout. Like GNU yes, a closed downstream pipe is
/// a normal exit (`watch ... | head` must not panic on a broken pipe); any
/// other write error is reported and ends the run.
fn write_screen(screen: &str) {
    let mut out = io::stdout().lock();
    if let Err(e) = out.write_all(screen.as_bytes()).and_then(|()| out.flush()) {
        if e.kind() == io::ErrorKind::BrokenPipe {
            std::process::exit(0);
        }
        eprintln!("watch: {e}");
        std::process::exit(1);
    }
}

/// Runs the command once, capturing stdout and stderr instead of inheriting
/// the terminal: the output is re-rendered every iteration and -d/-g need a
/// stable copy to compare against. Returns the combined output (CRLF folded
/// to LF), the exit code (None when the command could not be started) and the
/// note shown in the content area on failure. Neither failure kind ends the
/// loop: like procps watch, the error is displayed and the cycle continues.
fn run_once(program: &str, args: &[String]) -> (String, Option<i32>, Option<String>) {
    let output = match Command::new(program).args(args).output() {
        Ok(output) => output,
        Err(err) => {
            return (
                String::new(),
                None,
                Some(format!("watch: failed to run '{program}': {err}")),
            );
        }
    };
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    let text = text.replace("\r\n", "\n");
    let code = output.status.code();
    let note = code
        .filter(|&code| code != 0)
        .map(|code| format!("watch: command exited with status {code}"));
    (text, code, note)
}

/// Renders the captured output line by line. With `-d`, every line that is
/// new or differs from the previous run is wrapped in reverse video
/// (`\x1B[7m ... \x1B[0m`). The first run is the baseline and is never
/// highlighted, like procps watch which only compares against the previous
/// screen.
fn render_output(lines: &[&str], previous: Option<&[String]>, differences: bool) -> String {
    let mut body = String::new();
    for (i, line) in lines.iter().enumerate() {
        let changed = match (differences, previous) {
            (true, Some(prev)) => prev.get(i).is_none_or(|old| old.as_str() != *line),
            _ => false,
        };
        if changed {
            body.push_str("\x1B[7m");
        }
        body.push_str(line);
        if changed {
            body.push_str("\x1B[0m");
        }
        body.push('\n');
    }
    body
}

/// Builds the header line: `Every Ns: COMMAND` followed by the local clock
/// (`%a %b %d %H:%M:%S %Y`, procps style). The clock is right-justified on an
/// 80-column line when everything fits; otherwise it simply follows the
/// command so that neither the command nor the clock is ever cut off (unlike
/// ncurses watch, the real terminal width is unknown when output is
/// redirected, and clipping would hide the command text).
fn title_line(interval: f64, command: &str, now: DateTime<Local>) -> String {
    let right = now.format("%a %b %d %H:%M:%S %Y").to_string();
    let left = format!("Every {interval:.1}s: {command}");
    let pad = SCREEN_WIDTH.saturating_sub(left.chars().count() + right.chars().count());
    if pad >= 1 {
        format!("{left}{}{right}", " ".repeat(pad))
    } else {
        format!("{left} {right}")
    }
}

/// Parses `-n SECONDS` as a plain decimal number of seconds. procps watch
/// reads the value with a strict strtod (no unit suffixes) and clamps it into
/// [0.1s, 31 days]; both behaviors are kept here. Returns the offending input
/// on failure.
fn parse_interval(raw: &str) -> Result<f64, String> {
    let seconds: f64 = raw.parse().map_err(|_| raw.to_string())?;
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(raw.to_string());
    }
    Ok(seconds.clamp(0.1, 31.0 * 24.0 * 60.0 * 60.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn parses_fractional_seconds() {
        assert_eq!(parse_interval("2"), Ok(2.0));
        assert_eq!(parse_interval("0.1"), Ok(0.1));
        assert_eq!(parse_interval("1.5"), Ok(1.5));
        assert_eq!(parse_interval("10"), Ok(10.0));
    }

    #[test]
    fn clamps_like_procps() {
        assert_eq!(parse_interval("0"), Ok(0.1));
        assert_eq!(parse_interval("0.01"), Ok(0.1));
        assert_eq!(parse_interval("1e12"), Ok(31.0 * 24.0 * 60.0 * 60.0));
    }

    #[test]
    fn rejects_non_numeric_negative_and_suffixes() {
        for bad in ["", "abc", "2s", "-1", "inf", "nan", "1.2.3"] {
            assert_eq!(parse_interval(bad), Err(bad.to_string()), "input {bad:?}");
        }
    }

    #[test]
    fn title_right_justifies_the_clock() {
        let now = Local.with_ymd_and_hms(2026, 10, 24, 12, 34, 56).unwrap();
        let line = title_line(2.0, "date", now);
        assert!(line.starts_with("Every 2.0s: date"));
        assert!(line.ends_with("Sat Oct 24 12:34:56 2026"));
        assert_eq!(line.chars().count(), SCREEN_WIDTH);
    }

    #[test]
    fn title_keeps_command_and_clock_when_they_do_not_fit() {
        let now = Local.with_ymd_and_hms(2026, 10, 24, 12, 34, 56).unwrap();
        let long = "x".repeat(200);
        let line = title_line(0.1, &long, now);
        assert!(line.starts_with("Every 0.1s: "));
        assert!(line.contains(&long));
        assert!(line.ends_with("Sat Oct 24 12:34:56 2026"));
        assert!(!line.contains("..."));
    }

    #[test]
    fn differences_highlight_only_changed_lines() {
        let previous: Vec<String> = vec!["alpha".into(), "beta".into()];
        let out = render_output(&["alpha", "BETA", "gamma"], Some(&previous), true);
        assert_eq!(out, "alpha\n\x1B[7mBETA\x1B[0m\n\x1B[7mgamma\x1B[0m\n");
    }

    #[test]
    fn first_run_is_the_baseline_without_highlight() {
        let out = render_output(&["alpha", "beta"], None, true);
        assert_eq!(out, "alpha\nbeta\n");
        // -d off never highlights either.
        let previous: Vec<String> = vec!["other".into()];
        let out = render_output(&["alpha"], Some(&previous), false);
        assert_eq!(out, "alpha\n");
    }

    #[test]
    fn output_is_rendered_with_normalized_line_endings() {
        // run_once needs a real command; check the CRLF fold directly.
        let text = "hello\r\nworld\r\n".replace("\r\n", "\n");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(render_output(&lines, None, false), "hello\nworld\n");
    }
}
