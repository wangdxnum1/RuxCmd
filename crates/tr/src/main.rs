mod cli;

use clap::Parser;
use cli::Cli;
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

fn build_char_vec(pattern: &str) -> Vec<char> {
    let mut vec = Vec::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 2 < chars.len() && chars[i + 1] == '-' {
            let start = chars[i];
            let end = chars[i + 2];
            for c in start..=end {
                vec.push(c);
            }
            i += 3;
        } else {
            vec.push(chars[i]);
            i += 1;
        }
    }

    vec
}

fn build_char_set(pattern: &str) -> HashSet<char> {
    let mut set = HashSet::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 2 < chars.len() && chars[i + 1] == '-' {
            let start = chars[i];
            let end = chars[i + 2];
            for c in start..=end {
                set.insert(c);
            }
            i += 3;
        } else {
            set.insert(chars[i]);
            i += 1;
        }
    }

    set
}

fn build_complement_set(set1: &HashSet<char>) -> HashSet<char> {
    let mut complement = HashSet::new();
    for c in char::MIN..=char::MAX {
        if !set1.contains(&c) {
            complement.insert(c);
        }
    }
    complement
}

fn transform_replace(input: &str, set1: &[char], set2: &[char]) -> String {
    let set2_len = set2.len();
    let mut result = String::new();

    for c in input.chars() {
        if let Some(idx) = set1.iter().position(|&x| x == c) {
            if idx < set2_len {
                result.push(set2[idx]);
            } else if set2_len > 0 {
                result.push(set2[set2_len - 1]);
            } else {
                // Empty SET2 never reaches translation (rejected upfront),
                // but keep the mapping total instead of underflowing.
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn transform_delete(input: &str, set: &HashSet<char>) -> String {
    input.chars().filter(|c| !set.contains(c)).collect()
}

fn transform_squeeze(input: &str, set: &HashSet<char>) -> String {
    let mut result = String::new();
    let mut prev_char: Option<char> = None;

    for c in input.chars() {
        if set.contains(&c) {
            if prev_char != Some(c) {
                result.push(c);
                prev_char = Some(c);
            }
        } else {
            result.push(c);
            prev_char = Some(c);
        }
    }

    result
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if cli.version {
        println!("tr 0.1.0");
        return Ok(());
    }

    let set1_str = match cli.set1 {
        Some(s) => s,
        None => {
            eprintln!("tr: missing operand");
            eprintln!("Usage: tr [OPTION]... SET1 [SET2]");
            std::process::exit(1);
        }
    };

    if !cli.delete && cli.set2.is_none() && !cli.squeeze {
        // Translation without SET2 has nothing to translate to.
        eprintln!("tr: missing operand after '{}'", set1_str);
        std::process::exit(1);
    }

    if !cli.delete {
        if let Some(ref set2) = cli.set2 {
            if set2.is_empty() {
                eprintln!("tr: when translating, SET2 must be non-empty");
                std::process::exit(1);
            }
        }
    }

    let set1_vec = build_char_vec(&set1_str);
    let mut set1_set = build_char_set(&set1_str);

    if cli.complement {
        set1_set = build_complement_set(&set1_set);
    }

    let stdin = io::stdin();
    let mut output = String::new();

    for line in stdin.lock().lines() {
        let line = line?;
        let mut transformed = line;

        if cli.delete {
            transformed = transform_delete(&transformed, &set1_set);
        } else if let Some(ref set2) = cli.set2 {
            let set2_vec = build_char_vec(set2);
            transformed = transform_replace(&transformed, &set1_vec, &set2_vec);
        }

        if cli.squeeze {
            transformed = transform_squeeze(&transformed, &set1_set);
        }

        output.push_str(&transformed);
        output.push('\n');
    }

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(output.as_bytes())?;

    Ok(())
}
