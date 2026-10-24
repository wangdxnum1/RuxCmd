use clap::{Arg, Command};

pub struct Cli {
    duration: std::time::Duration,
}

fn parse_duration(input: &str) -> Result<std::time::Duration, String> {
    let input = input.trim().to_lowercase();
    let (num_str, unit_str) = input
        .find(|c: char| c.is_ascii_alphabetic())
        .map(|idx| input.split_at(idx))
        .unwrap_or((&input, "s"));

    let num: f64 = num_str.parse().map_err(|_| "Invalid number".to_string())?;
    let unit_ms = match unit_str {
        "s" => 1_000.0,
        "m" => 60_000.0,
        "h" => 3_600_000.0,
        _ => return Err("Invalid time unit".to_string()),
    };

    let ms = num * unit_ms;
    if !ms.is_finite() || ms < 0.0 {
        return Err("Invalid time interval".to_string());
    }
    if ms >= u64::MAX as f64 {
        return Err("Time interval overflow".to_string());
    }

    Ok(std::time::Duration::from_millis(ms as u64))
}

impl Cli {
    pub fn new() -> Self {
        let cmd = Command::new("sleep")
            .version("0.1.0")
            .author("")
            .about("Delay execution for a specified amount of time")
            .disable_version_flag(true)
            .arg(
                Arg::new("duration")
                    .required(true)
                    .help(
                        "Time to sleep (supports fractional s/m/h suffix, e.g., 0.5s, 5s, 1m, 2h)",
                    )
                    .value_parser(parse_duration),
            )
            .arg(
                Arg::new("version")
                    .short('v')
                    .long("version")
                    .action(clap::ArgAction::Version)
                    .help("Print version information"),
            );

        let matches = cmd.get_matches();
        Cli {
            duration: *matches.get_one::<std::time::Duration>("duration").unwrap(),
        }
    }

    pub fn duration(&self) -> std::time::Duration {
        self.duration
    }
}
