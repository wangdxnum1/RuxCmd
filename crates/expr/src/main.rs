use clap::Parser;
use regex::Regex;

/// Evaluate expressions - a Windows port of GNU expr (integer arithmetic,
/// comparisons, | / &, anchored BRE matching, and the string functions).
#[derive(Parser, Debug)]
#[command(name = "expr", version, about, disable_version_flag = false)]
pub struct Args {
    /// Expression tokens; parentheses must be quoted by the caller
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub tokens: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let mut tokens = args.tokens;
    if tokens.is_empty() {
        eprintln!("expr: missing operand");
        std::process::exit(2);
    }
    tokens.reverse();

    let mut parser = ParserState { tokens };
    let result = match parser.parse_or() {
        Ok(value) => {
            if !parser.tokens.is_empty() {
                let unexpected = parser.tokens.last().cloned().unwrap_or_default();
                eprintln!("expr: syntax error: unexpected argument '{}'", unexpected);
                std::process::exit(2);
            }
            value
        }
        Err(e) => {
            eprintln!("expr: {}", e);
            std::process::exit(2);
        }
    };

    println!("{}", result);
    if result.is_empty() || result == "0" {
        std::process::exit(1);
    }
}

struct ParserState {
    tokens: Vec<String>,
}

impl ParserState {
    fn next(&mut self) -> Option<String> {
        self.tokens.pop()
    }

    fn peek(&self) -> Option<&String> {
        self.tokens.last()
    }

    // Precedence (low to high): | & comparisons additive multiplicative :

    fn parse_or(&mut self) -> Result<String, String> {
        let mut left = self.parse_and()?;
        while self.peek().map(|t| t == "|").unwrap_or(false) {
            self.next();
            let right = self.parse_and()?;
            left = if is_null_or_zero(&left) {
                if is_null_or_zero(&right) {
                    "0".to_string()
                } else {
                    right
                }
            } else {
                left
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<String, String> {
        let mut left = self.parse_comparison()?;
        while self.peek().map(|t| t == "&").unwrap_or(false) {
            self.next();
            let right = self.parse_comparison()?;
            // GNU: the left operand is returned when both are non-null and
            // non-zero; otherwise "0".
            left = if is_null_or_zero(&left) || is_null_or_zero(&right) {
                "0".to_string()
            } else {
                left
            };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<String, String> {
        let mut left = self.parse_additive()?;
        while let Some(op) = self.peek() {
            let op = op.clone();
            if !matches!(op.as_str(), "=" | "!=" | "<" | "<=" | ">=" | ">") {
                break;
            }
            self.next();
            let right = self.parse_additive()?;
            left = compare(&left, &right, &op);
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<String, String> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                Some(t) if t == "+" || t == "-" => t.clone(),
                _ => break,
            };
            self.next();
            let right = self.parse_multiplicative()?;
            let a = to_int(&left)?;
            let b = to_int(&right)?;
            left = (match op.as_str() {
                "+" => a.checked_add(b),
                "-" => a.checked_sub(b),
                _ => unreachable!(),
            })
            .map(|v| v.to_string())
            .ok_or_else(|| "numeric operation overflow".to_string())?;
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<String, String> {
        let mut left = self.parse_match()?;
        loop {
            let op = match self.peek() {
                Some(t) if t == "*" || t == "/" || t == "%" => t.clone(),
                _ => break,
            };
            self.next();
            let right = self.parse_match()?;
            let a = to_int(&left)?;
            let b = to_int(&right)?;
            left = match op.as_str() {
                "*" => a
                    .checked_mul(b)
                    .map(|v| v.to_string())
                    .ok_or_else(|| "numeric operation overflow".to_string())?,
                "/" | "%" => {
                    if b == 0 {
                        return Err("division by zero".to_string());
                    }
                    match op.as_str() {
                        "/" => a.wrapping_div(b),
                        _ => a.wrapping_rem(b),
                    }
                    .to_string()
                }
                _ => unreachable!(),
            };
        }
        Ok(left)
    }

    fn parse_match(&mut self) -> Result<String, String> {
        let mut left = self.parse_atom()?;
        loop {
            if !self.peek().map(|t| t == ":").unwrap_or(false) {
                break;
            }
            self.next();
            let pattern = self.parse_atom()?;
            left = anchored_match(&left, &pattern)?;
        }
        Ok(left)
    }

    fn parse_atom(&mut self) -> Result<String, String> {
        let token = self
            .next()
            .ok_or_else(|| "syntax error: unexpected end of expression".to_string())?;

        match token.as_str() {
            "(" => {
                let value = self.parse_or()?;
                match self.next() {
                    Some(t) if t == ")" => Ok(value),
                    _ => Err("syntax error: expected ')'".to_string()),
                }
            }
            "length" => {
                let value = self.parse_atom()?;
                Ok(value.chars().count().to_string())
            }
            "match" => {
                let subject = self.parse_atom()?;
                let pattern = self.parse_atom()?;
                anchored_match(&subject, &pattern)
            }
            "index" => {
                let subject = self.parse_atom()?;
                let chars = self.parse_atom()?;
                let found = subject
                    .chars()
                    .position(|c| chars.contains(c))
                    .map(|i| i + 1)
                    .unwrap_or(0);
                Ok(found.to_string())
            }
            "substr" => {
                let subject = self.parse_atom()?;
                let pos_raw = self.parse_atom()?;
                let len_raw = self.parse_atom()?;
                let pos = to_int(&pos_raw)?;
                let len = to_int(&len_raw)?;
                if pos < 1 || len < 0 {
                    return Ok(String::new());
                }
                Ok(subject
                    .chars()
                    .skip((pos - 1) as usize)
                    .take(len as usize)
                    .collect())
            }
            other => Ok(other.to_string()),
        }
    }
}

fn is_null_or_zero(value: &str) -> bool {
    value.is_empty() || value == "0"
}

fn to_int(value: &str) -> Result<i64, String> {
    value
        .trim()
        .parse::<i64>()
        .map_err(|_| "non-integer argument".to_string())
}

fn compare(left: &str, right: &str, op: &str) -> String {
    let ordering = match (left.trim().parse::<i64>(), right.trim().parse::<i64>()) {
        (Ok(a), Ok(b)) => a.cmp(&b),
        _ => left.cmp(right),
    };
    let truth = match op {
        "=" => ordering.is_eq(),
        "!=" => ordering.is_ne(),
        "<" => ordering.is_lt(),
        "<=" => ordering.is_le(),
        ">=" => ordering.is_ge(),
        ">" => ordering.is_gt(),
        _ => false,
    };
    if truth {
        "1".to_string()
    } else {
        "0".to_string()
    }
}

/// GNU anchored match: `EXPR : BRE` and `match EXPR BRE` return the first
/// group when the pattern has one, otherwise the match length in characters.
fn anchored_match(subject: &str, pattern: &str) -> Result<String, String> {
    // Core translates the BRE; expr anchors it at the head like GNU.
    let rust_pattern =
        ruxcmd_core::bre::to_regex(pattern).map_err(|e| format!("syntax error: {}", e))?;
    let re = Regex::new(&format!("^{}", rust_pattern))
        .map_err(|_| "syntax error: invalid regular expression".to_string())?;
    match re.captures(subject) {
        None => Ok(String::new()),
        Some(captures) => match captures.get(1) {
            Some(group) => Ok(group.as_str().to_string()),
            None => Ok(captures
                .get(0)
                .unwrap()
                .as_str()
                .chars()
                .count()
                .to_string()),
        },
    }
}
