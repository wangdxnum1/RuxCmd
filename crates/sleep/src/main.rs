//! GNU sleep: operands are nonnegative durations with an optional lowercase
//! s/m/h/d suffix; multiple operands are summed.

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();

    if argv.iter().any(|a| a == "--version") {
        println!("sleep {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if argv.iter().any(|a| a == "--help") {
        println!("Usage: sleep [OPTION]... DURATION...");
        println!("Pause for DURATION seconds.  Suffix may be 's' for seconds (the default),");
        println!("'m' for minutes, 'h' for hours or 'd' for days.");
        println!();
        println!("      --help     display this help and exit");
        println!("      --version  output version information and exit");
        return;
    }

    if argv.is_empty() {
        eprintln!("sleep: missing operand");
        eprintln!("Try 'sleep --help' for more information.");
        std::process::exit(1);
    }

    let mut total_secs = 0.0f64;
    for operand in &argv {
        match parse_duration(operand) {
            Some(secs) => total_secs += secs,
            None => {
                eprintln!("sleep: invalid time interval '{}'", operand);
                std::process::exit(1);
            }
        }
    }

    if !total_secs.is_finite() || total_secs < 0.0 {
        eprintln!("sleep: invalid time interval");
        std::process::exit(1);
    }
    // Cap at ~68 years of milliseconds to keep the u64 conversion sane.
    if total_secs >= u64::MAX as f64 / 1000.0 {
        eprintln!("sleep: invalid time interval: overflow");
        std::process::exit(1);
    }

    std::thread::sleep(std::time::Duration::from_millis(
        (total_secs * 1000.0) as u64,
    ));
}

/// Parse one GNU sleep operand: digits with optional fraction and an
/// optional lowercase suffix (s, m, h, d). Uppercase suffixes are invalid.
pub fn parse_duration(input: &str) -> Option<f64> {
    let (num_part, unit) = match input.find(|c: char| c.is_ascii_alphabetic()) {
        Some(idx) => {
            let (num, unit) = input.split_at(idx);
            (num, unit)
        }
        None => (input, "s"),
    };
    if num_part.is_empty() {
        return None;
    }
    let num: f64 = num_part.parse().ok()?;
    if num < 0.0 || !num.is_finite() {
        return None;
    }
    let multiplier = match unit {
        "s" => 1.0,
        "m" => 60.0,
        "h" => 3600.0,
        "d" => 86400.0,
        _ => return None,
    };
    Some(num * multiplier)
}

#[cfg(test)]
mod tests {
    use super::parse_duration;

    #[test]
    fn suffixes_multiply() {
        assert_eq!(parse_duration("2"), Some(2.0));
        assert_eq!(parse_duration("0.5s"), Some(0.5));
        assert_eq!(parse_duration("1m"), Some(60.0));
        assert_eq!(parse_duration("1h"), Some(3600.0));
        assert_eq!(parse_duration("1d"), Some(86400.0));
    }

    #[test]
    fn invalid_operands_are_rejected() {
        assert_eq!(parse_duration("1S"), None);
        assert_eq!(parse_duration("xx"), None);
        assert_eq!(parse_duration("1x"), None);
        assert_eq!(parse_duration("-1"), None);
        assert_eq!(parse_duration(""), None);
    }

    #[test]
    fn bare_unit_without_number_is_invalid() {
        assert_eq!(parse_duration("s"), None);
    }
}
