mod cli;

use clap::Parser;

fn main() {
    let args = cli::Args::parse();

    let (start, step, end) = parse_numbers(&args.numbers);

    let sequence = generate_sequence(start, step, end);

    let formatted = format_sequence(&sequence, &args);

    print!("{}", formatted);
}

fn parse_numbers(numbers: &[String]) -> (f64, f64, f64) {
    match numbers.len() {
        1 => {
            let end = numbers[0].parse::<f64>().unwrap();
            (1.0, 1.0, end)
        }
        2 => {
            let start = numbers[0].parse::<f64>().unwrap();
            let end = numbers[1].parse::<f64>().unwrap();
            (start, 1.0, end)
        }
        3 => {
            let start = numbers[0].parse::<f64>().unwrap();
            let step = numbers[1].parse::<f64>().unwrap();
            let end = numbers[2].parse::<f64>().unwrap();
            (start, step, end)
        }
        _ => {
            eprintln!("seq: wrong number of arguments");
            std::process::exit(1);
        }
    }
}

fn generate_sequence(start: f64, step: f64, end: f64) -> Vec<f64> {
    let mut sequence = Vec::new();
    let mut current = start;

    if step > 0.0 {
        while current <= end {
            sequence.push(current);
            current += step;
        }
    } else if step < 0.0 {
        while current >= end {
            sequence.push(current);
            current += step;
        }
    } else {
        eprintln!("seq: step cannot be zero");
        std::process::exit(1);
    }

    sequence
}

fn format_sequence(sequence: &[f64], args: &cli::Args) -> String {
    let formatted: Vec<String> = if let Some(format) = &args.format {
        sequence.iter().map(|n| format_number(n, format)).collect()
    } else if args.equal_width {
        let max_width = max_width(sequence);
        sequence
            .iter()
            .map(|n| format_number_equal_width(n, max_width))
            .collect()
    } else {
        sequence.iter().map(|n| format_number_default(n)).collect()
    };

    formatted.join(&args.separator)
}

fn format_number(n: &f64, format: &str) -> String {
    match format {
        "%f" => format!("{}", n),
        "%g" => {
            let s = format!("{}", n);
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        }
        "%e" => format!("{:e}", n),
        fmt if fmt.starts_with("%") => {
            eprintln!("seq: invalid format string: {}", fmt);
            std::process::exit(1);
        }
        _ => format!("{}", n),
    }
}

fn format_number_default(n: &f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", *n as i64)
    } else {
        format!("{}", n)
    }
}

fn max_width(sequence: &[f64]) -> usize {
    sequence
        .iter()
        .map(|n| format_number_default(n).len())
        .max()
        .unwrap_or(0)
}

fn format_number_equal_width(n: &f64, width: usize) -> String {
    let s = format_number_default(n);
    if s.len() < width {
        let padding = "0".repeat(width - s.len());
        format!("{}{}", padding, s)
    } else {
        s
    }
}