mod cli;

use clap::Parser;
use cli::Args;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, IsTerminal};

fn main() {
    let args = Args::parse();
    let mut interp = Interp::default();
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
                Ok(line) => match process_line(&line, &mut interp) {
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
                        match process_line(line, &mut interp) {
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

fn process_line(line: &str, interp: &mut Interp) -> Outcome {
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
        match run_statement(&tokens[start..idx], interp) {
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

fn run_statement(tokens: &[Token], interp: &mut Interp) -> Flow {
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

    match eval(&expr, interp) {
        // Assignment statements are silent, like bc; expression statements
        // print their value.
        Ok(value) => {
            if !matches!(expr, Expr::Assign(_, _)) {
                println!("{}", format_value(&value));
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
        Err(EvalError::ScaleRange) => {
            eprintln!("bc: scale must be between 0 and {MAX_SCALE}");
            Flow::Error
        }
        Err(EvalError::NegativeSquareRoot) => {
            eprintln!("bc: square root of negative number");
            Flow::Error
        }
    }
}

// --------------------------------------------------------------- numbers ---
// Values are i128 fixed-point: `digits` holds the number times 10^`scale`,
// and every value carries the number of fractional digits it was produced
// with (like GNU bc, where each number remembers its own scale).

/// Highest assignable `scale`; 10^64 is also where i128 fixed-point runs out
/// of room for most results, so larger values would only produce overflows.
const MAX_SCALE: u32 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Value {
    digits: i128,
    scale: u32,
}

const ZERO: Value = Value {
    digits: 0,
    scale: 0,
};

#[derive(Default)]
struct Interp {
    vars: HashMap<String, Value>,
    scale: u32,
}

/// bc prints a non-zero number with exactly its own `scale` fractional
/// digits (a leading zero before the point, unlike GNU bc's `0.33333` vs
/// `.33333`); zero is always plain `0`.
fn format_value(value: &Value) -> String {
    if value.digits == 0 {
        return "0".to_string();
    }
    if value.scale == 0 {
        return value.digits.to_string();
    }
    let mut magnitude = value.digits.unsigned_abs().to_string();
    let scale = value.scale as usize;
    if magnitude.len() <= scale {
        magnitude.insert_str(0, &"0".repeat(scale - magnitude.len() + 1));
    }
    let split = magnitude.len() - scale;
    let mut out = String::new();
    if value.digits < 0 {
        out.push('-');
    }
    out.push_str(&magnitude[..split]);
    out.push('.');
    out.push_str(&magnitude[split..]);
    out
}

/// 10^k, or None once it no longer fits in an i128 (k > 38).
fn pow10(k: u32) -> Option<i128> {
    if k > 38 {
        return None;
    }
    let mut result: i128 = 1;
    for _ in 0..k {
        result = result.checked_mul(10)?;
    }
    Some(result)
}

fn mul_pow10(digits: i128, k: u32) -> Result<i128, EvalError> {
    match pow10(k) {
        Some(factor) => digits.checked_mul(factor).ok_or(EvalError::Overflow),
        // |digits| < 10^39 always, so only zero survives a bigger shift.
        None if digits == 0 => Ok(0),
        None => Err(EvalError::Overflow),
    }
}

/// The value truncated toward zero to an integer (bc does this whenever an
/// integer is required, e.g. for `scale` and exponents).
fn integer_part(value: Value) -> i128 {
    match pow10(value.scale) {
        Some(factor) => value.digits / factor,
        None => 0,
    }
}

// ---------------------------------------------------------------- lexer ---

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(Value),
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
    "ibase", "obase", "define", "if", "while", "for", "return", "auto", "break", "halt", "print",
    "read", "limits",
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
            '0'..='9' | '.' => {
                let mut int_digits = String::new();
                let mut frac_digits = String::new();
                if c == '.' {
                    chars.next();
                    if !matches!(chars.peek(), Some(d) if d.is_ascii_digit()) {
                        return Err(LexError::Syntax);
                    }
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() {
                            frac_digits.push(d);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                } else {
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() {
                            int_digits.push(d);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if chars.peek() == Some(&'.') {
                        chars.next();
                        while let Some(&d) = chars.peek() {
                            if d.is_ascii_digit() {
                                frac_digits.push(d);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                        if frac_digits.is_empty() {
                            return Err(LexError::Syntax);
                        }
                    }
                }
                let mut digits = int_digits;
                digits.push_str(&frac_digits);
                match digits.parse::<i128>() {
                    Ok(value) => tokens.push(Token::Num(Value {
                        digits: value,
                        scale: frac_digits.len() as u32,
                    })),
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
    Num(Value),
    Var(String),
    Assign(String, Box<Expr>),
    Neg(Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    Sqrt(Box<Expr>),
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

    /// atom := NUM | IDENT | IDENT '(' expr ')' | '(' expr ')'
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
                    // sqrt is the one builtin function this build supports.
                    if name == "sqrt" {
                        self.pos += 1;
                        let inner = self.parse_expr()?;
                        if matches!(self.peek(), Some(Token::RParen)) {
                            self.pos += 1;
                            return Ok(Expr::Sqrt(Box::new(inner)));
                        }
                        return Err(StmtError::Syntax);
                    }
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
    ScaleRange,
    NegativeSquareRoot,
}

fn eval(expr: &Expr, interp: &mut Interp) -> Result<Value, EvalError> {
    match expr {
        Expr::Num(value) => Ok(*value),
        // Uninitialized variables are 0, like bc; `scale` reads the global.
        Expr::Var(name) => {
            if name == "scale" {
                Ok(Value {
                    digits: interp.scale as i128,
                    scale: 0,
                })
            } else {
                Ok(interp.vars.get(name).copied().unwrap_or(ZERO))
            }
        }
        Expr::Assign(name, rhs) => {
            let value = eval(rhs, interp)?;
            if name == "scale" {
                let n = integer_part(value);
                if !(0..=i128::from(MAX_SCALE)).contains(&n) {
                    return Err(EvalError::ScaleRange);
                }
                interp.scale = n as u32;
                Ok(Value {
                    digits: n,
                    scale: 0,
                })
            } else {
                interp.vars.insert(name.clone(), value);
                Ok(value)
            }
        }
        Expr::Neg(inner) => {
            let value = eval(inner, interp)?;
            Ok(Value {
                digits: value.digits.checked_neg().ok_or(EvalError::Overflow)?,
                scale: value.scale,
            })
        }
        Expr::Sqrt(inner) => {
            let value = eval(inner, interp)?;
            sqrt(value, interp.scale)
        }
        Expr::Bin(op, lhs, rhs) => {
            let a = eval(lhs, interp)?;
            let b = eval(rhs, interp)?;
            match op {
                BinOp::Add => add(a, b),
                BinOp::Sub => sub(a, b),
                BinOp::Mul => mul(a, b),
                BinOp::Div => divide(a, b, interp.scale),
                // bc: a % b == a - (a/b)*b with a/b truncated at the current
                // scale, so the sign follows the dividend.
                BinOp::Rem => {
                    if b.digits == 0 {
                        return Err(EvalError::DivisionByZero);
                    }
                    let quotient = divide(a, b, interp.scale)?;
                    sub(a, mul(quotient, b)?)
                }
                BinOp::Pow => pow(a, b, interp.scale),
            }
        }
    }
}

/// Bring both values to the larger of their scales.
fn align(a: Value, b: Value) -> Result<(Value, Value), EvalError> {
    if a.scale < b.scale {
        let digits = mul_pow10(a.digits, b.scale - a.scale)?;
        Ok((
            Value {
                digits,
                scale: b.scale,
            },
            b,
        ))
    } else if b.scale < a.scale {
        let digits = mul_pow10(b.digits, a.scale - b.scale)?;
        Ok((
            a,
            Value {
                digits,
                scale: a.scale,
            },
        ))
    } else {
        Ok((a, b))
    }
}

fn add(a: Value, b: Value) -> Result<Value, EvalError> {
    let (a, b) = align(a, b)?;
    Ok(Value {
        digits: a.digits.checked_add(b.digits).ok_or(EvalError::Overflow)?,
        scale: a.scale,
    })
}

fn sub(a: Value, b: Value) -> Result<Value, EvalError> {
    let (a, b) = align(a, b)?;
    Ok(Value {
        digits: a.digits.checked_sub(b.digits).ok_or(EvalError::Overflow)?,
        scale: a.scale,
    })
}

fn mul(a: Value, b: Value) -> Result<Value, EvalError> {
    Ok(Value {
        digits: a.digits.checked_mul(b.digits).ok_or(EvalError::Overflow)?,
        scale: a.scale.saturating_add(b.scale),
    })
}

/// bc division: the quotient carries exactly `scale` fractional digits and
/// truncates toward zero. digits(q) = trunc(a.digits * 10^(scale + b.scale -
/// a.scale) / b.digits), built by schoolbook long division so no
/// intermediate 10^shift product has to fit in an i128.
fn divide(a: Value, b: Value, scale: u32) -> Result<Value, EvalError> {
    if b.digits == 0 {
        return Err(EvalError::DivisionByZero);
    }
    if a.digits == 0 {
        return Ok(Value { digits: 0, scale });
    }
    let shift = scale as i64 + b.scale as i64 - a.scale as i64;
    let digits = if shift >= 0 {
        let mut q = a.digits.checked_div(b.digits).ok_or(EvalError::Overflow)?;
        let mut r = a.digits.checked_rem(b.digits).ok_or(EvalError::Overflow)?;
        for _ in 0..shift {
            r = r.checked_mul(10).ok_or(EvalError::Overflow)?;
            let digit = r / b.digits;
            q = q
                .checked_mul(10)
                .and_then(|value| value.checked_add(digit))
                .ok_or(EvalError::Overflow)?;
            r %= b.digits;
        }
        q
    } else {
        match pow10((-shift) as u32).and_then(|factor| b.digits.checked_mul(factor)) {
            Some(denominator) => a
                .digits
                .checked_div(denominator)
                .ok_or(EvalError::Overflow)?,
            // The denominator alone overflows i128, so the quotient truncates to 0.
            None => 0,
        }
    };
    Ok(Value { digits, scale })
}

/// bc exponentiation with an integer exponent; a negative exponent yields
/// 1/base^n truncated at the current scale.
fn pow(base: Value, exponent: Value, scale: u32) -> Result<Value, EvalError> {
    // bc requires an integer exponent and truncates fractional ones.
    let n = match pow10(exponent.scale) {
        Some(factor) => exponent.digits / factor,
        None => 0,
    };
    if n == 0 {
        return Ok(Value {
            digits: 1,
            scale: 0,
        });
    }
    if n < 0 {
        let magnitude = pow_positive(base, n.checked_neg().ok_or(EvalError::Overflow)?)?;
        return divide(
            Value {
                digits: 1,
                scale: 0,
            },
            magnitude,
            scale,
        );
    }
    pow_positive(base, n)
}

fn pow_positive(base: Value, exponent: i128) -> Result<Value, EvalError> {
    debug_assert!(exponent > 0);
    // Multiplying a scale-`s` value by itself n times gives scale s*n.
    let scale = base
        .scale
        .saturating_mul(u32::try_from(exponent).unwrap_or(u32::MAX));
    match base.digits {
        0 => Ok(Value {
            digits: 0,
            scale: 0,
        }),
        1 => Ok(Value { digits: 1, scale }),
        -1 => Ok(Value {
            digits: if exponent % 2 == 0 { 1 } else { -1 },
            scale,
        }),
        // |digits| >= 2 overflows i128 within ~128 multiplications, so the
        // loop cannot run away on a huge exponent.
        _ => {
            let mut result = Value {
                digits: 1,
                scale: 0,
            };
            for _ in 0..exponent {
                result = mul(result, base)?;
            }
            Ok(result)
        }
    }
}

/// bc sqrt: the root of `digits/10^scale` truncated at the current scale,
/// via Newton's method on floor(sqrt(n)).
fn sqrt(value: Value, scale: u32) -> Result<Value, EvalError> {
    if value.digits < 0 {
        return Err(EvalError::NegativeSquareRoot);
    }
    // root = isqrt(value.digits * 10^(2*scale - value.scale)).
    let shift = 2 * scale as i64 - value.scale as i64;
    let n = if shift >= 0 {
        mul_pow10(value.digits, shift as u32)?
    } else {
        match pow10((-shift) as u32) {
            // Truncating the radicand first cannot change the truncated root.
            Some(factor) => value.digits / factor,
            None => 0,
        }
    };
    Ok(Value {
        digits: integer_sqrt(n as u128) as i128,
        scale,
    })
}

/// Newton's method for floor(sqrt(n)); every intermediate stays below n, so
/// the arithmetic cannot overflow for an n that fits in u128.
fn integer_sqrt(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let mut x = n;
    let mut y = x.div_ceil(2);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}
