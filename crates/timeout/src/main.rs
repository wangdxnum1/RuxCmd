mod cli;

use clap::Parser;
use cli::Args;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(50);

fn main() {
    let args = Args::parse();

    if args.command.is_empty() {
        eprintln!("timeout: missing command");
        std::process::exit(1);
    }

    let duration = match parse_duration(&args.duration) {
        Ok(d) => d,
        Err(bad) => {
            eprintln!("timeout: invalid time interval '{}'", bad);
            std::process::exit(1);
        }
    };

    let kill_after = match args.kill_after.as_deref() {
        Some(s) => match parse_duration(s) {
            Ok(d) => Some(d),
            Err(bad) => {
                eprintln!("timeout: invalid time interval '{}'", bad);
                std::process::exit(1);
            }
        },
        None => None,
    };

    let program = &args.command[0];
    let cmd_args: Vec<&str> = args.command[1..].iter().map(|s| s.as_str()).collect();

    let mut child = Command::new(program)
        .args(cmd_args)
        .spawn()
        .unwrap_or_else(|e| {
            eprintln!("timeout: failed to execute '{}': {}", program, e);
            std::process::exit(1);
        });

    let start = Instant::now();
    loop {
        match child.try_wait() {
            // The child finished before the deadline: pass its exit code through.
            Ok(Some(status)) => {
                std::process::exit(status.code().unwrap_or(1));
            }
            Ok(None) => {
                if start.elapsed() >= duration {
                    expire(&mut child, kill_after);
                }
            }
            Err(e) => {
                eprintln!("timeout: failed to wait for command: {}", e);
                std::process::exit(1);
            }
        }
        thread::sleep(POLL_INTERVAL);
    }
}

/// The deadline expired: kill the child and exit 124. With `-k`, give the child
/// `kill_after` more seconds to terminate after the first kill; if it is still
/// alive then, kill it again. Either way the final exit code is 124.
fn expire(child: &mut std::process::Child, kill_after: Option<Duration>) -> ! {
    let _ = child.kill();
    let extra_kill_deadline = kill_after.map(|kd| Instant::now() + kd);
    let mut killed_twice = false;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if let (Some(deadline), false) = (extra_kill_deadline, killed_twice) {
                    if Instant::now() >= deadline {
                        let _ = child.kill();
                        killed_twice = true;
                    }
                }
            }
            Err(_) => break,
        }
        thread::sleep(POLL_INTERVAL);
    }
    std::process::exit(124);
}

/// Parse a duration of the form `[0-9]+(\.[0-9]+)?[smhd]?` (default unit is
/// seconds). Returns the offending input string on failure instead of falling
/// back to a zero duration.
fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    let invalid = || s.to_string();
    if s.is_empty() {
        return Err(invalid());
    }
    let lowered = s.to_ascii_lowercase();
    let (num, multiplier) = match lowered.chars().last() {
        Some('s') => (&lowered[..lowered.len() - 1], 1.0),
        Some('m') => (&lowered[..lowered.len() - 1], 60.0),
        Some('h') => (&lowered[..lowered.len() - 1], 3600.0),
        Some('d') => (&lowered[..lowered.len() - 1], 86_400.0),
        _ => (&lowered[..], 1.0),
    };
    let seconds = parse_number(num).ok_or_else(invalid)? * multiplier;
    // Guard against values Duration::from_secs_f64 would panic on.
    if !seconds.is_finite() || seconds < 0.0 || seconds > 1e15 {
        return Err(invalid());
    }
    Ok(Duration::from_secs_f64(seconds))
}

/// Strictly `one-or-more digits [. one-or-more digits]`.
fn parse_number(num: &str) -> Option<f64> {
    if num.is_empty() {
        return None;
    }
    let (int_part, frac_part) = match num.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (num, None),
    };
    if int_part.is_empty() || !int_part.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if let Some(f) = frac_part {
        if f.is_empty() || !f.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
    }
    num.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_suffixes_and_decimals() {
        assert_eq!(parse_duration("2"), Ok(Duration::from_secs(2)));
        assert_eq!(parse_duration("2s"), Ok(Duration::from_secs(2)));
        assert_eq!(parse_duration("1.5s"), Ok(Duration::from_millis(1500)));
        assert_eq!(parse_duration("1.5"), Ok(Duration::from_millis(1500)));
        assert_eq!(parse_duration("0.25s"), Ok(Duration::from_millis(250)));
        assert_eq!(parse_duration("1.5m"), Ok(Duration::from_secs(90)));
        assert_eq!(parse_duration("2m"), Ok(Duration::from_secs(120)));
        assert_eq!(parse_duration("1h"), Ok(Duration::from_secs(3600)));
        assert_eq!(parse_duration("1d"), Ok(Duration::from_secs(86_400)));
        assert_eq!(parse_duration("1.5S"), Ok(Duration::from_millis(1500)));
    }

    #[test]
    fn rejects_invalid_intervals() {
        for bad in ["1x", "", "s", ".5", "1.", "1.2.3", "-5", "1e3", "abc"] {
            assert_eq!(parse_duration(bad), Err(bad.to_string()), "input {bad:?}");
        }
    }
}
