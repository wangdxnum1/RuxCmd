//! Command-line parsing for `find`.
//!
//! GNU `find` takes operands first and an expression afterwards, and `-exec`
//! swallows arguments up to a literal `;` or `+`. clap cannot express that
//! without losing where the expression resumes, so this subset is parsed by
//! hand. Both GNU single-dash long options (`-name`) and double-dash spellings
//! (`--name`) are accepted.

use regex::Regex;
use std::path::PathBuf;

/// Comparison operator shared by `-size` and `-mtime`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    Less,
    Equal,
    Greater,
}

impl Bound {
    pub fn matches(self, actual: i64, expected: i64) -> bool {
        match self {
            Bound::Less => actual < expected,
            Bound::Equal => actual == expected,
            Bound::Greater => actual > expected,
        }
    }
}

/// `-size N[cwbkMG]`: sizes are rounded up to the unit before comparing, and
/// the default unit is a 512-byte block, as in GNU.
#[derive(Debug, Clone)]
pub struct SizeSpec {
    pub bound: Bound,
    pub count: u64,
    pub unit: u64,
}

impl SizeSpec {
    pub fn matches(&self, len: u64) -> bool {
        let units = len.div_ceil(self.unit) as i64;
        self.bound.matches(units, self.count as i64)
    }
}

/// `-mtime N` with GNU's 24-hour periods (fractions are ignored).
#[derive(Debug, Clone)]
pub struct AgeSpec {
    pub bound: Bound,
    pub days: u64,
}

/// `-exec COMMAND ... ;` runs once per entry; the `+` form batches entries
/// into a single invocation, so `{}` must be the last argument.
#[derive(Debug, Clone)]
pub struct ExecAction {
    pub program: String,
    pub args: Vec<String>,
    pub batch: bool,
}

/// One element of the expression. GNU evaluates tests and actions strictly
/// left to right with implicit `-and` short-circuiting, so order is preserved
/// here instead of splitting the expression into a filter and an action list.
#[derive(Debug, Clone)]
pub enum Node {
    Name(String),
    Regex(Regex),
    Type(char),
    Size(SizeSpec),
    Mtime(AgeSpec),
    Print,
    Print0,
    Delete,
    Exec(ExecAction),
}

impl Node {
    /// Actions are evaluated even when a preceding test fails only by GNU's
    /// `-and` rule, but the presence of any action disables the implicit print.
    pub fn is_action(&self) -> bool {
        matches!(
            self,
            Node::Print | Node::Print0 | Node::Delete | Node::Exec(_)
        )
    }
}

#[derive(Debug, Default)]
pub struct Args {
    pub paths: Vec<PathBuf>,
    pub mindepth: Option<usize>,
    pub maxdepth: Option<usize>,
    pub basename: bool,
    pub ignore_case: bool,
    /// `-delete` was given, which also implies depth-first traversal.
    pub delete: bool,
    /// Expression nodes in command-line order.
    pub nodes: Vec<Node>,
}

/// Parses the arguments after the program name.
pub fn parse(argv: &[String]) -> Result<Args, String> {
    let mut args = Args::default();
    let mut index = 0;

    while index < argv.len() {
        let token = argv[index].clone();
        index += 1;
        // GNU accepts operands anywhere on the command line, so anything that
        // is not option-shaped is a start point.
        if !token.starts_with('-') || token == "-" {
            args.paths.push(PathBuf::from(token));
            continue;
        }
        match token.as_str() {
            "-name" | "--name" => {
                args.nodes
                    .push(Node::Name(take_value(argv, &mut index, "-name")?));
            }
            "-regex" | "--regex" => {
                let pattern = take_value(argv, &mut index, "-regex")?;
                let re = Regex::new(&pattern)
                    .map_err(|e| format!("invalid regex '{}': {}", pattern, e))?;
                args.nodes.push(Node::Regex(re));
            }
            "-type" | "--type" => {
                let value = take_value(argv, &mut index, "-type")?;
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    (Some(c @ ('d' | 'f' | 'l')), None) => args.nodes.push(Node::Type(c)),
                    _ => return Err(format!("invalid argument '{}' to '-type'", value)),
                }
            }
            "-maxdepth" | "--maxdepth" => {
                args.maxdepth = Some(parse_depth(argv, &mut index, "-maxdepth")?);
            }
            "-mindepth" | "--mindepth" => {
                args.mindepth = Some(parse_depth(argv, &mut index, "-mindepth")?);
            }
            "-size" | "--size" => {
                let value = take_value(argv, &mut index, "-size")?;
                args.nodes.push(Node::Size(parse_size(&value)?));
            }
            "-mtime" | "--mtime" => {
                let value = take_value(argv, &mut index, "-mtime")?;
                args.nodes.push(Node::Mtime(parse_age(&value)?));
            }
            "-print" | "--print" => args.nodes.push(Node::Print),
            "-print0" | "--print0" => args.nodes.push(Node::Print0),
            "-delete" | "--delete" => {
                args.nodes.push(Node::Delete);
                args.delete = true;
            }
            "-exec" | "--exec" => args.nodes.push(Node::Exec(parse_exec(argv, &mut index)?)),
            "-basename" | "--basename" | "-f" => args.basename = true,
            "-ignore-case" | "--ignore-case" | "-i" => args.ignore_case = true,
            "-v" | "--version" => {
                println!("find {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            other => return Err(format!("unknown predicate '{}'", other)),
        }
    }

    Ok(args)
}

fn take_value(argv: &[String], index: &mut usize, option: &str) -> Result<String, String> {
    if *index < argv.len() {
        let value = argv[*index].clone();
        *index += 1;
        Ok(value)
    } else {
        Err(format!("missing argument to '{}'", option))
    }
}

fn parse_depth(argv: &[String], index: &mut usize, option: &str) -> Result<usize, String> {
    let value = take_value(argv, index, option)?;
    value
        .parse()
        .map_err(|_| format!("invalid argument '{}' to '{}'", value, option))
}

fn parse_exec(argv: &[String], index: &mut usize) -> Result<ExecAction, String> {
    let mut words = Vec::new();
    let mut batch = false;
    let mut terminated = false;
    while *index < argv.len() {
        let token = argv[*index].clone();
        *index += 1;
        if token == ";" {
            terminated = true;
            break;
        }
        if token == "+" {
            batch = true;
            terminated = true;
            break;
        }
        words.push(token);
    }
    if !terminated {
        return Err("missing argument to '-exec'".to_string());
    }
    if words.is_empty() {
        return Err("missing command for '-exec'".to_string());
    }
    if batch {
        // GNU only substitutes all paths where `{}` stands alone as the last
        // argument, so anything else is rejected instead of guessed at.
        if words.last().map(String::as_str) != Some("{}") {
            return Err("'-exec ... {} +' requires '{}' as the last argument".to_string());
        }
        let placeholders = words.iter().filter(|w| w.contains("{}")).count();
        if placeholders != 1 {
            return Err("'-exec ... {} +' accepts exactly one '{}'".to_string());
        }
    }
    let program = words.remove(0);
    Ok(ExecAction {
        program,
        args: words,
        batch,
    })
}

pub(crate) fn parse_size(value: &str) -> Result<SizeSpec, String> {
    let (bound, rest) = split_bound(value);
    let (digits, unit) = match rest.chars().last() {
        Some(c) if c.is_ascii_alphabetic() => {
            let unit = match c {
                'b' => 512,
                'c' => 1,
                'w' => 2,
                'k' => 1024,
                'M' => 1024 * 1024,
                'G' => 1024 * 1024 * 1024,
                _ => return Err(format!("invalid -size type '{}'", c)),
            };
            (&rest[..rest.len() - 1], unit)
        }
        _ => (rest, 512),
    };
    let count = parse_number(digits, "-size")?;
    Ok(SizeSpec { bound, count, unit })
}

fn parse_age(value: &str) -> Result<AgeSpec, String> {
    let (bound, rest) = split_bound(value);
    let days = parse_number(rest, "-mtime")?;
    Ok(AgeSpec { bound, days })
}

fn split_bound(value: &str) -> (Bound, &str) {
    if let Some(rest) = value.strip_prefix('+') {
        (Bound::Greater, rest)
    } else if let Some(rest) = value.strip_prefix('-') {
        (Bound::Less, rest)
    } else {
        (Bound::Equal, value)
    }
}

fn parse_number(digits: &str, option: &str) -> Result<u64, String> {
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("invalid argument '{}' to '{}'", digits, option));
    }
    digits
        .parse()
        .map_err(|_| format!("invalid argument '{}' to '{}'", digits, option))
}

fn print_help() {
    println!("Usage: find [PATH...] [EXPRESSION]");
    println!("Search for files in a directory hierarchy.");
    println!();
    println!("Expression tests: -name PATTERN, -regex PATTERN, -type d|f|l,");
    println!("  -size [+-]N[cwbkMG], -mtime [+-]N, -mindepth N, -maxdepth N");
    println!("Actions: -print, -print0, -delete, -exec COMMAND {{}} \\; | +");
}
