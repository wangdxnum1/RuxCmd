mod cli;

use clap::Parser;

fn main() {
    let args = cli::Args::parse();
    let formatted = printf(&args.format, &args.args);
    print!("{}", formatted);
}

fn printf(format: &str, args: &[String]) -> String {
    let mut result = String::new();
    let mut chars = format.chars().peekable();
    let mut arg_index = 0;

    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.next() {
                Some('s') => {
                    if arg_index < args.len() {
                        result.push_str(&args[arg_index]);
                    } else {
                        result.push_str("%s");
                    }
                    arg_index += 1;
                }
                Some('d') => {
                    if arg_index < args.len() {
                        if let Ok(num) = args[arg_index].parse::<i64>() {
                            result.push_str(&num.to_string());
                        } else {
                            result.push_str("%d");
                        }
                    } else {
                        result.push_str("%d");
                    }
                    arg_index += 1;
                }
                Some('x') => {
                    if arg_index < args.len() {
                        if let Ok(num) = args[arg_index].parse::<u64>() {
                            result.push_str(&format!("{:x}", num));
                        } else {
                            result.push_str("%x");
                        }
                    } else {
                        result.push_str("%x");
                    }
                    arg_index += 1;
                }
                Some('f') => {
                    if arg_index < args.len() {
                        if let Ok(num) = args[arg_index].parse::<f64>() {
                            result.push_str(&num.to_string());
                        } else {
                            result.push_str("%f");
                        }
                    } else {
                        result.push_str("%f");
                    }
                    arg_index += 1;
                }
                Some('c') => {
                    if arg_index < args.len() {
                        if let Some(ch) = args[arg_index].chars().next() {
                            result.push(ch);
                        } else {
                            result.push_str("%c");
                        }
                    } else {
                        result.push_str("%c");
                    }
                    arg_index += 1;
                }
                Some('%') => {
                    result.push('%');
                }
                Some(other) => {
                    result.push('%');
                    result.push(other);
                }
                None => {
                    result.push('%');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}
