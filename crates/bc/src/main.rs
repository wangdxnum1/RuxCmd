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
    let mut outcome = Outcome::Ok;
    // A line is a list of statements separated by `;`; `#` comments out the
    // rest of the line.  Each statement is lexed just before it runs so an
    // earlier `ibase=` on the same line changes how later numbers are read,
    // like GNU bc (`ibase=16; FF` prints 255).
    let mut rest = line;
    loop {
        let cut = rest.find([';', '#']);
        let (segment, tail) = match cut {
            Some(i) if rest.as_bytes()[i] == b';' => (&rest[..i], Some(&rest[i + 1..])),
            _ => (rest, None),
        };
        match run_segment(segment, interp) {
            Flow::Quit => return Outcome::Quit,
            Flow::Error => outcome = Outcome::Error,
            Flow::Ok => {}
        }
        match tail {
            Some(next) => rest = next,
            None => return outcome,
        }
    }
}

enum Flow {
    Ok,
    Error,
    Quit,
}

/// Evaluates one `;`-separated segment of a bc line: lexes it, then runs the
/// statement; the resulting [`Flow`] tells `process_line` whether to keep
/// going or stop.
fn run_segment(segment: &str, interp: &mut Interp) -> Flow {
    // Lex with the ibase in effect right now so later segments of the same
    // line read their numbers in the updated base (`ibase=16; FF` → 255).
    match lex(segment, interp.ibase) {
        Ok(tokens) => run_statement(&tokens, interp),
        // Unimplemented keywords report and let the program continue with
        // exit status 0, like unsupported function calls (`foo(3)`).
        Err(LexError::Unsupported(name)) => {
            eprintln!("bc: {name} is not supported in this build");
            Flow::Ok
        }
        Err(_) => {
            eprintln!("bc: lex error");
            Flow::Error
        }
    }
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
                println!("{}", format_value(&value, interp.obase));
            }
            Flow::Ok
        }
        Err(EvalError::DivisionByZero) => {
            eprintln!("bc: division by zero");
            Flow::Error
        }
        Err(EvalError::BaseRange(name)) => {
            eprintln!("bc: {name} must be between 2 and 16");
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

struct Interp {
    vars: HashMap<String, Value>,
    scale: u32,
    ibase: u32,
    obase: u32,
}

impl Default for Interp {
    fn default() -> Self {
        Self {
            vars: HashMap::new(),
            scale: 0,
            ibase: 10,
            obase: 10,
        }
    }
}

/// bc prints a non-zero number with exactly its own `scale` fractional
/// digits (a leading zero before the point, unlike GNU bc's `0.33333` vs
/// `.33333`); zero is always plain `0`.  `obase` (2-16) selects the radix;
/// digits 10-15 print as uppercase letters like GNU bc.
fn format_value(value: &Value, obase: u32) -> String {
    if value.digits == 0 {
        return "0".to_string();
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
    out.push_str(&integer_in_base(&magnitude[..split], obase));
    if scale > 0 {
        out.push('.');
        out.push_str(&fraction_in_base(&magnitude[split..], obase));
    }
    out
}

/// One base-`obase` digit; 10-15 print as uppercase letters like GNU bc.
fn base_digit(digit: u32) -> char {
    if digit < 10 {
        char::from(b'0' + digit as u8)
    } else {
        char::from(b'A' + (digit - 10) as u8)
    }
}

/// Integer part in `obase` by repeated division; base 10 passes through
/// verbatim.  Parsed as u128 so even the `i128::MIN` magnitude fits.
fn integer_in_base(decimal: &str, obase: u32) -> String {
    if obase == 10 {
        return decimal.to_string();
    }
    let mut n: u128 = decimal.parse().unwrap_or(0);
    if n == 0 {
        return "0".to_string();
    }
    let mut out = Vec::new();
    while n > 0 {
        out.push(base_digit((n % u128::from(obase)) as u32));
        n /= u128::from(obase);
    }
    out.iter().rev().collect()
}

/// Fraction part in `obase` with exactly the value's scale digits, by
/// repeated multiplication of the decimal numerator (`0.5000` renders
/// `.8000` in base 16); base 10 passes through.
fn fraction_in_base(decimal: &str, obase: u32) -> String {
    if obase == 10 {
        return decimal.to_string();
    }
    let mut digits: Vec<u32> = decimal.bytes().map(|b| u32::from(b - b'0')).collect();
    let mut out = String::new();
    for _ in 0..digits.len() {
        let mut carry = 0u32;
        for d in digits.iter_mut().rev() {
            let v = *d * obase + carry;
            *d = v % 10;
            carry = v / 10;
        }
        out.push(base_digit(carry));
    }
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

/// Keywords whose GNU bc behavior is not implemented in this build.  `ibase`
/// and `obase` are implemented as special variables, not keywords.
const RESERVED: &[&str] = &[
    "define", "if", "while", "for", "return", "auto", "break", "halt", "print", "read", "limits",
];

fn lex(line: &str, ibase: u32) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\r' => {
                chars.next();
            }
            // Comment: skip the rest of the line.
            '#' => break,
            '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' | '.' => {
                // Digits and '.' start a number; letters join a number as
                // digits only when ibase > 10 (`ibase=16; FF` reads 255).  A
                // digit invalid in the current base is a lex error.
                if c.is_ascii_digit() && digit_value(c, ibase).is_none() {
                    return Err(LexError::Syntax);
                }
                let may_start_number = c == '.'
                    || c.is_ascii_digit()
                    || (ibase > 10 && digit_value(c, ibase).is_some());
                if may_start_number {
                    tokens.push(Token::Num(lex_number(&mut chars, ibase)?));
                } else {
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

/// Value of one number digit in `ibase`; letters become digits 10-15 (either
/// case, like GNU bc) once the base allows them, everything else is None.
fn digit_value(c: char, ibase: u32) -> Option<i128> {
    let value = match c {
        '0'..='9' => i128::from(c as u8 - b'0'),
        'a'..='z' => 10 + i128::from(c as u8 - b'a'),
        'A'..='Z' => 10 + i128::from(c as u8 - b'A'),
        _ => return None,
    };
    if value < i128::from(ibase) {
        Some(value)
    } else {
        None
    }
}

/// Decimal places needed to render one `ibase` fraction digit exactly; only
/// bases built from 2s and 5s terminate, so the rest spend six places per
/// input digit and truncate.
fn decimal_frac_factor(ibase: u32) -> u32 {
    match ibase {
        2 | 5 | 10 => 1,
        4 => 2,
        8 => 3,
        16 => 4,
        _ => 6,
    }
}

/// Lexes one number in `ibase` and converts it to decimal fixed-point: an
/// optional integer part, then an optional fraction.  Base-10 numbers keep
/// their typed digits verbatim (scale = fraction length); other bases are
/// converted with trailing zeros trimmed, so hex `0.8` becomes 0.5.
fn lex_number(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    ibase: u32,
) -> Result<Value, LexError> {
    let mut int_val: i128 = 0;
    while let Some(&d) = chars.peek() {
        match digit_value(d, ibase) {
            Some(v) => {
                int_val = int_val
                    .checked_mul(i128::from(ibase))
                    .and_then(|acc| acc.checked_add(v))
                    .ok_or(LexError::NumberTooLarge)?;
                chars.next();
            }
            None => break,
        }
    }
    let mut frac_num: i128 = 0;
    let mut frac_len: u32 = 0;
    if chars.peek() == Some(&'.') {
        chars.next();
        while let Some(&d) = chars.peek() {
            match digit_value(d, ibase) {
                Some(v) => {
                    frac_num = frac_num
                        .checked_mul(i128::from(ibase))
                        .and_then(|acc| acc.checked_add(v))
                        .ok_or(LexError::NumberTooLarge)?;
                    frac_len += 1;
                    chars.next();
                }
                None => break,
            }
        }
        if frac_len == 0 {
            return Err(LexError::Syntax);
        }
    }
    if frac_len == 0 {
        return Ok(Value {
            digits: int_val,
            scale: 0,
        });
    }
    // Render the fraction `frac_num / ibase^frac_len` in decimal; the long
    // division in `divide` keeps intermediates small where a direct
    // `frac_num * 10^scale` product would overflow i128.
    let scale = frac_len * decimal_frac_factor(ibase);
    let mut denom: i128 = 1;
    for _ in 0..frac_len {
        denom = denom
            .checked_mul(i128::from(ibase))
            .ok_or(LexError::NumberTooLarge)?;
    }
    let frac = match divide(
        Value {
            digits: frac_num,
            scale: 0,
        },
        Value {
            digits: denom,
            scale: 0,
        },
        scale,
    ) {
        Ok(value) => value.digits,
        Err(_) => return Err(LexError::NumberTooLarge),
    };
    let shifted = mul_pow10(int_val, scale).map_err(|_| LexError::NumberTooLarge)?;
    let mut digits = shifted.checked_add(frac).ok_or(LexError::NumberTooLarge)?;
    let mut scale = scale;
    if ibase != 10 {
        while digits != 0 && scale > 0 && digits % 10 == 0 {
            digits /= 10;
            scale -= 1;
        }
    }
    Ok(Value { digits, scale })
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
    /// A base assignment outside 2-16; the payload names the variable.
    BaseRange(&'static str),
    NegativeSquareRoot,
}

fn eval(expr: &Expr, interp: &mut Interp) -> Result<Value, EvalError> {
    match expr {
        Expr::Num(value) => Ok(*value),
        // Uninitialized variables are 0, like bc; the special variables
        // `scale`, `ibase` and `obase` read the globals.
        Expr::Var(name) => {
            if name == "scale" {
                Ok(Value {
                    digits: interp.scale as i128,
                    scale: 0,
                })
            } else if name == "ibase" {
                Ok(Value {
                    digits: i128::from(interp.ibase),
                    scale: 0,
                })
            } else if name == "obase" {
                Ok(Value {
                    digits: i128::from(interp.obase),
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
            } else if name == "ibase" || name == "obase" {
                let n = integer_part(value);
                if !(2..=16).contains(&n) {
                    return Err(EvalError::BaseRange(if name == "ibase" {
                        "ibase"
                    } else {
                        "obase"
                    }));
                }
                if name == "ibase" {
                    interp.ibase = n as u32;
                } else {
                    interp.obase = n as u32;
                }
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
