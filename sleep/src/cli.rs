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

    let num: u64 = num_str.parse().map_err(|_| "Invalid number")?;
    let unit_seconds = match unit_str {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        _ => return Err("Invalid time unit".to_string()),
    };

    Ok(std::time::Duration::from_secs(num * unit_seconds))
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
                    .help("Time to sleep (supports s/m/h suffix, e.g., 5s, 1m, 2h)")
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