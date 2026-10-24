mod cli;

use clap::Parser;
use std::cmp::Ordering;
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

    // Parse once for the whole run: action bodies are stored as pre-tokenized
    // fragments so if/else conditions are evaluated against the original
    // tokens, and BEGIN/END are not re-run per input file.
    let tokens = tokenize(&program);
    let parsed = match parse_program(&tokens) {
        Ok(p) => p,
        Err(msg) => {
            eprintln!("awk: {}", msg);
            std::process::exit(1);
        }
    };

    let mut vars: HashMap<String, String> = HashMap::new();
    for (k, v) in &args.vars {
        vars.insert(k.clone(), v.clone());
    }
    vars.insert("FS".to_string(), args.field_separator.clone());
    vars.insert("OFS".to_string(), " ".to_string());
    vars.insert("RS".to_string(), "\n".to_string());
    vars.insert("ORS".to_string(), "\n".to_string());
    vars.insert("NR".to_string(), "0".to_string());
    vars.insert("NF".to_string(), "0".to_string());

    // BEGIN runs exactly once, before any input file is read.
    for action in &parsed.begin_actions {
        execute_block(action, &mut vars);
    }

    let mut exit_code = 0i32;
    // NR keeps counting across all input files.
    let mut nr: usize = 0;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = process_input(stdin.lock(), &parsed, &mut vars, &mut nr) {
            eprintln!("awk: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for path in &args.files {
            let path_str = path.to_string_lossy();
            if path_str == "-" {
                let stdin = std::io::stdin();
                if let Err(e) = process_input(stdin.lock(), &parsed, &mut vars, &mut nr) {
                    eprintln!("awk: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(path) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        if let Err(e) = process_input(reader, &parsed, &mut vars, &mut nr) {
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

    // END runs exactly once, after all input files have been processed.
    for action in &parsed.end_actions {
        execute_block(action, &mut vars);
    }

    std::process::exit(exit_code);
}

fn process_input<R: BufRead>(
    mut reader: R,
    parsed: &ParsedProgram,
    vars: &mut HashMap<String, String>,
    nr: &mut usize,
) -> Result<(), std::io::Error> {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        lines.push(line.trim_end().to_string());
        line.clear();
    }

    for line in &lines {
        *nr += 1;
        let fields: Vec<&str> = if vars["FS"] == " " {
            line.split_whitespace().collect()
        } else {
            line.split(&vars["FS"]).collect()
        };
        let nf = fields.len();

        // Clear every $N left over from the previous record before
        // installing this record's fields.
        vars.retain(|k, _| !(k.starts_with('$') && k != "$0"));
        vars.insert("NR".to_string(), nr.to_string());
        vars.insert("NF".to_string(), nf.to_string());
        vars.insert("$0".to_string(), line.clone());
        for (j, f) in fields.iter().enumerate() {
            vars.insert(format!("${}", j + 1), (*f).to_string());
        }

        for (pattern, action) in &parsed.pattern_actions {
            if match_pattern(pattern, line, vars) && execute_block(action, vars) == Flow::Next {
                break;
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum Token {
    /// Braced statement block, kept as pre-tokenized fragments.
    Action(Vec<Token>),
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
                tokens.push(Token::Action(tokenize(&action)));
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
            '=' => {
                // `==` is a comparison operator; a lone `=` is assignment.
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Op("==".to_string()));
                } else {
                    tokens.push(Token::Assign);
                }
            }
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
enum Pattern {
    Always,
    /// `/substring/` matches against the whole line (declared difference:
    /// substring matching, no regex engine).
    Regex(String),
    /// Expression pattern (single variable/field reference), evaluated with
    /// awk truthiness: non-zero number, or non-empty string other than "0".
    Expr(Vec<Token>),
}

#[derive(Debug, Clone)]
struct ParsedProgram {
    begin_actions: Vec<Vec<Token>>,
    pattern_actions: Vec<(Pattern, Vec<Token>)>,
    end_actions: Vec<Vec<Token>>,
}

fn parse_program(tokens: &[Token]) -> Result<ParsedProgram, String> {
    if contains_for(tokens) {
        return Err("for loops are not supported".to_string());
    }

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
            Token::Regex(r) => {
                if let Some(Token::Action(a)) = tokens.get(i + 1) {
                    parsed
                        .pattern_actions
                        .push((Pattern::Regex(r.clone()), a.clone()));
                    i += 2;
                } else {
                    // Pattern without an action defaults to `print`.
                    parsed
                        .pattern_actions
                        .push((Pattern::Regex(r.clone()), vec![Token::Print]));
                    i += 1;
                }
            }
            Token::Var(_) => {
                // Expression pattern: collect tokens up to the action block
                // so forms like `$1 == "a" { ... }` parse as a single rule.
                let end = expr_end(tokens, i);
                let expr: Vec<Token> = tokens[i..end].to_vec();
                if let Some(Token::Action(a)) = tokens.get(end) {
                    parsed.pattern_actions.push((Pattern::Expr(expr), a.clone()));
                    i = end + 1;
                } else {
                    parsed
                        .pattern_actions
                        .push((Pattern::Expr(expr), vec![Token::Print]));
                    i = end;
                }
            }
            Token::Action(a) => {
                parsed.pattern_actions.push((Pattern::Always, a.clone()));
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    Ok(parsed)
}

fn contains_for(tokens: &[Token]) -> bool {
    tokens.iter().any(|t| match t {
        Token::For => true,
        Token::Action(inner) => contains_for(inner),
        _ => false,
    })
}

fn match_pattern(pattern: &Pattern, line: &str, vars: &HashMap<String, String>) -> bool {
    match pattern {
        Pattern::Always => true,
        Pattern::Regex(r) => line.contains(r),
        Pattern::Expr(tokens) => is_truthy(&eval_tokens(tokens, vars)),
    }
}

/// A dynamically typed awk value: numbers keep their numeric form, strings
/// keep their original text (so `print $1` never reformats the field).
#[derive(Debug, Clone)]
enum Value {
    Num(f64),
    Str(String),
}

fn fmt_num(n: f64) -> String {
    if n.is_finite() && n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

impl Value {
    fn to_num(&self) -> f64 {
        match self {
            Value::Num(n) => *n,
            Value::Str(s) => s.trim().parse::<f64>().unwrap_or(0.0),
        }
    }

    fn to_str(&self) -> String {
        match self {
            Value::Num(n) => fmt_num(*n),
            Value::Str(s) => s.clone(),
        }
    }

    /// awk truthiness: a non-zero number, or a non-empty string that is
    /// not "0" (fields and variables are treated as numeric strings).
    fn is_truthy(&self) -> bool {
        match self {
            Value::Num(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty() && s != "0",
        }
    }
}

fn is_truthy(v: &Value) -> bool {
    v.is_truthy()
}

fn num_str(v: &Value) -> Option<f64> {
    match v {
        Value::Num(n) => Some(*n),
        Value::Str(s) => s.trim().parse::<f64>().ok(),
    }
}

/// Comparison semantics: numeric when either side is a number or both sides
/// look numeric (fields are numeric strings), otherwise string comparison.
fn compare_values(a: &Value, b: &Value) -> Ordering {
    let numeric = matches!(a, Value::Num(_))
        || matches!(b, Value::Num(_))
        || (num_str(a).is_some() && num_str(b).is_some());
    if numeric {
        a.to_num().partial_cmp(&b.to_num()).unwrap_or(Ordering::Equal)
    } else {
        a.to_str().cmp(&b.to_str())
    }
}

fn bool_val(b: bool) -> Value {
    Value::Num(if b { 1.0 } else { 0.0 })
}

fn eval_tokens(tokens: &[Token], vars: &HashMap<String, String>) -> Value {
    let mut stack: Vec<Value> = Vec::new();
    let mut ops: Vec<String> = Vec::new();

    for token in tokens {
        match token {
            Token::Number(n) => stack.push(Value::Num(*n)),
            Token::String(s) => stack.push(Value::Str(s.clone())),
            Token::Var(v) => stack.push(Value::Str(vars.get(v).cloned().unwrap_or_default())),
            Token::Op(op) => {
                while let Some(top) = ops.last() {
                    if precedence(top) >= precedence(op) {
                        let pending = ops.pop().unwrap();
                        apply_op(&mut stack, &pending);
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
                    apply_op(&mut stack, &op);
                }
            }
            _ => {}
        }
    }

    while let Some(op) = ops.pop() {
        apply_op(&mut stack, &op);
    }

    stack.pop().unwrap_or(Value::Str(String::new()))
}

fn precedence(op: &str) -> i32 {
    match op {
        "*" | "/" => 2,
        "+" | "-" => 1,
        "<" | ">" | "<=" | ">=" | "==" | "!=" => 0,
        "(" => -2,
        _ => -1,
    }
}

fn apply_op(stack: &mut Vec<Value>, op: &str) {
    let b = stack.pop().unwrap_or(Value::Num(0.0));
    let a = stack.pop().unwrap_or(Value::Num(0.0));
    let result = match op {
        "+" => Value::Num(a.to_num() + b.to_num()),
        "-" => Value::Num(a.to_num() - b.to_num()),
        "*" => Value::Num(a.to_num() * b.to_num()),
        "/" => {
            let divisor = b.to_num();
            if divisor != 0.0 {
                Value::Num(a.to_num() / divisor)
            } else {
                Value::Num(0.0)
            }
        }
        "<" => bool_val(compare_values(&a, &b) == Ordering::Less),
        ">" => bool_val(compare_values(&a, &b) == Ordering::Greater),
        "<=" => bool_val(compare_values(&a, &b) != Ordering::Greater),
        ">=" => bool_val(compare_values(&a, &b) != Ordering::Less),
        "==" => bool_val(compare_values(&a, &b) == Ordering::Equal),
        "!=" => bool_val(compare_values(&a, &b) != Ordering::Equal),
        "&&" => bool_val(a.to_num() != 0.0 && b.to_num() != 0.0),
        "||" => bool_val(a.to_num() != 0.0 || b.to_num() != 0.0),
        _ => Value::Num(0.0),
    };
    stack.push(result);
}

/// Control flow of one executed block: `Next` skips the remaining rules for
/// the current input record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Normal,
    Next,
}

fn execute_block(tokens: &[Token], vars: &mut HashMap<String, String>) -> Flow {
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Print => {
                let paren = matches!(tokens.get(i + 1), Some(Token::LParen));
                let (args, next) = collect_args(tokens, i + 1, paren, vars);
                if args.is_empty() {
                    println!("{}", vars.get("$0").map(String::as_str).unwrap_or(""));
                } else {
                    let strs: Vec<String> = args.iter().map(Value::to_str).collect();
                    println!("{}", strs.join(&vars["OFS"]));
                }
                i = next;
            }
            Token::Printf => {
                let paren = matches!(tokens.get(i + 1), Some(Token::LParen));
                let (args, next) = collect_args(tokens, i + 1, paren, vars);
                if let Some(fmt) = args.first() {
                    let rendered = render_printf(&fmt.to_str(), &args[1..]);
                    print!("{}", rendered);
                }
                i = next;
            }
            Token::If => {
                let (next_i, flow) = execute_if(tokens, i, vars);
                if flow == Flow::Next {
                    return Flow::Next;
                }
                i = next_i;
            }
            Token::Next => return Flow::Next,
            Token::Var(v) => {
                if matches!(tokens.get(i + 1), Some(Token::Assign)) {
                    let name = v.clone();
                    let end = expr_end(tokens, i + 2);
                    let val = eval_tokens(&tokens[i + 2..end], vars).to_str();
                    vars.insert(name, val);
                    i = end;
                } else {
                    i += 1;
                }
            }
            Token::Action(inner) => {
                if execute_block(inner, vars) == Flow::Next {
                    return Flow::Next;
                }
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    Flow::Normal
}

/// Executes the `if` statement starting at `start`. Returns the index after
/// the whole if/else chain plus the control flow of the branch taken.
fn execute_if(
    tokens: &[Token],
    start: usize,
    vars: &mut HashMap<String, String>,
) -> (usize, Flow) {
    let Some(close) = find_matching_paren(tokens, start + 1) else {
        return (start + 1, Flow::Normal);
    };
    let cond = &tokens[start + 2..close];

    let mut j = close + 1;
    let then_end = statement_end(tokens, j);
    let then_stmt = &tokens[j..then_end];
    j = then_end;
    while matches!(tokens.get(j), Some(Token::Semicolon)) {
        j += 1;
    }
    let else_stmt: &[Token] = if matches!(tokens.get(j), Some(Token::Else)) {
        let end = statement_end(tokens, j + 1);
        let stmt = &tokens[j + 1..end];
        j = end;
        stmt
    } else {
        &[]
    };

    let taken: &[Token] = if is_truthy(&eval_tokens(cond, vars)) {
        then_stmt
    } else {
        else_stmt
    };
    let flow = if taken.is_empty() {
        Flow::Normal
    } else {
        execute_block(taken, vars)
    };
    (j, flow)
}

/// End (exclusive) of the statement starting at `i`.
fn statement_end(tokens: &[Token], i: usize) -> usize {
    match tokens.get(i) {
        None => i,
        Some(Token::Print) | Some(Token::Printf) => {
            let paren = matches!(tokens.get(i + 1), Some(Token::LParen));
            args_extent(tokens, i + 1, paren)
        }
        Some(Token::If) => if_chain_end(tokens, i),
        Some(Token::Next) | Some(Token::Action(_)) => i + 1,
        Some(Token::Var(_)) => {
            if matches!(tokens.get(i + 1), Some(Token::Assign)) {
                expr_end(tokens, i + 2)
            } else {
                i + 1
            }
        }
        Some(_) => i + 1,
    }
}

/// End (exclusive) of the whole if/else chain starting at `start`.
fn if_chain_end(tokens: &[Token], start: usize) -> usize {
    let Some(close) = find_matching_paren(tokens, start + 1) else {
        return start + 1;
    };
    let mut j = statement_end(tokens, close + 1);
    while matches!(tokens.get(j), Some(Token::Semicolon)) {
        j += 1;
    }
    if matches!(tokens.get(j), Some(Token::Else)) {
        j = statement_end(tokens, j + 1);
    }
    j
}

/// End (exclusive) of the comma-separated argument list starting at `start`.
/// With `paren`, the list is wrapped in parentheses.
fn args_extent(tokens: &[Token], start: usize, paren: bool) -> usize {
    let mut i = if paren { start + 1 } else { start };
    loop {
        let e = expr_end(tokens, i);
        i = e;
        match tokens.get(i) {
            Some(Token::Comma) => i += 1,
            Some(Token::RParen) if paren => return i + 1,
            // Adjacent value: the list already ended (concatenation is not
            // supported, so the neighbouring token is a new statement).
            Some(t) if is_value_start(t) => return i,
            _ => return i,
        }
    }
}

/// Collects and evaluates the comma-separated argument list starting at
/// `start`. Returns the values plus the index after the whole list.
fn collect_args(
    tokens: &[Token],
    start: usize,
    paren: bool,
    vars: &HashMap<String, String>,
) -> (Vec<Value>, usize) {
    let end = args_extent(tokens, start, paren);
    let mut args = Vec::new();
    let mut i = if paren { start + 1 } else { start };
    let limit = if paren { end - 1 } else { end };
    while i < limit {
        let e = expr_end(tokens, i);
        if e > i {
            args.push(eval_tokens(&tokens[i..e], vars));
        }
        match tokens.get(e) {
            Some(Token::Comma) => i = e + 1,
            _ => break,
        }
    }
    (args, end)
}

/// End (exclusive) of a single expression starting at `start`.
fn expr_end(tokens: &[Token], start: usize) -> usize {
    let mut i = start;
    while i < tokens.len() {
        let t = &tokens[i];
        if i == start && matches!(t, Token::Op(_) | Token::Assign) {
            // An expression cannot start with a bare operator (e.g. the `>`
            // of an unsupported redirection): treat the list as empty.
            break;
        }
        if i > start && is_value_end(&tokens[i - 1]) && is_value_start(t) {
            break;
        }
        match t {
            Token::Semicolon
            | Token::Comma
            | Token::RParen
            | Token::If
            | Token::Else
            | Token::Print
            | Token::Printf
            | Token::Next
            | Token::Begin
            | Token::End
            | Token::Assign
            | Token::Regex(_)
            | Token::Action(_) => break,
            Token::LParen => match find_matching_paren(tokens, i) {
                Some(close) => i = close + 1,
                None => return tokens.len(),
            },
            _ => i += 1,
        }
    }
    i
}

fn is_value_start(t: &Token) -> bool {
    matches!(
        t,
        Token::Var(_) | Token::Number(_) | Token::String(_) | Token::LParen
    )
}

fn is_value_end(t: &Token) -> bool {
    matches!(
        t,
        Token::Var(_) | Token::Number(_) | Token::String(_) | Token::RParen
    )
}

fn find_matching_paren(tokens: &[Token], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (j, t) in tokens.iter().enumerate().skip(open) {
        match t {
            Token::LParen => depth += 1,
            Token::RParen => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
    }
    None
}

fn render_printf(fmt: &str, args: &[Value]) -> String {
    let mut result = String::new();
    let mut arg_idx = 0usize;
    let chars: Vec<char> = fmt.chars().collect();
    let mut j = 0;
    while j < chars.len() {
        if chars[j] == '%' && j + 1 < chars.len() {
            j += 1;
            let spec = chars[j];
            let arg_str = args.get(arg_idx).map(Value::to_str).unwrap_or_default();
            let consumed = match spec {
                's' => {
                    result.push_str(&arg_str);
                    true
                }
                'd' => {
                    if let Ok(n) = arg_str.trim().parse::<i64>() {
                        result.push_str(&n.to_string());
                    } else {
                        result.push_str(&arg_str);
                    }
                    true
                }
                'f' => {
                    if let Ok(n) = arg_str.trim().parse::<f64>() {
                        result.push_str(&fmt_num(n));
                    } else {
                        result.push_str(&arg_str);
                    }
                    true
                }
                '%' => {
                    result.push('%');
                    false
                }
                _ => {
                    result.push('%');
                    result.push(spec);
                    false
                }
            };
            if consumed {
                arg_idx += 1;
            }
        } else {
            result.push(chars[j]);
        }
        j += 1;
    }
    result
}
