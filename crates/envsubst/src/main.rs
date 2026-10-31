use clap::Parser;
use std::io::Read;

/// Substitute environment variables in text — a Windows port of GNU envsubst.
#[derive(Parser, Debug)]
#[command(name = "envsubst", version, about)]
pub struct Args {
    /// Only replace the variables listed in this shell-format string
    #[arg(value_name = "SHELL-FORMAT")]
    pub shell_format: Option<String>,
}

fn main() {
    let args = Args::parse();

    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        eprintln!("envsubst: error reading stdin");
        std::process::exit(1);
    }

    let allowed: Option<Vec<String>> = args.shell_format.as_deref().map(extract_var_names);
    let result = substitute(&input, allowed.as_ref());
    print!("{}", result);
}

fn extract_var_names(fmt: &str) -> Vec<String> {
    let mut names = Vec::new();
    let bytes = fmt.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'{' {
                if let Some(end) = fmt[i + 2..].find('}') {
                    names.push(fmt[i + 2..i + 2 + end].to_string());
                    i += 2 + end + 1;
                    continue;
                }
            } else if bytes[i + 1].is_ascii_alphabetic() || bytes[i + 1] == b'_' {
                let start = i + 1;
                let mut end = start;
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_')
                {
                    end += 1;
                }
                names.push(fmt[start..end].to_string());
                i = end;
                continue;
            }
        }
        i += 1;
    }
    names
}

fn substitute(input: &str, allowed: Option<&Vec<String>>) -> String {
    let mut result = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'{' {
                if let Some(end) = input[i + 2..].find('}') {
                    let name = &input[i + 2..i + 2 + end];
                    let value = if allowed.map(|a| a.iter().any(|n| n == name)).unwrap_or(true) {
                        std::env::var(name).unwrap_or_default()
                    } else {
                        input[i..i + 2 + end + 1].to_string()
                    };
                    result.push_str(&value);
                    i += 2 + end + 1;
                    continue;
                }
            } else if bytes[i + 1].is_ascii_alphabetic() || bytes[i + 1] == b'_' {
                let start = i + 1;
                let mut end = start;
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_')
                {
                    end += 1;
                }
                let name = &input[start..end];
                let value = if allowed.map(|a| a.iter().any(|n| n == name)).unwrap_or(true) {
                    std::env::var(name).unwrap_or_default()
                } else {
                    input[i..end].to_string()
                };
                result.push_str(&value);
                i = end;
                continue;
            }
        }
        result.push_str(&input[i..i + 1]);
        i += 1;
    }
    result
}
