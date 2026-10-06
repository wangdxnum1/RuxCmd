mod cli;

use clap::Parser;
use cli::Args;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};

fn main() {
    let args = Args::parse();
    let mut vars: HashMap<String, f64> = HashMap::new();

    if !args.files.is_empty() {
        for path in &args.files {
            if let Ok(content) = fs::read_to_string(path) {
                process_input(&content, &mut vars);
            } else {
                eprintln!("bc: cannot open '{}'", path.display());
            }
        }
    } else {
        let stdin = io::stdin();
        println!("bc 1.07.1");
        println!("Copyright 1991-1994, 1997, 1998, 2000, 2004, 2006, 2008, 2012-2017 Free Software Foundation, Inc.");
        println!("This is free software with ABSOLUTELY NO WARRANTY.");
        println!("For details type `warranty'.");

        for line in stdin.lock().lines() {
            let line = line.unwrap();
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line == "quit" || line == "exit" {
                break;
            }
            if line == "warranty" {
                println!(
                    "This is free software; see the source for copying conditions.  There is NO"
                );
                println!(
                    "warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE."
                );
                continue;
            }
            process_line(line, &mut vars);
        }
    }
}

fn process_input(input: &str, vars: &mut HashMap<String, f64>) {
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        process_line(line, vars);
    }
}

fn process_line(line: &str, vars: &mut HashMap<String, f64>) {
    let line = line.replace(" ", "");
    if line.starts_with("scale=") {
        return;
    }
    if line.contains('=') {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() == 2 {
            let var_name = parts[0].to_string();
            let value = eval_expr(parts[1], vars);
            vars.insert(var_name, value);
            println!("{}", value);
        }
        return;
    }
    let result = eval_expr(&line, vars);
    println!("{}", result);
}

fn eval_expr(expr: &str, vars: &HashMap<String, f64>) -> f64 {
    let expr = expr.replace("(", "( ").replace(")", " )");
    let tokens: Vec<&str> = expr.split_whitespace().collect();
    let mut output: Vec<&str> = Vec::new();
    let mut ops: Vec<&str> = Vec::new();

    for token in tokens {
        match token {
            "(" => ops.push(token),
            ")" => {
                while let Some(op) = ops.pop() {
                    if op == "(" {
                        break;
                    }
                    output.push(op);
                }
            }
            "+" | "-" | "*" | "/" | "^" => {
                while let Some(&top) = ops.last() {
                    if top == "(" || precedence(top) < precedence(token) {
                        break;
                    }
                    output.push(ops.pop().unwrap());
                }
                ops.push(token);
            }
            _ => output.push(token),
        }
    }

    while let Some(op) = ops.pop() {
        output.push(op);
    }

    let mut stack: Vec<f64> = Vec::new();
    for token in output {
        if let Ok(num) = token.parse::<f64>() {
            stack.push(num);
        } else if let Some(&val) = vars.get(token) {
            stack.push(val);
        } else {
            let b = stack.pop().unwrap_or(0.0);
            let a = stack.pop().unwrap_or(0.0);
            match token {
                "+" => stack.push(a + b),
                "-" => stack.push(a - b),
                "*" => stack.push(a * b),
                "/" => stack.push(a / b),
                "^" => stack.push(a.powf(b)),
                _ => {}
            }
        }
    }

    stack.pop().unwrap_or(0.0)
}

fn precedence(op: &str) -> i32 {
    match op {
        "+" | "-" => 1,
        "*" | "/" => 2,
        "^" => 3,
        _ => 0,
    }
}
