mod cli;

use clap::Parser;
use cli::Args;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, IsTerminal};

fn main() {
    let args = Args::parse();
    let mut vars: HashMap<String, i128> = HashMap::new();
    let mut failed = false;
    let mut quit = false;

    if args.mathlib {
        eprintln!("bc: -l is not supported in this build");
    }

    if args.files.is_empty() {
        // Banner only on a real terminal, like GNU bc in interactive mode.
        if io::stdin().is_terminal() {
            println!("bc 0.1.0 (RuxCmd)");
        }
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => match process_line(&line, &mut vars) {
                    Outcome::Quit => break,
                    Outcome::Error => failed = true,
                    Outcome::Ok => {}
                },
                Err(err) => {
                    eprintln!("bc: failed to read input: {err}");
                    failed = true;
                    break;
                }
            }
        }
    } else {
        for path in &args.files {
            match fs::read_to_string(path) {
                Ok(content) => {
                    for line in content.lines() {
                        match process_line(line, &mut vars) {
                            Outcome::Quit => {
                                quit = true;
                                break;
                            }
                            Outcome::Error => failed = true,
                            Outcome::Ok => {}
                        }
                    }
                    if quit {
                        break;
                    }
                }
                Err(_) => {
                    eprintln!("bc: cannot open '{}'", path.display());
                    failed = true;
                }
            }
        }
    }

    if failed {
        std::process::exit(1);
    }
}

enum Outcome {
    Ok,
    Error,
    Quit,
}

fn process_line(line: &str, vars: &mut HashMap<String, i128>) -> Outcome {
    let tokens = match lex(line) {
        Ok(tokens) => tokens,
        Err(LexError::Unsupported(name)) => {
            eprintln!("bc: {name} is not supported in this build");
            return Outcome::Ok;
        }
        Err(LexError::NumberTooLarge) => {
            eprintln!("bc: number too large");
            return Outcome::Error;
        }
        Err(LexError::Syntax) => {
            eprintln!("bc: syntax error");
            return Outcome::Error;
        }
    };

    // A line is a list of statements separated by `;`.
    let mut outcome = Outcome::Ok;
    let mut start = 0usize;
    for (idx, token) in tokens
        .iter()
        .chain(std::iter::once(&Token::Semicolon))
        .enumerate()
    {
        let Token::Semicolon = token else { continue };
        match run_statement(&tokens[start..idx], vars) {
            Flow::Quit => return Outcome::Quit,
            Flow::Error => outcome = Outcome::Error,
            Flow::Ok => {}
        }
        start = idx + 1;
    }
    outcome
}

enum Flow {
    Ok,
    Error,
    Quit,
}

fn run_statement(tokens: &[Token], vars: &mut HashMap<String, i128>) -> Flow {
    if tokens.is_empty() {
        return Flow::Ok;
    }
    if let [Token::Ident(name)] = tokens {
        if name == "quit" || name == "exit" {
            return Flow::Quit;
        }
    }

    let expr = match ExprParser::new(tokens).parse_statement() {
        Ok(expr) => expr,
        Err(StmtError::Unsupported(name)) => {
            eprintln!("bc: {name} is not supported in this build");
            return Flow::Ok;
        }
        Err(StmtError::Syntax) => {
            eprintln!("bc: syntax error");
            return Flow::Error;
        }
    };

    match eval(&expr, vars) {
        // Assignment statements are silent, like bc; expression statements
        // print their value.
        Ok(value) => {
            if !matches!(expr, Expr::Assign(_, _)) {
                println!("{value}");
            }
            Flow::Ok
        }
        Err(EvalError::DivisionByZero) => {
            eprintln!("bc: division by zero");
            Flow::Error
        }
        Err(EvalError::Overflow) => {
            eprintln!("bc: overflow");
            Flow::Error
        }
    }
}

// ---------------------------------------------------------------- lexer ---

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(i128),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
    Assign,
    Semicolon,
}

enum LexError {
    Syntax,
    NumberTooLarge,
    Unsupported(String),
}

/// Keywords whose GNU bc behavior is not implemented in this build.
const RESERVED: &[&str] = &[
    "scale", "ibase", "obase", "define", "if", "while", "for", "return", "auto", "break", "halt",
    "print", "read", "limits",
];

fn lex(line: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\r' => {
                chars.next();
            }
            // Comment: skip the rest of the line.
            '#' => break,
            '0'..='9' => {
                let mut digits = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        digits.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match digits.parse::<i128>() {
                    Ok(value) => tokens.push(Token::Num(value)),
                    Err(_) => return Err(LexError::NumberTooLarge),
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut name = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        name.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if RESERVED.contains(&name.as_str()) {
                    return Err(LexError::Unsupported(name));
                }
                tokens.push(Token::Ident(name));
            }
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }
            '/' => {
                chars.next();
                tokens.push(Token::Slash);
            }
            '%' => {
                chars.next();
                tokens.push(Token::Percent);
            }
            '^' => {
                chars.next();
                tokens.push(Token::Caret);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            '=' => {
                chars.next();
                tokens.push(Token::Assign);
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            _ => return Err(LexError::Syntax),
        }
    }
    Ok(tokens)
}

// ------------------------------------------------------- recursive descent ---

#[derive(Debug)]
enum Expr {
    Num(i128),
    Var(String),
    Assign(String, Box<Expr>),
    Neg(Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
}

enum StmtError {
    Syntax,
    Unsupported(String),
}

struct ExprParser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> ExprParser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn parse_statement(&mut self) -> Result<Expr, StmtError> {
        let expr = self.parse_expr()?;
        if self.pos != self.tokens.len() {
            return Err(StmtError::Syntax);
        }
        Ok(expr)
    }

    /// assignment := IDENT '=' assignment | additive
    fn parse_expr(&mut self) -> Result<Expr, StmtError> {
        if let Some(Token::Ident(name)) = self.peek() {
            if matches!(self.tokens.get(self.pos + 1), Some(Token::Assign)) {
                let name = name.clone();
                self.pos += 2;
                let value = self.parse_expr()?;
                return Ok(Expr::Assign(name, Box::new(value)));
            }
        }
        self.parse_additive()
    }

    /// additive := multiplicative (('+' | '-') multiplicative)*
    fn parse_additive(&mut self) -> Result<Expr, StmtError> {
        let mut lhs = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => break,
            };
            self.pos += 1;
            let rhs = self.parse_multiplicative()?;
            lhs = Expr::Bin(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    /// multiplicative := power (('*' | '/' | '%') power)*
    fn parse_multiplicative(&mut self) -> Result<Expr, StmtError> {
        let mut lhs = self.parse_power()?;
        loop {
            let op = match self.peek() {
                Some(Token::Star) => BinOp::Mul,
                Some(Token::Slash) => BinOp::Div,
                Some(Token::Percent) => BinOp::Rem,
                _ => break,
            };
            self.pos += 1;
            let rhs = self.parse_power()?;
            lhs = Expr::Bin(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    /// power := unary ('^' power)?   (right associative)
    fn parse_power(&mut self) -> Result<Expr, StmtError> {
        let base = self.parse_unary()?;
        if matches!(self.peek(), Some(Token::Caret)) {
            self.pos += 1;
            let exponent = self.parse_power()?;
            return Ok(Expr::Bin(BinOp::Pow, Box::new(base), Box::new(exponent)));
        }
        Ok(base)
    }

    /// unary := '-' unary | atom
    /// Unary minus binds tighter than '^', matching GNU bc (-2^2 == 4).
    fn parse_unary(&mut self) -> Result<Expr, StmtError> {
        if matches!(self.peek(), Some(Token::Minus)) {
            self.pos += 1;
            let inner = self.parse_unary()?;
            return Ok(Expr::Neg(Box::new(inner)));
        }
        self.parse_atom()
    }

    /// atom := NUM | IDENT | '(' expr ')'
    fn parse_atom(&mut self) -> Result<Expr, StmtError> {
        match self.peek() {
            Some(Token::Num(value)) => {
                let value = *value;
                self.pos += 1;
                Ok(Expr::Num(value))
            }
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.pos += 1;
                if matches!(self.peek(), Some(Token::LParen)) {
                    return Err(StmtError::Unsupported(name));
                }
                Ok(Expr::Var(name))
            }
            Some(Token::LParen) => {
                self.pos += 1;
                let expr = self.parse_expr()?;
                if matches!(self.peek(), Some(Token::RParen)) {
                    self.pos += 1;
                    Ok(expr)
                } else {
                    Err(StmtError::Syntax)
                }
            }
            _ => Err(StmtError::Syntax),
        }
    }
}

// ---------------------------------------------------------------- eval ---

enum EvalError {
    DivisionByZero,
    Overflow,
}

fn eval(expr: &Expr, vars: &mut HashMap<String, i128>) -> Result<i128, EvalError> {
    match expr {
        Expr::Num(value) => Ok(*value),
        // Uninitialized variables are 0, like bc.
        Expr::Var(name) => Ok(vars.get(name).copied().unwrap_or(0)),
        Expr::Assign(name, rhs) => {
            let value = eval(rhs, vars)?;
            vars.insert(name.clone(), value);
            Ok(value)
        }
        Expr::Neg(inner) => eval(inner, vars)?.checked_neg().ok_or(EvalError::Overflow),
        Expr::Bin(op, lhs, rhs) => {
            let a = eval(lhs, vars)?;
            let b = eval(rhs, vars)?;
            match op {
                BinOp::Add => a.checked_add(b).ok_or(EvalError::Overflow),
                BinOp::Sub => a.checked_sub(b).ok_or(EvalError::Overflow),
                BinOp::Mul => a.checked_mul(b).ok_or(EvalError::Overflow),
                BinOp::Div => {
                    if b == 0 {
                        return Err(EvalError::DivisionByZero);
                    }
                    a.checked_div(b).ok_or(EvalError::Overflow)
                }
                BinOp::Rem => {
                    if b == 0 {
                        return Err(EvalError::DivisionByZero);
                    }
                    a.checked_rem(b).ok_or(EvalError::Overflow)
                }
                BinOp::Pow => checked_pow(a, b),
            }
        }
    }
}

/// Integer exponentiation with bc scale=0 truncation semantics.
fn checked_pow(base: i128, exponent: i128) -> Result<i128, EvalError> {
    if exponent < 0 {
        // Truncated integer division: 2^-3 is 0 at scale 0.
        return match base {
            0 => Err(EvalError::DivisionByZero),
            1 => Ok(1),
            -1 => Ok(if exponent % 2 == 0 { 1 } else { -1 }),
            _ => Ok(0),
        };
    }
    match base {
        0 => Ok(if exponent == 0 { 1 } else { 0 }),
        1 => Ok(1),
        -1 => Ok(if exponent % 2 == 0 { 1 } else { -1 }),
        _ => {
            let mut result: i128 = 1;
            for _ in 0..exponent {
                result = result.checked_mul(base).ok_or(EvalError::Overflow)?;
            }
            Ok(result)
        }
    }
}
