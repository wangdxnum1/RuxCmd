mod cli;

use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

fn main() {
    let args = cli::Args::parse();

    let stdin = io::stdin();
    let mut input_args: Vec<String> = Vec::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("xargs: error reading stdin: {}", e);
                std::process::exit(1);
            }
        };
        input_args.extend(line.split_whitespace().map(|s| s.to_string()));
    }

    if input_args.is_empty() {
        std::process::exit(0);
    }

    let replace_str = &args.replace;
    let replace_set = args.replace_set;
    let uses_replace = args
        .command
        .iter()
        .any(|arg| arg == replace_str || arg.contains(replace_str));
    let max_args_val = args.max_args;

    let mut command_template = args.command;
    if command_template.is_empty() {
        command_template.push("echo".to_string());
    }

    let (max_args, is_replace_mode) = if replace_set || uses_replace {
        (1, true)
    } else if max_args_val == 0 {
        (input_args.len(), false)
    } else {
        (max_args_val, false)
    };

    let mut i = 0;
    while i < input_args.len() {
        let end = std::cmp::min(i + max_args, input_args.len());
        let batch_args = &input_args[i..end];

        let mut cmd_args: Vec<String> = Vec::new();

        if is_replace_mode {
            for arg in &command_template {
                if arg == replace_str {
                    for input_arg in batch_args {
                        cmd_args.push(input_arg.clone());
                    }
                } else if arg.contains(replace_str) {
                    let replaced = arg.replace(replace_str, "{}");
                    let mut has_placeholder = false;
                    let mut replaced_arg = String::new();
                    for part in replaced.split("{}") {
                        replaced_arg.push_str(part);
                        if !has_placeholder && !batch_args.is_empty() {
                            replaced_arg.push_str(&batch_args[0]);
                            has_placeholder = true;
                        }
                    }
                    cmd_args.push(replaced_arg);
                } else {
                    cmd_args.push(arg.clone());
                }
            }

            if !uses_replace {
                for input_arg in batch_args {
                    cmd_args.push(input_arg.clone());
                }
            }
        } else {
            cmd_args.extend_from_slice(&command_template);
            cmd_args.extend_from_slice(batch_args);
        }

        if cmd_args.is_empty() {
            cmd_args.extend_from_slice(batch_args);
        }

        let program = &cmd_args[0];
        let cmd_args_slice = if cmd_args.len() > 1 {
            &cmd_args[1..]
        } else {
            &[]
        };

        if args.verbose {
            eprintln!("{} {}", program, cmd_args_slice.join(" "));
        }

        if args.interactive {
            print!("{} {} ?...", program, cmd_args_slice.join(" "));
            io::stdout().flush().unwrap();
            let mut response = String::new();
            io::stdin().read_line(&mut response).unwrap();
            let response = response.trim().to_lowercase();
            if response != "y" && response != "yes" {
                i = end;
                continue;
            }
        }

        let status = Command::new(program)
            .args(cmd_args_slice)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();

        match status {
            Ok(s) => {
                if !s.success() {
                    std::process::exit(s.code().unwrap_or(1));
                }
            }
            Err(e) => {
                eprintln!("xargs: {}: {}", program, e);
                std::process::exit(1);
            }
        }

        i = end;
    }
}
