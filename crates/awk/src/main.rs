mod cli;

use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("awk 0.1.0");
        return;
    }

    let program = if let Some(f) = &args.file {
        let mut content = String::new();
        match File::open(f) {
            Ok(mut file) => {
                if let Err(e) = file.read_to_string(&mut content) {
                    eprintln!("awk: {}: {}", f.display(), e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("awk: {}: {}", f.display(), e);
                std::process::exit(1);
            }
        }
        content
    } else if let Some(p) = &args.program {
        p.clone()
    } else {
        eprintln!("awk: missing program");
        std::process::exit(1);
    };

    let mut vars: HashMap<String, String> = HashMap::new();
    for (k, v) in &args.vars {
        vars.insert(k.clone(), v.clone());
    }
    vars.insert("FS".to_string(), args.field_separator.clone());
    vars.insert("OFS".to_string(), " ".to_string());
    vars.insert("RS".to_string(), "\n".to_string());
    vars.insert("ORS".to_string(), "\n".to_string());

    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = process_input(stdin.lock(), &program, &mut vars) {
            eprintln!("awk: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for path in &args.files {
            let path_str = path.to_string_lossy();
            if path_str == "-" {
                let stdin = std::io::stdin();
                if let Err(e) = process_input(stdin.lock(), &program, &mut vars) {
                    eprintln!("awk: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = process_input(reader, &program, &mut vars) {
                            eprintln!("awk: {}: {}", path.display(), e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("awk: {}: {}", path.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    std::process::exit(exit_code);
}

fn process_input<R: BufRead>(
    mut reader: R,
    program: &str,
    vars: &mut HashMap<String, String>,
) -> Result<(), std::io::Error> {
    let mut nr = 0;
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        lines.push(line.trim_end().to_string());
        line.clear();
    }

    let mut tokens = tokenize(program);
    let parsed = parse_program(&mut tokens);

    for action in &parsed.begin_actions {
        execute_action(action, "", &[], 0, 0, vars);
    }

    for (_i, line) in lines.iter().enumerate() {
        nr += 1;
        let fields: Vec<&str> = if vars["FS"] == " " {
            line.split_whitespace().collect()
        } else {
            line.split(&vars["FS"]).collect()
        };
        let nf = fields.len();

        vars.insert("NR".to_string(), nr.to_string());
        vars.insert("NF".to_string(), nf.to_string());
        vars.insert("$0".to_string(), line.clone());
        for (j, f) in fields.iter().enumerate() {
            vars.insert(format!("${}", j + 1), f.to_string());
        }

        for (pattern, action) in &parsed.pattern_actions {
            if match_pattern(pattern, line, vars) {
                execute_action(action, line, &fields, nr, nf, vars);
            }
        }
    }

    for action in &parsed.end_actions {
        execute_action(action, "", &[], nr, 0, vars);
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum Token {
    Pattern(String),
    Action(String),
    Begin,
    End,
    Print,
    Printf,
    If,
    Else,
    For,
    Next,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Comma,
    Assign,
    Var(String),
    Number(f64),
    String(String),
    Op(String),
    Regex(String),
}

fn tokenize(s: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            ' ' | '\t' | '\n' | '\r' => continue,
            '{' => {
                let mut action = String::new();
                let mut depth = 1;
                while let Some(ch) = chars.next() {
                    if ch == '{' {
                        depth += 1;
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    action.push(ch);
                }
                tokens.push(Token::Action(action));
            }
            '/' => {
                let mut regex = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '/' {
                        break;
                    }
                    regex.push(ch);
                }
                tokens.push(Token::Regex(regex));
            }
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            ';' => tokens.push(Token::Semicolon),
            ',' => tokens.push(Token::Comma),
            '=' => tokens.push(Token::Assign),
            '+' | '-' | '*' | '<' | '>' | '!' | '&' | '|' => {
                let mut op = String::new();
                op.push(c);
                while let Some(&next) = chars.peek() {
                    if next == '=' || next == '&' || next == '|' {
                        op.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Op(op));
            }
            '"' => {
                let mut str_val = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '"' {
                        break;
                    } else if ch == '\\' {
                        if let Some(next) = chars.next() {
                            str_val.push(match next {
                                'n' => '\n',
                                't' => '\t',
                                'r' => '\r',
                                '\\' => '\\',
                                '"' => '"',
                                _ => next,
                            });
                        }
                    } else {
                        str_val.push(ch);
                    }
                }
                tokens.push(Token::String(str_val));
            }
            '0'..='9' => {
                let mut num = String::new();
                num.push(c);
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_digit() || next == '.' {
                        num.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if let Ok(n) = num.parse::<f64>() {
                    tokens.push(Token::Number(n));
                } else {
                    tokens.push(Token::Var(num));
                }
            }
            '$' => {
                let mut var = String::new();
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_alphanumeric() || next == '_' {
                        var.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Var(format!("${}", var)));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut word = String::new();
                word.push(c);
                while let Some(&next) = chars.peek() {
                    if next.is_ascii_alphanumeric() || next == '_' {
                        word.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                match word.as_str() {
                    "BEGIN" => tokens.push(Token::Begin),
                    "END" => tokens.push(Token::End),
                    "print" => tokens.push(Token::Print),
                    "printf" => tokens.push(Token::Printf),
                    "if" => tokens.push(Token::If),
                    "else" => tokens.push(Token::Else),
                    "for" => tokens.push(Token::For),
                    "next" => tokens.push(Token::Next),
                    _ => tokens.push(Token::Var(word)),
                }
            }
            _ => {}
        }
    }

    tokens
}

#[derive(Debug, Clone)]
struct ParsedProgram {
    begin_actions: Vec<String>,
    pattern_actions: Vec<(String, String)>,
    end_actions: Vec<String>,
}

fn parse_program(tokens: &mut Vec<Token>) -> ParsedProgram {
    let mut parsed = ParsedProgram {
        begin_actions: Vec::new(),
        pattern_actions: Vec::new(),
        end_actions: Vec::new(),
    };

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Begin => {
                if let Some(Token::Action(a)) = tokens.get(i + 1) {
                    parsed.begin_actions.push(a.clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            Token::End => {
                if let Some(Token::Action(a)) = tokens.get(i + 1) {
                    parsed.end_actions.push(a.clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            Token::Regex(_) | Token::Var(_) => {
                let pattern = match &tokens[i] {
                    Token::Regex(r) => format!("/{}/", r),
                    Token::Var(p) => p.clone(),
                    _ => String::new(),
                };
                if let Some(Token::Action(a)) = tokens.get(i + 1) {
                    parsed.pattern_actions.push((pattern, a.clone()));
                    i += 2;
                } else {
                    parsed.pattern_actions.push((pattern, "print".to_string()));
                    i += 1;
                }
            }
            Token::Action(a) => {
                parsed.pattern_actions.push((String::new(), a.clone()));
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    parsed
}

fn match_pattern(pattern: &str, line: &str, vars: &HashMap<String, String>) -> bool {
    if pattern.is_empty() {
        return true;
    }
    if pattern.starts_with('/') && pattern.ends_with('/') {
        let regex = &pattern[1..pattern.len() - 1];
        line.contains(regex)
    } else {
        eval_expr(pattern, vars) > 0.0
    }
}

fn eval_expr(expr: &str, vars: &HashMap<String, String>) -> f64 {
    let tokens = tokenize(expr);
    eval_tokens(&tokens, vars)
}

fn eval_tokens(tokens: &[Token], vars: &HashMap<String, String>) -> f64 {
    let mut stack: Vec<f64> = Vec::new();
    let mut ops: Vec<String> = Vec::new();

    for token in tokens {
        match token {
            Token::Number(n) => stack.push(*n),
            Token::Var(v) => {
                if let Some(val) = vars.get(v) {
                    if let Ok(n) = val.parse::<f64>() {
                        stack.push(n);
                    } else {
                        stack.push(1.0);
                    }
                } else {
                    stack.push(0.0);
                }
            }
            Token::String(s) => {
                stack.push(s.len() as f64);
            }
            Token::Op(op) => {
                while let Some(top) = ops.last() {
                    if precedence(top) >= precedence(op) {
                        apply_op(&mut stack, ops.pop().unwrap());
                    } else {
                        break;
                    }
                }
                ops.push(op.clone());
            }
            Token::LParen => ops.push("(".to_string()),
            Token::RParen => {
                while let Some(op) = ops.pop() {
                    if op == "(" {
                        break;
                    }
                    apply_op(&mut stack, op);
                }
            }
            _ => {}
        }
    }

    while let Some(op) = ops.pop() {
        apply_op(&mut stack, op);
    }

    stack.pop().unwrap_or(0.0)
}

fn precedence(op: &str) -> i32 {
    match op {
        "*" | "/" => 2,
        "+" | "-" => 1,
        "<" | ">" | "<=" | ">=" | "==" | "!=" => 0,
        _ => -1,
    }
}

fn apply_op(stack: &mut Vec<f64>, op: String) {
    let b = stack.pop().unwrap_or(0.0);
    let a = stack.pop().unwrap_or(0.0);
    let result = match op.as_str() {
        "+" => a + b,
        "-" => a - b,
        "*" => a * b,
        "/" => {
            if b != 0.0 {
                a / b
            } else {
                0.0
            }
        }
        "<" => {
            if a < b {
                1.0
            } else {
                0.0
            }
        }
        ">" => {
            if a > b {
                1.0
            } else {
                0.0
            }
        }
        "<=" => {
            if a <= b {
                1.0
            } else {
                0.0
            }
        }
        ">=" => {
            if a >= b {
                1.0
            } else {
                0.0
            }
        }
        "==" => {
            if a == b {
                1.0
            } else {
                0.0
            }
        }
        "!=" => {
            if a != b {
                1.0
            } else {
                0.0
            }
        }
        _ => 0.0,
    };
    stack.push(result);
}

fn execute_action(
    action: &str,
    line: &str,
    fields: &[&str],
    nr: usize,
    nf: usize,
    vars: &mut HashMap<String, String>,
) {
    let tokens = tokenize(action);
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Print => {
                i += 1;
                let mut args = Vec::new();
                if let Some(Token::LParen) = tokens.get(i) {
                    i += 1;
                    while let Some(t) = tokens.get(i) {
                        match t {
                            Token::RParen => break,
                            Token::Comma => {
                                i += 1;
                                continue;
                            }
                            Token::Var(v) => args.push(vars.get(v).cloned().unwrap_or_default()),
                            Token::String(s) => args.push(s.clone()),
                            Token::Number(n) => args.push(n.to_string()),
                            _ => {}
                        }
                        i += 1;
                    }
                    i += 1;
                } else if i < tokens.len() {
                    match &tokens[i] {
                        Token::Var(v) => args.push(vars.get(v).cloned().unwrap_or_default()),
                        Token::String(s) => args.push(s.clone()),
                        _ => {}
                    }
                    i += 1;
                }

                if args.is_empty() {
                    println!("{}", line);
                } else {
                    println!("{}", args.join(&vars["OFS"]));
                }
            }
            Token::Printf => {
                i += 1;
                let mut fmt = String::new();
                let mut args = Vec::new();
                if let Some(Token::LParen) = tokens.get(i) {
                    i += 1;
                    if let Some(Token::String(s)) = tokens.get(i) {
                        fmt = s.clone();
                        i += 1;
                    }
                    while let Some(t) = tokens.get(i) {
                        match t {
                            Token::RParen => break,
                            Token::Comma => {
                                i += 1;
                                continue;
                            }
                            Token::Var(v) => args.push(vars.get(v).cloned().unwrap_or_default()),
                            Token::String(s) => args.push(s.clone()),
                            Token::Number(n) => args.push(n.to_string()),
                            _ => {}
                        }
                        i += 1;
                    }
                    i += 1;
                }
                if !fmt.is_empty() {
                    let mut result = String::new();
                    let mut arg_idx = 0;
                    let chars: Vec<char> = fmt.chars().collect();
                    let mut j = 0;
                    while j < chars.len() {
                        if chars[j] == '%' && j + 1 < chars.len() {
                            j += 1;
                            let spec = chars[j];
                            let arg_str = args.get(arg_idx).cloned().unwrap_or_default();
                            match spec {
                                's' => result.push_str(&arg_str),
                                'd' => {
                                    if let Ok(n) = arg_str.parse::<i64>() {
                                        result.push_str(&n.to_string());
                                    } else {
                                        result.push_str(&arg_str);
                                    }
                                }
                                'f' => {
                                    if let Ok(n) = arg_str.parse::<f64>() {
                                        result.push_str(&n.to_string());
                                    } else {
                                        result.push_str(&arg_str);
                                    }
                                }
                                '%' => result.push('%'),
                                _ => {
                                    result.push('%');
                                    result.push(spec);
                                }
                            }
                            arg_idx += 1;
                        } else {
                            result.push(chars[j]);
                        }
                        j += 1;
                    }
                    print!("{}", result);
                }
            }
            Token::Var(v) => {
                if let Some(Token::Assign) = tokens.get(i + 1) {
                    let var_name = v.clone();
                    i += 2;
                    let val = match &tokens[i] {
                        Token::Var(v2) => vars.get(v2).cloned().unwrap_or_default(),
                        Token::String(s) => s.clone(),
                        Token::Number(n) => n.to_string(),
                        _ => String::new(),
                    };
                    vars.insert(var_name, val);
                    i += 1;
                } else {
                    i += 1;
                }
            }
            Token::If => {
                i += 1;
                let mut cond = String::new();
                if let Some(Token::LParen) = tokens.get(i) {
                    i += 1;
                    while let Some(t) = tokens.get(i) {
                        match t {
                            Token::RParen => break,
                            _ => cond.push_str(&format!("{:?}", t)),
                        }
                        i += 1;
                    }
                    i += 1;
                }
                if let Some(Token::Action(a)) = tokens.get(i) {
                    if eval_expr(&cond, vars) > 0.0 {
                        execute_action(a, line, fields, nr, nf, vars);
                    }
                    i += 1;
                }
            }
            Token::Next => {
                return;
            }
            _ => {
                i += 1;
            }
        }
    }
}
