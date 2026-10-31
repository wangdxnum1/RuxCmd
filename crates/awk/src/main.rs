mod cli;

use clap::Parser;
use regex::Regex;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

/// Separator between an array name and its element key inside the variable
/// store, so arrays cannot collide with scalar variables.
const ARRAY_SEP: &str = "\u{1}";

enum OutHandle {
    File(std::fs::File),
    Stdout(std::io::Stdout),
    Stderr(std::io::Stderr),
    Pipe(std::process::Child),
}

impl OutHandle {
    fn write(&mut self, bytes: &[u8]) -> Result<(), String> {
        use std::io::Write;
        match self {
            OutHandle::File(f) => f.write_all(bytes).map_err(|e| e.to_string()),
            OutHandle::Stdout(s) => s.write_all(bytes).map_err(|e| e.to_string()),
            OutHandle::Stderr(s) => s.write_all(bytes).map_err(|e| e.to_string()),
            OutHandle::Pipe(c) => c
                .stdin
                .as_mut()
                .ok_or_else(|| "pipe stdin already closed".to_string())?
                .write_all(bytes)
                .map_err(|e| e.to_string()),
        }
    }

    fn flush(&mut self) -> Result<(), String> {
        use std::io::Write;
        match self {
            OutHandle::File(f) => f.flush().map_err(|e| e.to_string()),
            OutHandle::Stdout(s) => s.flush().map_err(|e| e.to_string()),
            OutHandle::Stderr(s) => s.flush().map_err(|e| e.to_string()),
            OutHandle::Pipe(_) => Ok(()),
        }
    }

    fn close(&mut self) {
        if let OutHandle::Pipe(c) = self {
            c.stdin.take();
            let _ = c.wait();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RedirectMode {
    Truncate,
    Append,
    Pipe,
}

thread_local! {
    /// Open print-redirection targets by destination name (GNU semantics:
    /// a target opened once stays open and subsequent writes append).
    static REDIRECTS: std::cell::RefCell<HashMap<String, OutHandle>> =
        std::cell::RefCell::new(HashMap::new());
    /// User-defined functions: name -> (parameters, body tokens).
    /// Installed once at parse time; fixed for the program duration.
    static FUNCTIONS: std::cell::RefCell<HashMap<String, (Vec<String>, Vec<Token>)>> =
        std::cell::RefCell::new(HashMap::new());
    /// The value of the most recent `return` inside a user function body.
    static RETURN_VALUE: std::cell::RefCell<Value> =
        const { std::cell::RefCell::new(Value::Str(String::new())) };
    /// Recursion guard for user function calls.
    static CALL_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

const MAX_CALL_DEPTH: u32 = 512;

#[derive(Debug, Clone)]
enum PrintDest {
    Target(String, RedirectMode),
}

/// Collect print/printf arguments, splitting off a trailing output
/// redirection (`> file`, `>> file`, `| command`) if present.
fn collect_print_args(
    tokens: &[Token],
    start: usize,
    vars: &mut HashMap<String, String>,
) -> (Vec<Value>, usize, Option<PrintDest>) {
    let mut depth = 0usize;
    let mut redir_at: Option<(usize, RedirectMode)> = None;
    let mut j = start;
    while j < tokens.len() {
        match tokens.get(j) {
            Some(Token::LParen) => depth += 1,
            Some(Token::RParen) => depth = depth.saturating_sub(1),
            Some(Token::Op(op)) if depth == 0 && matches!(op.as_str(), ">" | ">>" | "|") => {
                let mode = match op.as_str() {
                    ">>" => RedirectMode::Append,
                    "|" => RedirectMode::Pipe,
                    _ => RedirectMode::Truncate,
                };
                redir_at = Some((j, mode));
                break;
            }
            Some(t)
                if depth == 0
                    && matches!(
                        t,
                        Token::Semicolon
                            | Token::If
                            | Token::Else
                            | Token::Print
                            | Token::Printf
                            | Token::Next
                            | Token::Begin
                            | Token::End
                            | Token::Action(_)
                    ) =>
            {
                break;
            }
            _ => {}
        }
        j += 1;
    }

    let Some((redir_index, mode)) = redir_at else {
        let (args, next) = collect_args(tokens, start, false, vars);
        return (args, next, None);
    };

    let dest_end = expr_end(tokens, redir_index + 1);
    let dest_tokens = &tokens[redir_index + 1..dest_end];
    let dest_expr = if dest_tokens.is_empty() {
        None
    } else {
        Some(eval_tokens(dest_tokens, vars).to_str())
    };

    let mut args: Vec<Value> = Vec::new();
    let mut seg = start;
    let mut k = start;
    while k < redir_index {
        if matches!(tokens.get(k), Some(Token::Comma)) && depth == 0 {
            if seg < k {
                args.push(eval_tokens(&tokens[seg..k], vars));
            }
            seg = k + 1;
        }
        if matches!(tokens.get(k), Some(Token::LParen)) {
            depth += 1;
        }
        if matches!(tokens.get(k), Some(Token::RParen)) {
            depth = depth.saturating_sub(1);
        }
        k += 1;
    }
    if seg < redir_index {
        args.push(eval_tokens(&tokens[seg..redir_index], vars));
    }

    let dest = dest_expr.map(|d| match mode {
        RedirectMode::Pipe => PrintDest::Target(d, RedirectMode::Pipe),
        m => PrintDest::Target(d, m),
    });
    (args, redir_index.max(start), dest)
}

fn dest_mode(dest: &PrintDest) -> RedirectMode {
    match dest {
        PrintDest::Target(_, mode) => *mode,
        _ => RedirectMode::Truncate,
    }
}

/// Write one print/printf result to the resolved destination.
fn write_output(text: &str, add_newline: bool, dest: Option<&PrintDest>) {
    let payload = if add_newline {
        format!("{}\n", text)
    } else {
        text.to_string()
    };
    let result = match dest {
        None => {
            use std::io::Write;
            let stdout = std::io::stdout();
            let mut lock = stdout.lock();
            lock.write_all(payload.as_bytes())
                .and_then(|()| lock.flush())
                .map_err(|e| e.to_string())
        }
        Some(PrintDest::Target(name, _)) => {
            let mode = dest_mode(dest.unwrap());
            REDIRECTS.with(|r| -> Result<(), String> {
                let mut map = r.borrow_mut();
                if !map.contains_key(name) {
                    let handle = match name.as_str() {
                        "/dev/stdout" => OutHandle::Stdout(std::io::stdout()),
                        "/dev/stderr" => OutHandle::Stderr(std::io::stderr()),
                        pipe if mode == RedirectMode::Pipe => {
                            let child = std::process::Command::new("cmd")
                                .args(["/C", pipe])
                                .stdin(std::process::Stdio::piped())
                                .spawn()
                                .map_err(|e| format!("cannot open pipe '{}': {}", pipe, e))?;
                            OutHandle::Pipe(child)
                        }
                        path => {
                            let file = std::fs::OpenOptions::new()
                                .write(true)
                                .create(true)
                                .append(mode == RedirectMode::Append)
                                .open(path)
                                .map_err(|e| format!("cannot open '{}': {}", path, e))?;
                            OutHandle::File(file)
                        }
                    };
                    map.insert(name.to_string(), handle);
                }
                let handle = map.get_mut(name).unwrap();
                handle.write(payload.as_bytes())?;
                handle.flush()
            })
        }
    };
    if let Err(e) = result {
        eprintln!("awk: print redirection failed: {}", e);
    }
}

fn close_redirects(redirects: &mut HashMap<String, OutHandle>) {
    for handle in redirects.values_mut() {
        handle.close();
    }
    redirects.clear();
}

fn call_user_function(
    name: &str,
    args: Vec<Value>,
    vars: &mut HashMap<String, String>,
) -> Option<Value> {
    let (params, body) = FUNCTIONS.with(|f| f.borrow().get(name).cloned())?;
    let depth = CALL_DEPTH.with(|d| d.get());
    if depth >= MAX_CALL_DEPTH {
        return Some(Value::Str(String::new()));
    }
    CALL_DEPTH.with(|d| d.set(depth + 1));
    eprintln!("DBGF name={:?} params={:?} args={:?}", name, params, args);

    // Parameters are dynamic locals: save their current values, bind the
    // arguments, run the body, then restore. Extra params default to "".
    let mut saved: Vec<(String, Option<String>)> = Vec::new();
    for (idx, param) in params.iter().enumerate() {
        saved.push((param.clone(), vars.get(param).cloned()));
        let value = args.get(idx).cloned().unwrap_or(Value::Str(String::new()));
        vars.insert(param.clone(), value.to_str());
    }

    let flow = execute_block(&body, vars);
    let result = if matches!(flow, Flow::Return) {
        RETURN_VALUE.with(|r| r.borrow().clone())
    } else {
        Value::Str(String::new())
    };

    for (param, old) in saved {
        match old {
            Some(v) => {
                vars.insert(param, v);
            }
            None => {
                vars.remove(&param);
            }
        }
    }
    eprintln!("DBGF result={:?}", result);
    CALL_DEPTH.with(|d| d.set(depth));
    Some(result)
}

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

    // Flush and close any open print-redirection targets.
    REDIRECTS.with(|r| close_redirects(&mut r.borrow_mut()));

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
            if !match_pattern(pattern, line, vars) {
                continue;
            }
            let flow = execute_block(action, vars);
            if matches!(flow, Flow::Next | Flow::Return) {
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
    Delete,
    In,
    Function,
    Return,
    Index(String, Vec<Token>),
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
                for ch in chars.by_ref() {
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
                for ch in chars.by_ref() {
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
                    } else if op == ">" && next == '>' {
                        op.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                // Postfix/prefix increment and decrement.
                if (op == "+" || op == "-") && chars.peek() == Some(&c) {
                    chars.next();
                    op.push(c);
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
                    "delete" => tokens.push(Token::Delete),
                    "function" | "func" => tokens.push(Token::Function),
                    "return" => tokens.push(Token::Return),
                    "in" => tokens.push(Token::In),
                    _ => {
                        // arr[expr] indexes an array element.
                        if chars.peek() == Some(&'[') {
                            chars.next();
                            let mut depth = 1usize;
                            let mut inner = Vec::new();
                            for ch in chars.by_ref() {
                                if ch == '[' {
                                    depth += 1;
                                } else if ch == ']' {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                }
                                inner.push(ch);
                            }
                            tokens.push(Token::Index(
                                word,
                                tokenize(&inner.into_iter().collect::<String>()),
                            ));
                            continue;
                        }
                        tokens.push(Token::Var(word))
                    }
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
    /// `/ERE/` compiled once at parse time (extended regular expressions,
    /// like gawk/mawk).
    Regex(Regex),
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
    let mut parsed = ParsedProgram {
        begin_actions: Vec::new(),
        pattern_actions: Vec::new(),
        end_actions: Vec::new(),
    };
    FUNCTIONS.with(|f| f.borrow_mut().clear());

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Function => {
                // function name(param, ...) { body }
                let Some(Token::Var(name)) = tokens.get(i + 1) else {
                    return Err("awk: function: expected a name".to_string());
                };
                let name = name.clone();
                let Some(Token::LParen) = tokens.get(i + 2) else {
                    return Err(format!("awk: function {}: expected '('", name));
                };
                let Some(close) = find_matching_paren(tokens, i + 2) else {
                    return Err(format!("awk: function {}: expected ')'", name));
                };
                let mut params = Vec::new();
                for t in &tokens[i + 3..close] {
                    if let Token::Var(v) = t {
                        if !v.starts_with('$') {
                            params.push(v.clone());
                        }
                    }
                }
                let Some(Token::Action(body)) = tokens.get(close + 1) else {
                    return Err(format!("awk: function {}: expected '{{'", name));
                };
                FUNCTIONS.with(|f| f.borrow_mut().insert(name.clone(), (params, body.to_vec())));
                eprintln!(
                    "DBGParsed fn={:?} all={:?}",
                    name,
                    FUNCTIONS.with(|f| f.borrow().keys().cloned().collect::<Vec<_>>())
                );
                i = close + 2;
            }
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
                let compiled = compile_ere(r)?;
                if let Some(Token::Action(a)) = tokens.get(i + 1) {
                    parsed
                        .pattern_actions
                        .push((Pattern::Regex(compiled), a.clone()));
                    i += 2;
                } else {
                    // Pattern without an action defaults to `print`.
                    parsed
                        .pattern_actions
                        .push((Pattern::Regex(compiled), vec![Token::Print]));
                    i += 1;
                }
            }
            Token::Var(_) => {
                // Expression pattern: collect tokens up to the action block
                // so forms like `$1 == "a" { ... }` parse as a single rule.
                let end = expr_end(tokens, i);
                let expr: Vec<Token> = tokens[i..end].to_vec();
                if let Some(Token::Action(a)) = tokens.get(end) {
                    parsed
                        .pattern_actions
                        .push((Pattern::Expr(expr), a.clone()));
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

/// Compile one `/ERE/` pattern; errors surface at parse time like gawk.
fn compile_ere(pattern: &str) -> Result<Regex, String> {
    Regex::new(pattern)
        .map_err(|e| format!("awk: /{}/: invalid regular expression ({})", pattern, e))
}

fn match_pattern(pattern: &Pattern, line: &str, vars: &mut HashMap<String, String>) -> bool {
    match pattern {
        Pattern::Always => true,
        Pattern::Regex(re) => re.is_match(line),
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
        a.to_num()
            .partial_cmp(&b.to_num())
            .unwrap_or(Ordering::Equal)
    } else {
        a.to_str().cmp(&b.to_str())
    }
}

fn bool_val(b: bool) -> Value {
    Value::Num(if b { 1.0 } else { 0.0 })
}

fn eval_tokens(tokens: &[Token], vars: &mut HashMap<String, String>) -> Value {
    let mut stack: Vec<Value> = Vec::new();
    let mut ops: Vec<String> = Vec::new();

    let mut index = 0usize;
    let mut last_was_value = false;
    while index < tokens.len() {
        let token = &tokens[index];
        if last_was_value && is_value_start(token) {
            while let Some(top) = ops.last() {
                if precedence(top) >= precedence("concat") {
                    let pending = ops.pop().unwrap();
                    apply_op(&mut stack, &pending);
                } else {
                    break;
                }
            }
            ops.push("concat".to_string());
        }
        match token {
            Token::Number(n) => {
                stack.push(Value::Num(*n));
                last_was_value = true;
                index += 1;
            }
            Token::String(s) => {
                stack.push(Value::Str(s.clone()));
                last_was_value = true;
                index += 1;
            }
            Token::Var(v) => {
                // Builtin function call: name followed by an argument list.
                let is_builtin = matches!(
                    v.as_str(),
                    "length"
                        | "substr"
                        | "index"
                        | "sprintf"
                        | "toupper"
                        | "tolower"
                        | "sub"
                        | "gsub"
                        | "split"
                        | "close"
                );
                let is_user_fn = FUNCTIONS.with(|f| f.borrow().contains_key(v));
                if matches!(tokens.get(index + 1), Some(Token::LParen))
                    && (is_builtin || is_user_fn)
                {
                    let Some(close) = find_matching_paren(tokens, index + 1) else {
                        stack.push(Value::Str(String::new()));
                        index += 1;
                        continue;
                    };
                    let arg_tokens = &tokens[index + 2..close];
                    // split's second argument is a bare array name.
                    if v == "split" {
                        // find the second top-level comma segment's first token
                        let mut commas = 0usize;
                        let mut depth = 0usize;
                        let mut second_start = None;
                        for (j, t) in arg_tokens.iter().enumerate() {
                            match t {
                                Token::LParen => depth += 1,
                                Token::RParen => depth = depth.saturating_sub(1),
                                Token::Comma if depth == 0 => {
                                    commas += 1;
                                    if commas == 1 {
                                        second_start = Some(j + 1);
                                    }
                                }
                                _ => {}
                            }
                        }
                        if let Some(ts) = second_start
                            && let Some(Token::Var(name)) = arg_tokens.get(ts)
                        {
                            vars.insert("\u{0}split_arr".to_string(), name.clone());
                        }
                    }
                    // sub/gsub take an lvalue as their optional third
                    // argument; record its name before values are evaluated.
                    if matches!(v.as_str(), "sub" | "gsub") {
                        let mut commas = 0usize;
                        let mut depth = 0usize;
                        let mut third_start = None;
                        for (j, t) in arg_tokens.iter().enumerate() {
                            match t {
                                Token::LParen => depth += 1,
                                Token::RParen => depth = depth.saturating_sub(1),
                                Token::Comma if depth == 0 => {
                                    commas += 1;
                                    if commas == 2 {
                                        third_start = Some(j + 1);
                                    }
                                }
                                _ => {}
                            }
                        }
                        if let Some(ts) = third_start
                            && let Some(Token::Var(name)) = arg_tokens.get(ts)
                        {
                            vars.insert("\u{0}sub_target".to_string(), name.clone());
                        }
                    }
                    let eval_arg =
                        |segment: &[Token], vars: &mut HashMap<String, String>| -> Value {
                            // A bare /regex/ argument passes its source text.
                            if let [Token::Regex(r)] = segment {
                                Value::Str(r.clone())
                            } else {
                                eval_tokens(segment, vars)
                            }
                        };
                    let mut args: Vec<Value> = Vec::new();
                    let mut start = 0usize;
                    for (j, t) in arg_tokens.iter().enumerate() {
                        if matches!(t, Token::Comma) {
                            args.push(eval_arg(&arg_tokens[start..j], vars));
                            start = j + 1;
                        }
                    }
                    if start <= arg_tokens.len() {
                        args.push(eval_arg(&arg_tokens[start..], vars));
                    }
                    if v == "close" {
                        if let Some(target) = args.first() {
                            let name = target.to_str();
                            let removed = REDIRECTS.with(|r| {
                                eprintln!(
                                    "DBGR close key={:?} keys={:?}",
                                    name,
                                    r.borrow().keys().cloned().collect::<Vec<_>>()
                                );
                                r.borrow_mut().remove(&name).map(|mut h| {
                                    h.close();
                                })
                            });
                            stack.push(Value::Num(if removed.is_some() { 0.0 } else { -1.0 }));
                        } else {
                            stack.push(Value::Num(-1.0));
                        }
                        index = close + 1;
                        continue;
                    }
                    if FUNCTIONS.with(|f| f.borrow().contains_key(v)) {
                        stack.push(
                            call_user_function(v, args, vars)
                                .unwrap_or_else(|| Value::Str(String::new())),
                        );
                    } else {
                        stack.push(call_builtin(v, args, vars));
                    }
                    index = close + 1;
                    continue;
                }
                // Dynamic field reference: `$name` that is not a cached
                // field evaluates the name as a number and reads $<n>.
                if let Some(inner) = v.strip_prefix('$')
                    && !inner.is_empty()
                    && !vars.contains_key(v)
                    && vars.contains_key(inner)
                {
                    let n = vars
                        .get(inner)
                        .and_then(|s| s.trim().parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let key = if n.fract() == 0.0 && n >= 0.0 {
                        format!("${}", n as u64)
                    } else {
                        "$0".to_string()
                    };
                    stack.push(Value::Str(vars.get(&key).cloned().unwrap_or_default()));
                    index += 1;
                    continue;
                }
                // Bare `length` means length($0) like POSIX awk.
                if v == "length" {
                    stack.push(Value::Num(
                        vars.get("$0").map(|s| s.chars().count()).unwrap_or(0) as f64,
                    ));
                    index += 1;
                    continue;
                }
                stack.push(Value::Str(vars.get(v).cloned().unwrap_or_default()));
                last_was_value = true;
                index += 1;
            }
            Token::Index(name, key_tokens) => {
                let key = eval_tokens(key_tokens, vars).to_str();
                let full = format!("{}{}{}", name, ARRAY_SEP, key);
                stack.push(Value::Str(vars.get(&full).cloned().unwrap_or_default()));
                index += 1;
            }
            Token::In => {
                // `key in arr`: the left value was pushed; the right side is
                // the bare array name.
                let Some(Token::Var(arr)) = tokens.get(index + 1) else {
                    stack.push(Value::Num(0.0));
                    index += 1;
                    continue;
                };
                let key = stack.pop().map(|v| v.to_str()).unwrap_or_default();
                let present = vars.contains_key(&format!("{}{}{}", arr, ARRAY_SEP, key));
                stack.push(bool_val(present));
                index += 2;
            }
            Token::Op(op) => {
                last_was_value = false;
                while let Some(top) = ops.last() {
                    if precedence(top) >= precedence(op) {
                        let pending = ops.pop().unwrap();
                        apply_op(&mut stack, &pending);
                    } else {
                        break;
                    }
                }
                ops.push(op.clone());
                index += 1;
            }
            Token::LParen => {
                ops.push("(".to_string());
                last_was_value = false;
                index += 1;
            }
            Token::RParen => {
                while let Some(op) = ops.pop() {
                    if op == "(" {
                        break;
                    }
                    apply_op(&mut stack, &op);
                }
                last_was_value = true;
                index += 1;
            }
            _ => {
                index += 1;
            }
        }
    }

    while let Some(op) = ops.pop() {
        if op == "(" {
            break;
        }
        apply_op(&mut stack, &op);
    }

    stack.pop().unwrap_or(Value::Str(String::new()))
}

/// Dispatch one builtin function call. `split` requires arrays, which this
/// port does not have yet; it evaluates to the empty string (declared
/// difference).
fn call_builtin(name: &str, args: Vec<Value>, vars: &mut HashMap<String, String>) -> Value {
    let arg = |i: usize| args.get(i).map(Value::to_str).unwrap_or_default();
    match name {
        "length" => Value::Num(arg(0).chars().count() as f64),
        "substr" => {
            let text = arg(0);
            let chars: Vec<char> = text.chars().collect();
            let m = args.get(1).map(|v| v.to_num()).unwrap_or(1.0);
            if m < 1.0 {
                // gawk: positions before 1 clip to the start; the count
                // applies from there unchanged (substr("abc", 0, 2) == "ab").
                let n = args.get(2).map(|v| v.to_num());
                let take = match n {
                    Some(n) if n.is_finite() => (n.max(0.0) as usize).min(chars.len()),
                    _ => chars.len(),
                };
                return Value::Str(chars[..take].iter().collect());
            }
            let start = ((m as usize).saturating_sub(1)).min(chars.len());
            let take = match args.get(2).map(|v| v.to_num()) {
                Some(n) if n.is_finite() => (n.max(0.0) as usize).min(chars.len() - start),
                Some(_) => 0,
                None => chars.len() - start,
            };
            Value::Str(chars[start..start + take].iter().collect())
        }
        "index" => {
            let hay = arg(0);
            let needle = arg(1);
            Value::Num(match hay.find(&needle) {
                Some(byte_pos) => hay[..byte_pos].chars().count() as f64 + 1.0,
                None => 0.0,
            })
        }
        "sprintf" => {
            let fmt = arg(0);
            let rest: Vec<String> = args.iter().skip(1).map(Value::to_str).collect();
            let mut bytes = Vec::new();
            let mut stop = false;
            let _ = printf::run_format(&mut bytes, &fmt, &rest, &mut stop);
            Value::Str(String::from_utf8_lossy(&bytes).into_owned())
        }
        "toupper" => Value::Str(arg(0).to_uppercase()),
        "tolower" => Value::Str(arg(0).to_lowercase()),
        "split" => {
            let source = arg(0);
            let arr = vars.get("\u{0}split_arr").cloned().unwrap_or_default();
            vars.remove("\u{0}split_arr");
            if arr.is_empty() {
                return Value::Num(0.0);
            }
            let fs = if args.len() >= 3 {
                arg(2)
            } else {
                vars.get("FS").cloned().unwrap_or_else(|| " ".to_string())
            };
            let parts: Vec<String> = if fs == " " {
                source.split_whitespace().map(str::to_string).collect()
            } else if fs.is_empty() {
                source.chars().map(|c| c.to_string()).collect()
            } else {
                source.split(&fs).map(str::to_string).collect()
            };
            for (j, part) in parts.iter().enumerate() {
                vars.insert(format!("{}{}{}", arr, ARRAY_SEP, j + 1), part.clone());
            }
            Value::Num(parts.len() as f64)
        }
        "sub" | "gsub" => {
            // The third argument names the lvalue; eval passes values only,
            // so the caller stashes the target key beforehand.
            let target_key = vars
                .get("\u{0}sub_target")
                .cloned()
                .unwrap_or_else(|| "$0".to_string());
            vars.remove("\u{0}sub_target");
            let current = vars.get(&target_key).cloned().unwrap_or_default();
            let Ok(re) = Regex::new(&arg(0)) else {
                return Value::Num(0.0);
            };
            let global = name == "gsub";
            let (new_value, count) = awk_sub(&current, &re, &arg(1), global);
            let was_record = target_key == "$0";
            vars.insert(target_key, new_value);
            if was_record {
                resplit_fields(vars);
            }
            Value::Num(count as f64)
        }
        _ => Value::Str(String::new()),
    }
}

/// awk sub/gsub replacement: `&` inserts the whole match, `\&` is literal.
fn awk_sub(text: &str, re: &Regex, replacement: &str, global: bool) -> (String, usize) {
    let mut count = 0usize;
    let mut out = String::new();
    let mut last = 0usize;
    for m in re.find_iter(text) {
        count += 1;
        out.push_str(&text[last..m.start()]);
        for ch in replacement.chars() {
            if ch == '&' {
                out.push_str(m.as_str());
            } else {
                out.push(ch);
            }
        }
        last = m.end();
        if !global {
            break;
        }
    }
    out.push_str(&text[last..]);
    (out, count)
}

/// Re-split $0 into $1..$NF with the current FS after an in-place edit.
fn resplit_fields(vars: &mut HashMap<String, String>) {
    let line = vars.get("$0").cloned().unwrap_or_default();
    let fs = vars.get("FS").cloned().unwrap_or_else(|| " ".to_string());
    let fields: Vec<&str> = if fs == " " {
        line.split_whitespace().collect()
    } else {
        line.split(&fs).collect()
    };
    vars.retain(|k, _| !(k.starts_with('$') && k != "$0"));
    vars.insert("NF".to_string(), fields.len().to_string());
    for (j, f) in fields.iter().enumerate() {
        vars.insert(format!("${}", j + 1), (*f).to_string());
    }
}

fn precedence(op: &str) -> i32 {
    match op {
        "*" | "/" => 2,
        "+" | "-" => 1,
        "concat" => 0,
        "<" | ">" | "<=" | ">=" | "==" | "!=" => -1,
        "&&" => -2,
        "||" => -3,
        "(" => -4,
        _ => -5,
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
        "concat" => Value::Str(format!("{}{}", a.to_str(), b.to_str())),
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
    Return,
}

fn execute_block(tokens: &[Token], vars: &mut HashMap<String, String>) -> Flow {
    execute_block_inner(tokens, vars)
}

fn execute_block_inner(tokens: &[Token], vars: &mut HashMap<String, String>) -> Flow {
    let mut i = 0;

    while i < tokens.len() {
        match &tokens[i] {
            Token::Print => {
                let (args, next, dest) = collect_print_args(tokens, i + 1, vars);
                let rendered = if args.is_empty() {
                    vars.get("$0").cloned().unwrap_or_default()
                } else {
                    args.iter()
                        .map(Value::to_str)
                        .collect::<Vec<_>>()
                        .join(&vars["OFS"])
                };
                write_output(&rendered, true, dest.as_ref());
                i = next;
            }
            Token::Printf => {
                let (args, next, dest) = collect_print_args(tokens, i + 1, vars);
                if let Some(fmt) = args.first() {
                    let rest: Vec<String> = args[1..].iter().map(Value::to_str).collect();
                    let mut bytes = Vec::new();
                    let mut stop = false;
                    let _ = printf::run_format(&mut bytes, &fmt.to_str(), &rest, &mut stop);
                    write_output(&String::from_utf8_lossy(&bytes), false, dest.as_ref());
                }
                i = next;
            }
            Token::If => {
                let (next_i, flow) = execute_if(tokens, i, vars);
                if matches!(flow, Flow::Next | Flow::Return) {
                    return flow;
                }
                i = next_i;
            }
            Token::For => {
                let (next_i, flow) = execute_for(tokens, i, vars);
                if matches!(flow, Flow::Next | Flow::Return) {
                    return flow;
                }
                i = next_i;
            }
            Token::Next => return Flow::Next,
            Token::Return => {
                // return [expr] — the value lands in RETURN_VALUE for the
                // enclosing user-function call; a bare return yields "".
                let i2 = i + 1;
                let value = if !matches!(
                    tokens.get(i2),
                    None | Some(Token::Semicolon) | Some(Token::Action(_))
                ) {
                    let end = expr_end(tokens, i2);
                    eval_tokens(&tokens[i2..end], vars)
                } else {
                    Value::Str(String::new())
                };
                RETURN_VALUE.with(|r| *r.borrow_mut() = value);
                return Flow::Return;
            }
            Token::Index(name, key_tokens) => {
                if matches!(tokens.get(i + 1), Some(Token::Assign)) {
                    let key = eval_tokens(key_tokens, vars).to_str();
                    let end = expr_end(tokens, i + 2);
                    let val = eval_tokens(&tokens[i + 2..end], vars).to_str();
                    vars.insert(format!("{}{}{}", name, ARRAY_SEP, key), val);
                    i = end;
                } else if let Some(Token::Op(op)) = tokens.get(i + 1)
                    && matches!(op.as_str(), "+=" | "-=" | "*=" | "/=")
                {
                    let key = eval_tokens(key_tokens, vars).to_str();
                    let full = format!("{}{}{}", name, ARRAY_SEP, key);
                    let end = expr_end(tokens, i + 2);
                    let rhs = eval_tokens(&tokens[i + 2..end], vars);
                    let cur = vars
                        .get(&full)
                        .and_then(|s| s.trim().parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let val = match op.as_str() {
                        "+=" => Value::Num(cur + rhs.to_num()),
                        "-=" => Value::Num(cur - rhs.to_num()),
                        "*=" => Value::Num(cur * rhs.to_num()),
                        _ => {
                            let d = rhs.to_num();
                            if d == 0.0 {
                                Value::Num(0.0)
                            } else {
                                Value::Num(cur / d)
                            }
                        }
                    };
                    vars.insert(full, val.to_str());
                    i = end;
                } else {
                    i += 1;
                }
            }
            Token::Delete => {
                // delete arr[key] or delete arr
                match tokens.get(i + 1) {
                    Some(Token::Index(name, key_tokens)) => {
                        let key = eval_tokens(key_tokens, vars).to_str();
                        vars.remove(&format!("{}{}{}", name, ARRAY_SEP, key));
                        i += 2;
                    }
                    Some(Token::Var(name)) => {
                        let prefix = format!("{}{}", name, ARRAY_SEP);
                        vars.retain(|k, _| !k.starts_with(&prefix));
                        i += 2;
                    }
                    _ => i += 1,
                }
            }
            Token::Var(v) => {
                if matches!(tokens.get(i + 1), Some(Token::Assign)) {
                    let name = v.clone();
                    let end = expr_end(tokens, i + 2);
                    let val = eval_tokens(&tokens[i + 2..end], vars).to_str();
                    vars.insert(name, val);
                    i = end;
                } else if matches!(tokens.get(i + 1), Some(Token::LParen))
                    && matches!(v.as_str(), "sub" | "gsub" | "split" | "close")
                {
                    // A bare sub/gsub/split statement runs for its side effect.
                    let end = args_extent(tokens, i + 1, true);
                    let _ = eval_tokens(&tokens[i..end], vars);
                    i = end;
                } else if let Some(Token::Op(op)) = tokens.get(i + 1)
                    && matches!(op.as_str(), "+=" | "-=" | "*=" | "/=")
                {
                    // Compound assignment: s += expr and friends.
                    let end = expr_end(tokens, i + 2);
                    let rhs = eval_tokens(&tokens[i + 2..end], vars);
                    let cur = vars
                        .get(v)
                        .and_then(|s| s.trim().parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let val = match op.as_str() {
                        "+=" => Value::Num(cur + rhs.to_num()),
                        "-=" => Value::Num(cur - rhs.to_num()),
                        "*=" => Value::Num(cur * rhs.to_num()),
                        _ => {
                            let d = rhs.to_num();
                            if d == 0.0 {
                                Value::Num(0.0)
                            } else {
                                Value::Num(cur / d)
                            }
                        }
                    };
                    vars.insert(v.clone(), val.to_str());
                    i = end;
                } else if let Some(Token::Op(op)) = tokens.get(i + 1) {
                    // i++ / i-- statements (and prefixes within expressions).
                    match op.as_str() {
                        "++" => {
                            let n = vars
                                .get(v)
                                .and_then(|s| s.trim().parse::<f64>().ok())
                                .unwrap_or(0.0);
                            vars.insert(v.clone(), fmt_num(n + 1.0));
                            i += 2;
                        }
                        "--" => {
                            let n = vars
                                .get(v)
                                .and_then(|s| s.trim().parse::<f64>().ok())
                                .unwrap_or(0.0);
                            vars.insert(v.clone(), fmt_num(n - 1.0));
                            i += 2;
                        }
                        _ => i += 1,
                    }
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
fn execute_if(tokens: &[Token], start: usize, vars: &mut HashMap<String, String>) -> (usize, Flow) {
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

/// Executes the three-part `for (init; cond; step) body` statement.
/// Arrays and `for (k in arr)` remain unsupported.
fn execute_for(
    tokens: &[Token],
    start: usize,
    vars: &mut HashMap<String, String>,
) -> (usize, Flow) {
    let Some(open) = tokens
        .get(start + 1)
        .filter(|t| matches!(t, Token::LParen))
        .map(|_| start + 1)
    else {
        return (start + 1, Flow::Normal);
    };
    // Top-level semicolons inside the header.
    let mut depth = 0usize;
    let mut seps: Vec<usize> = Vec::new();
    let mut close = None;
    for (j, t) in tokens.iter().enumerate().skip(open) {
        match t {
            Token::LParen => depth += 1,
            Token::RParen => {
                if depth == 1 {
                    close = Some(j);
                    break;
                }
                depth -= 1;
            }
            Token::Semicolon if depth == 1 => seps.push(j),
            _ => {}
        }
    }
    let (Some(close_paren), _) = (close, ()) else {
        return (start + 1, Flow::Normal);
    };
    // `for (k in arr)` form: no semicolons, Var In Var header.
    if seps.len() < 2 {
        if matches!(tokens.get(open + 1), Some(Token::Var(_)))
            && matches!(tokens.get(open + 2), Some(Token::In))
            && matches!(tokens.get(open + 3), Some(Token::Var(_arr)))
        {
            let Some(Token::Var(key_var)) = tokens.get(open + 1) else {
                return (close_paren + 1, Flow::Normal);
            };
            let Some(Token::Var(arr)) = tokens.get(open + 3) else {
                return (close_paren + 1, Flow::Normal);
            };
            let prefix = format!("{}{}", arr, ARRAY_SEP);
            let mut keys: Vec<String> = vars
                .keys()
                .filter(|k| k.starts_with(&prefix))
                .map(|k| k[prefix.len()..].to_string())
                .collect();
            keys.sort();
            let body_start = close_paren + 1;
            let body_end = statement_end(tokens, body_start);
            let body = &tokens[body_start..body_end];
            for key in keys {
                vars.insert(key_var.clone(), key);
                let flow = execute_block(body, vars);
                if flow == Flow::Next {
                    return (body_end, Flow::Next);
                }
            }
            return (body_end, Flow::Normal);
        }
        return (close_paren + 1, Flow::Normal);
    }
    let init = &tokens[open + 1..seps[0]];
    let cond = &tokens[seps[0] + 1..seps[1]];
    let step = &tokens[seps[1] + 1..close_paren];

    let body_start = close_paren + 1;
    let body_end = statement_end(tokens, body_start);
    let body = &tokens[body_start..body_end];

    if !init.is_empty() {
        execute_block(init, vars);
    }
    while is_truthy(&eval_tokens(cond, vars)) {
        let flow = execute_block(body, vars);
        if flow == Flow::Next {
            // `next` abandons the whole rule set for this record.
            return (body_end, Flow::Next);
        }
        if !step.is_empty() {
            execute_block(step, vars);
        }
    }
    (body_end, Flow::Normal)
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
        Some(Token::For) => for_end(tokens, i),
        Some(Token::Next) | Some(Token::Action(_)) => i + 1,
        Some(Token::Return) => expr_end(tokens, i + 1),
        Some(Token::Index(_, _)) => {
            if matches!(tokens.get(i + 1), Some(Token::Assign)) {
                expr_end(tokens, i + 2)
            } else {
                i + 1
            }
        }
        Some(Token::Delete) => match tokens.get(i + 1) {
            Some(Token::Index(_, _)) => i + 2,
            Some(Token::Var(_)) => i + 2,
            _ => i + 1,
        },
        Some(Token::Var(_)) => {
            if matches!(tokens.get(i + 1), Some(Token::Assign)) {
                expr_end(tokens, i + 2)
            } else if let Some(Token::Op(op)) = tokens.get(i + 1)
                && matches!(op.as_str(), "++" | "--" | "+=" | "-=" | "*=" | "/=")
            {
                if op == "++" || op == "--" {
                    i + 2
                } else {
                    expr_end(tokens, i + 2)
                }
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

/// End (exclusive) of the whole for statement starting at `start`.
fn for_end(tokens: &[Token], start: usize) -> usize {
    let Some(open) = tokens
        .get(start + 1)
        .filter(|t| matches!(t, Token::LParen))
        .map(|_| start + 1)
    else {
        return start + 1;
    };
    let mut depth = 0usize;
    let mut close = None;
    for (j, t) in tokens.iter().enumerate().skip(open) {
        match t {
            Token::LParen => depth += 1,
            Token::RParen => {
                if depth == 1 {
                    close = Some(j);
                    break;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    let Some(close_paren) = close else {
        return start + 1;
    };
    statement_end(tokens, close_paren + 1)
}

/// End (exclusive) of the comma-separated argument list starting at
/// `start`. Depth counts from zero: commas at depth zero split arguments,
/// and a closing parenthesis at depth zero ends the list unless a comma
/// follows it (`print (1), (2)` keeps going).
fn args_extent(tokens: &[Token], start: usize, paren: bool) -> usize {
    let mut i = start;
    let mut depth = 0usize;
    while i < tokens.len() {
        match tokens.get(i) {
            Some(Token::LParen) => depth += 1,
            Some(Token::RParen) => {
                depth = depth.saturating_sub(1);
                if depth == 0 && !matches!(tokens.get(i + 1), Some(Token::Comma)) {
                    return if paren { i + 1 } else { i };
                }
            }
            Some(t)
                if depth == 0
                    && matches!(
                        t,
                        Token::Semicolon
                            | Token::If
                            | Token::Else
                            | Token::Print
                            | Token::Printf
                            | Token::Next
                            | Token::Begin
                            | Token::End
                            | Token::Action(_)
                    ) =>
            {
                return i;
            }
            _ => {}
        }
        i += 1;
    }
    i
}

/// Collects and evaluates the comma-separated argument list starting at
/// `start`. Returns the values plus the index after the whole list.
fn collect_args(
    tokens: &[Token],
    start: usize,
    paren: bool,
    vars: &mut HashMap<String, String>,
) -> (Vec<Value>, usize) {
    let mut i = start;
    let mut depth = 0usize;
    let mut args: Vec<Value> = Vec::new();
    let mut seg = i;
    while i < tokens.len() {
        match tokens.get(i) {
            Some(Token::LParen) => depth += 1,
            Some(Token::RParen) => {
                depth = depth.saturating_sub(1);
                // Call-style list: this RParen closes the call. Wrapper-less
                // print lists keep the RParen inside the current segment.
                if depth == 0 && paren {
                    if seg < i {
                        args.push(eval_tokens(&tokens[seg..i], vars));
                    }
                    return (args, i + 1);
                }
            }
            Some(Token::Comma) if depth == usize::from(paren) => {
                if seg < i {
                    args.push(eval_tokens(&tokens[seg..i], vars));
                }
                seg = i + 1;
            }
            Some(t)
                if depth == usize::from(paren)
                    && matches!(
                        t,
                        Token::Semicolon
                            | Token::If
                            | Token::Else
                            | Token::Print
                            | Token::Printf
                            | Token::Next
                            | Token::Begin
                            | Token::End
                            | Token::Action(_)
                    ) =>
            {
                if seg < i {
                    args.push(eval_tokens(&tokens[seg..i], vars));
                }
                return (args, i);
            }
            _ => {}
        }
        i += 1;
    }
    if seg < i {
        args.push(eval_tokens(&tokens[seg..i], vars));
    }
    (args, i)
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
        // A `name (` sequence is a function call, not concatenation: the
        // LParen must be consumed with its matching close below.
        if i > start
            && is_value_end(&tokens[i - 1])
            && is_value_start(t)
            && !matches!(t, Token::LParen)
        {
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
            | Token::Action(_) => break,
            Token::LParen => match find_matching_paren(tokens, i) {
                Some(close) => i = close + 1,
                None => return tokens.len(),
            },
            Token::In => i += 1,
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
