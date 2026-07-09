mod cli;

use clap::Parser;

fn main() {
    let args = cli::Args::parse();

    let text = args.strings.join(" ");
    
    let output = if args.enable_escapes || (!args.disable_escapes && !args.strings.is_empty()) {
        expand_escapes(&text)
    } else {
        text
    };

    if args.no_newline {
        print!("{}", output);
    } else {
        println!("{}", output);
    }
}

fn expand_escapes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some(&'n') => {
                    result.push('\n');
                    chars.next();
                }
                Some(&'r') => {
                    result.push('\r');
                    chars.next();
                }
                Some(&'t') => {
                    result.push('\t');
                    chars.next();
                }
                Some(&'\\') => {
                    result.push('\\');
                    chars.next();
                }
                Some(&'\"') => {
                    result.push('\"');
                    chars.next();
                }
                _ => {
                    result.push(c);
                }
            }
        } else {
            result.push(c);
        }
    }
    
    result
}