//! Command-line parsing for `find`.
//!
//! GNU `find` takes operands first and an expression afterwards, and `-exec`
//! swallows arguments up to a literal `;` or `+`. clap cannot express that
//! without losing where the expression resumes, so this subset is parsed by
//! hand. Both GNU single-dash long options (`-name`) and double-dash spellings
//! (`--name`) are accepted.
//!
//! The expression is parsed into a tree with GNU's precedence: `!` binds
//! tightest, then implicit/explicit `-a`, then `-o`. `\( \)` groups and the
//! operators are shell tokens (`bash find \( -name x \)`), so they arrive as
//! ordinary `(`, `)` and `!` arguments. Actions stay leaves inside the tree,
//! which keeps command-line order (`-print -delete`) and short-circuiting.
//! Two GNU quirks are reproduced: a stray `)` with no expression before it is
//! treated as a start point, and a missing action appends an implicit
//! trailing `-print` around the whole expression.

use regex::Regex;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
    /// Unique key for the `+` batching buffer; one buffer per `-exec` action.
    pub id: usize,
}

/// Which stamp `-newer` / `-newermt` / `-newerat` compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampKind {
    Modified,
    Accessed,
}

/// One leaf of the expression: a test or an action.
#[derive(Debug, Clone)]
pub enum Node {
    Name(String),
    Regex(Regex),
    Type(char),
    Size(SizeSpec),
    Mtime(AgeSpec),
    /// `-newer FILE` (reference mtime resolved at parse time) and
    /// `-newermt` / `-newerat DATE` with the instant already parsed.
    Newer {
        instant: SystemTime,
        kind: StampKind,
    },
    Print,
    Print0,
    Delete,
    Exec(ExecAction),
}

/// The parsed expression tree. Evaluation is left to right with GNU's
/// short-circuit rules: `-a` skips the right side when the left fails and
/// `-o` skips it when the left succeeds.
#[derive(Debug, Clone)]
pub enum Expr {
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Primary(Node),
}

impl Expr {
    /// Any action in the tree disables the implicit trailing `-print`.
    pub fn has_action(&self) -> bool {
        match self {
            Expr::Primary(node) => {
                matches!(
                    node,
                    Node::Print | Node::Print0 | Node::Delete | Node::Exec(_)
                )
            }
            Expr::Not(inner) => inner.has_action(),
            Expr::And(left, right) | Expr::Or(left, right) => {
                left.has_action() || right.has_action()
            }
        }
    }
}

#[derive(Debug)]
pub struct Args {
    pub paths: Vec<PathBuf>,
    pub mindepth: Option<usize>,
    pub maxdepth: Option<usize>,
    pub basename: bool,
    pub ignore_case: bool,
    /// `-delete` was given, which also implies depth-first traversal.
    pub delete: bool,
    /// Number of `-exec` actions in the expression (`+` batching buffers).
    pub exec_count: usize,
    /// Expression tree, including GNU's implicit trailing `-print`.
    pub expr: Expr,
}

/// One classified token of the expression grammar.
#[derive(Debug)]
enum Tok {
    Primary(Node),
    Open,
    Close,
    /// `!` or `-not`, with the spelling used on the command line for errors.
    Not(String),
    /// `-a`/`-and` or `-o`/`-or`, with the spelling for error messages.
    Binary {
        is_or: bool,
        spelling: String,
    },
}

/// Parses the arguments after the program name.
pub fn parse(argv: &[String]) -> Result<Args, String> {
    let mut scan = Scan {
        argv,
        index: 0,
        paths: Vec::new(),
        toks: Vec::new(),
        depth: 0,
        started: false,
        mindepth: None,
        maxdepth: None,
        basename: false,
        ignore_case: false,
        delete: false,
        exec_count: 0,
    };
    scan.tokens()?;
    if scan.depth > 0 {
        return Err(
            "invalid expression; I was expecting to find a ')' somewhere but did not see one."
                .to_string(),
        );
    }

    let mut parser = TreeParser {
        toks: scan.toks,
        pos: 0,
    };
    let mut expr = parser.expression()?;
    // GNU appends an implicit -print when the expression has no action; it
    // applies to the whole expression, so it is AND-ed at the root.
    if !expr.has_action() {
        expr = Expr::And(Box::new(expr), Box::new(Expr::Primary(Node::Print)));
    }

    Ok(Args {
        paths: scan.paths,
        mindepth: scan.mindepth,
        maxdepth: scan.maxdepth,
        basename: scan.basename,
        ignore_case: scan.ignore_case,
        delete: scan.delete,
        exec_count: scan.exec_count,
        expr,
    })
}

/// First pass: splits argv into start points, global options and expression
/// tokens. Primary arguments (including `-exec ... ;`) are consumed here, so
/// values that look like operators never confuse the grammar.
struct Scan<'a> {
    argv: &'a [String],
    index: usize,
    paths: Vec<PathBuf>,
    toks: Vec<Tok>,
    /// Current `(` nesting depth, for GNU's unbalanced-group handling.
    depth: usize,
    /// Whether a primary has been seen; a `)` before any primary is a start
    /// point instead of a syntax error.
    started: bool,
    mindepth: Option<usize>,
    maxdepth: Option<usize>,
    basename: bool,
    ignore_case: bool,
    delete: bool,
    exec_count: usize,
}

impl Scan<'_> {
    fn tokens(&mut self) -> Result<(), String> {
        while self.index < self.argv.len() {
            let token = self.argv[self.index].clone();
            self.index += 1;
            match token.as_str() {
                // The grouping and negation tokens never start with a dash,
                // so they are classified before the operand check below.
                "(" => {
                    self.depth += 1;
                    self.toks.push(Tok::Open);
                }
                ")" => {
                    // GNU: a ')' before any expression element is just a
                    // start point (it later fails to stat it), while one
                    // after an expression is reported as "too many ')'".
                    if self.depth == 0 && !self.started {
                        self.paths.push(PathBuf::from(token));
                    } else {
                        if self.depth > 0 {
                            self.depth -= 1;
                        }
                        self.toks.push(Tok::Close);
                    }
                }
                "!" => self.toks.push(Tok::Not(token)),
                _ => {
                    // GNU accepts operands anywhere on the command line, so
                    // anything that is not option-shaped is a start point.
                    if !token.starts_with('-') || token == "-" {
                        self.paths.push(PathBuf::from(token));
                        continue;
                    }
                    self.option(token)?;
                }
            }
        }
        Ok(())
    }

    /// Classifies one dash-prefixed token: expression operator, global option
    /// or primary.
    fn option(&mut self, token: String) -> Result<(), String> {
        match token.as_str() {
            "-not" => self.toks.push(Tok::Not(token)),
            "-a" | "-and" => self.toks.push(Tok::Binary {
                is_or: false,
                spelling: token,
            }),
            "-o" | "-or" => self.toks.push(Tok::Binary {
                is_or: true,
                spelling: token,
            }),
            "-name" | "--name" => {
                let value = self.take_value("-name")?;
                self.primary(Node::Name(value));
            }
            "-regex" | "--regex" => {
                let pattern = self.take_value("-regex")?;
                let re = Regex::new(&pattern)
                    .map_err(|e| format!("invalid regex '{}': {}", pattern, e))?;
                self.primary(Node::Regex(re));
            }
            "-type" | "--type" => {
                let value = self.take_value("-type")?;
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    (Some(c @ ('d' | 'f' | 'l')), None) => self.primary(Node::Type(c)),
                    _ => return Err(format!("invalid argument '{}' to '-type'", value)),
                }
            }
            "-maxdepth" | "--maxdepth" => {
                self.maxdepth = Some(self.take_depth("-maxdepth")?);
            }
            "-mindepth" | "--mindepth" => {
                self.mindepth = Some(self.take_depth("-mindepth")?);
            }
            "-size" | "--size" => {
                let value = self.take_value("-size")?;
                self.primary(Node::Size(parse_size(&value)?));
            }
            "-mtime" | "--mtime" => {
                let value = self.take_value("-mtime")?;
                self.primary(Node::Mtime(parse_age(&value)?));
            }
            "-newer" | "--newer" => {
                let value = self.take_value("-newer")?;
                let instant = reference_mtime(&value)?;
                self.primary(Node::Newer {
                    instant,
                    kind: StampKind::Modified,
                });
            }
            "-newermt" | "--newermt" => self.newer_date(&token, StampKind::Modified)?,
            "-newerat" | "--newerat" => self.newer_date(&token, StampKind::Accessed)?,
            "-print" | "--print" => self.primary(Node::Print),
            "-print0" | "--print0" => self.primary(Node::Print0),
            "-delete" | "--delete" => {
                self.primary(Node::Delete);
                self.delete = true;
            }
            "-exec" | "--exec" => {
                let action = self.take_exec()?;
                self.primary(Node::Exec(action));
            }
            "-basename" | "--basename" | "-f" => self.basename = true,
            "-ignore-case" | "--ignore-case" | "-i" => self.ignore_case = true,
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
        Ok(())
    }

    fn primary(&mut self, node: Node) {
        if matches!(node, Node::Exec(_)) {
            self.exec_count += 1;
        }
        self.started = true;
        self.toks.push(Tok::Primary(node));
    }

    fn take_value(&mut self, option: &str) -> Result<String, String> {
        if self.index < self.argv.len() {
            let value = self.argv[self.index].clone();
            self.index += 1;
            Ok(value)
        } else {
            Err(format!("missing argument to '{}'", option))
        }
    }

    fn take_depth(&mut self, option: &str) -> Result<usize, String> {
        let value = self.take_value(option)?;
        value
            .parse()
            .map_err(|_| format!("invalid argument '{}' to '{}'", value, option))
    }

    /// `-newermt DATE` / `-newerat DATE`. GNU words the missing argument
    /// differently from `-newer`.
    fn newer_date(&mut self, token: &str, kind: StampKind) -> Result<(), String> {
        if self.index >= self.argv.len() {
            return Err(format!("The '{}' test needs an argument", token));
        }
        let value = self.argv[self.index].clone();
        self.index += 1;
        let instant = parse_date(&value).ok_or_else(|| {
            format!("I cannot figure out how to interpret '{value}' as a date or time")
        })?;
        self.primary(Node::Newer { instant, kind });
        Ok(())
    }

    fn take_exec(&mut self) -> Result<ExecAction, String> {
        let mut words = Vec::new();
        let mut batch = false;
        let mut terminated = false;
        while self.index < self.argv.len() {
            let token = self.argv[self.index].clone();
            self.index += 1;
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
            id: self.exec_count,
        })
    }
}

/// `-newer FILE` resolves the reference file's mtime at parse time, exactly
/// like GNU, which fails before walking when the file is missing.
fn reference_mtime(path: &str) -> Result<SystemTime, String> {
    let metadata =
        std::fs::metadata(path).map_err(|error| format!("'{}': {}", path, io_message(&error)))?;
    metadata
        .modified()
        .map_err(|error| format!("'{}': {}", path, io_message(&error)))
}

/// GNU prints the strerror text; std's Windows messages differ, so the common
/// not-found case is translated and anything else is shown verbatim.
pub fn io_message(error: &std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::NotFound => "No such file or directory".to_string(),
        _ => error.to_string(),
    }
}

/// Second pass: recursive descent over the classified tokens.
struct TreeParser {
    toks: Vec<Tok>,
    pos: usize,
}

impl TreeParser {
    fn expression(&mut self) -> Result<Expr, String> {
        if self.toks.is_empty() {
            // No expression at all: everything matches, and the caller's
            // implicit -print does the printing.
            return Ok(Expr::Primary(Node::Print));
        }
        let expr = self.or_expr(None)?;
        // Only a stray ')' can remain here; every other token is consumed by
        // the grammar or reported as an error while parsing.
        if matches!(self.toks.get(self.pos), Some(Tok::Close)) {
            return Err("you have too many ')'".to_string());
        }
        Ok(expr)
    }

    /// `-o` chains, lowest precedence. `required_by` names the operator whose
    /// operand is being parsed, for GNU's error wording.
    fn or_expr(&mut self, required_by: Option<&str>) -> Result<Expr, String> {
        let mut left = self.and_expr(required_by)?;
        while let Some(Tok::Binary {
            is_or: true,
            spelling,
        }) = self.toks.get(self.pos)
        {
            let spelling = spelling.clone();
            self.pos += 1;
            let right = self.and_expr(Some(&spelling))?;
            left = Expr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// `-a` chains, both the explicit operator and GNU's implicit and between
    /// two adjacent primaries.
    fn and_expr(&mut self, required_by: Option<&str>) -> Result<Expr, String> {
        let mut left = self.not_expr(required_by)?;
        loop {
            if let Some(Tok::Binary {
                is_or: false,
                spelling,
            }) = self.toks.get(self.pos)
            {
                let spelling = spelling.clone();
                self.pos += 1;
                let right = self.not_expr(Some(&spelling))?;
                left = Expr::And(Box::new(left), Box::new(right));
            } else if self.starts_operand() {
                // Implicit -and between adjacent tests/actions.
                let right = self.not_expr(None)?;
                left = Expr::And(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// `!` / `-not` prefixes.
    fn not_expr(&mut self, required_by: Option<&str>) -> Result<Expr, String> {
        if let Some(Tok::Not(spelling)) = self.toks.get(self.pos) {
            let spelling = spelling.clone();
            self.pos += 1;
            let inner = self.not_expr(Some(&spelling))?;
            return Ok(Expr::Not(Box::new(inner)));
        }
        self.operand(required_by)
    }

    fn starts_operand(&self) -> bool {
        matches!(
            self.toks.get(self.pos),
            Some(Tok::Primary(_)) | Some(Tok::Open) | Some(Tok::Not(_))
        )
    }

    fn operand(&mut self, required_by: Option<&str>) -> Result<Expr, String> {
        match self.toks.get(self.pos) {
            Some(Tok::Primary(node)) => {
                let node = node.clone();
                self.pos += 1;
                Ok(Expr::Primary(node))
            }
            Some(Tok::Open) => {
                self.pos += 1;
                if matches!(self.toks.get(self.pos), Some(Tok::Close)) {
                    return Err(
                        "invalid expression; empty parentheses are not allowed.".to_string()
                    );
                }
                let inner = self.or_expr(None)?;
                if !matches!(self.toks.get(self.pos), Some(Tok::Close)) {
                    return Err(
                        "invalid expression; I was expecting to find a ')' somewhere but did not see one."
                            .to_string(),
                    );
                }
                self.pos += 1;
                Ok(inner)
            }
            Some(Tok::Binary { spelling, .. }) => Err(format!(
                "invalid expression; you have used a binary operator '{}' with nothing before it.",
                spelling
            )),
            Some(Tok::Close) => match required_by {
                // `! )` and `-o )` both name the operator that was waiting
                // for an expression.
                Some(operator) => Err(format!(
                    "expected an expression between '{operator}' and ')'"
                )),
                None => Err("invalid expression; empty parentheses are not allowed.".to_string()),
            },
            None => match required_by {
                Some(operator) => Err(format!("expected an expression after '{operator}'")),
                // Only reachable for an empty token list, handled earlier.
                None => Ok(Expr::Primary(Node::Print)),
            },
            Some(Tok::Not(_)) => unreachable!("not_expr consumes '!' before calling operand"),
        }
    }
}

/// `touch -d` subset: `@SECONDS`, `YYYY-MM-DD`, `YYYY-MM-DDTHH:MM[:SS]` and
/// `YYYY-MM-DD HH:MM[:SS]`, all interpreted as local time. An empty string
/// means today at 00:00, as in GNU. Relative forms ("1 day ago") are not
/// supported.
fn parse_date(raw: &str) -> Option<SystemTime> {
    if raw.is_empty() {
        return today_midnight();
    }
    if let Some(seconds) = raw.strip_prefix('@') {
        let seconds: i64 = seconds.parse().ok()?;
        return epoch_to_system_time(seconds);
    }
    let (date_part, time_part) = match raw.find(['T', ' ']) {
        Some(index) => (&raw[..index], Some(&raw[index + 1..])),
        None => (raw, None),
    };
    let (year, month, day) = parse_ymd(date_part)?;
    let (hour, minute, second) = match time_part {
        None => (0, 0, 0),
        Some(time) => parse_hms(time)?,
    };
    if month == 0 || month > 12 || day == 0 || day > days_in_month(year, month) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    wall_to_system_time(WallClock {
        year,
        month,
        day,
        hour,
        minute,
        second,
    })
}

#[derive(Debug, Clone, Copy)]
struct WallClock {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

/// `YYYY-MM-DD` (strict 4-2-2 digits), as in the touch implementation.
fn parse_ymd(raw: &str) -> Option<(i32, u32, u32)> {
    let parts: Vec<&str> = raw.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return None;
    }
    for part in &parts {
        if !part.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

/// `HH:MM` or `HH:MM:SS` (strict 2-2[:2] digits).
fn parse_hms(raw: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = raw.split(':').collect();
    let (hour, minute) = (parts.first()?, parts.get(1)?);
    let seconds = match parts.len() {
        2 => 0,
        3 => {
            if parts[2].len() != 2 || !parts[2].bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            parts[2].parse().ok()?
        }
        _ => return None,
    };
    if hour.len() != 2 || minute.len() != 2 {
        return None;
    }
    if !hour.bytes().all(|b| b.is_ascii_digit()) || !minute.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((hour.parse().ok()?, minute.parse().ok()?, seconds))
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        // February: leap years are divisible by 4, except centuries that are
        // not divisible by 400.
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 0,
    }
}

fn epoch_to_system_time(seconds: i64) -> Option<SystemTime> {
    if seconds >= 0 {
        UNIX_EPOCH.checked_add(Duration::from_secs(seconds as u64))
    } else {
        UNIX_EPOCH.checked_sub(Duration::from_secs(seconds.unsigned_abs()))
    }
}

/// Interprets the wall clock in the local timezone. Windows provides the
/// conversion through kernel32 (the same API the touch implementation uses
/// via the windows crate); other targets fall back to UTC.
#[cfg(windows)]
fn wall_to_system_time(wall: WallClock) -> Option<SystemTime> {
    let local = SystemTimeFields {
        year: wall.year as u16,
        month: wall.month as u16,
        day_of_week: 0,
        day: wall.day as u16,
        hour: wall.hour as u16,
        minute: wall.minute as u16,
        second: wall.second as u16,
        milliseconds: 0,
    };
    let mut utc = SystemTimeFields {
        year: 0,
        month: 0,
        day_of_week: 0,
        day: 0,
        hour: 0,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };
    let mut filetime = FileTimeFields { low: 0, high: 0 };
    // A null timezone parameter means "use the current timezone database",
    // which handles historical DST offsets.
    unsafe {
        if TzSpecificLocalTimeToSystemTime(std::ptr::null(), &local, &mut utc) == 0 {
            return None;
        }
        if SystemTimeToFileTime(&utc, &mut filetime) == 0 {
            return None;
        }
    }
    let intervals = ((filetime.high as i64) << 32) | filetime.low as i64;
    // FILETIME counts 100ns intervals since 1601-01-01.
    let unix_intervals = intervals.checked_sub(116_444_736_000_000_000)?;
    intervals_to_system_time(unix_intervals)
}

/// 100ns intervals since the Unix epoch, negative before 1970.
fn intervals_to_system_time(intervals: i64) -> Option<SystemTime> {
    if intervals >= 0 {
        UNIX_EPOCH.checked_add(Duration::from_nanos(intervals as u64 * 100))
    } else {
        UNIX_EPOCH.checked_sub(Duration::from_nanos(intervals.unsigned_abs() * 100))
    }
}

/// 100ns intervals since the Unix epoch for a SystemTime at or after 1970.
fn system_time_to_intervals(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        * 10_000_000
        + i64::from(
            time.duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
                / 100,
        )
}

/// GNU resolves an empty date string to today at 00:00 local time.
#[cfg(windows)]
fn today_midnight() -> Option<SystemTime> {
    let now = system_time_to_intervals(SystemTime::now()) + 116_444_736_000_000_000;
    let now_filetime = FileTimeFields {
        low: now as u32,
        high: (now >> 32) as u32,
    };
    let mut utc = SystemTimeFields {
        year: 0,
        month: 0,
        day_of_week: 0,
        day: 0,
        hour: 0,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };
    let mut local = SystemTimeFields {
        year: 0,
        month: 0,
        day_of_week: 0,
        day: 0,
        hour: 0,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };
    unsafe {
        if FileTimeToSystemTime(&now_filetime, &mut utc) == 0 {
            return None;
        }
        if SystemTimeToTzSpecificLocalTime(std::ptr::null(), &utc, &mut local) == 0 {
            return None;
        }
    }
    wall_to_system_time(WallClock {
        year: i32::from(local.year),
        month: u32::from(local.month),
        day: u32::from(local.day),
        hour: 0,
        minute: 0,
        second: 0,
    })
}

/// GNU resolves an empty date string to today at 00:00; the non-Windows
/// fallback approximates it with UTC midnight.
#[cfg(not(windows))]
fn today_midnight() -> Option<SystemTime> {
    let seconds = system_time_to_intervals(SystemTime::now()) / 10_000_000;
    let day_start = seconds.div_euclid(86_400) * 86_400;
    epoch_to_system_time(day_start)
}

#[cfg(not(windows))]
fn wall_to_system_time(wall: WallClock) -> Option<SystemTime> {
    // Without a timezone database, interpret the wall clock as UTC.
    let days = days_from_civil(wall.year, wall.month, wall.day);
    let seconds =
        days * 86_400 + wall.hour as i64 * 3600 + wall.minute as i64 * 60 + wall.second as i64;
    epoch_to_system_time(seconds)
}

/// Days since 1970-01-01 (Howard Hinnant's civil-from-days inverse), used
/// only by the non-Windows fallback.
#[cfg(not(windows))]
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year as i64 - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_of_year = (month as i64 + 9) % 12;
    let day_of_year = (153 * month_of_year + 2) / 5 + day as i64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(windows)]
#[repr(C)]
struct SystemTimeFields {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

#[cfg(windows)]
#[repr(C)]
struct FileTimeFields {
    low: u32,
    high: u32,
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn TzSpecificLocalTimeToSystemTime(
        timezone: *const core::ffi::c_void,
        local: *const SystemTimeFields,
        universal: *mut SystemTimeFields,
    ) -> i32;
    fn SystemTimeToTzSpecificLocalTime(
        timezone: *const core::ffi::c_void,
        universal: *const SystemTimeFields,
        local: *mut SystemTimeFields,
    ) -> i32;
    fn SystemTimeToFileTime(time: *const SystemTimeFields, filetime: *mut FileTimeFields) -> i32;
    fn FileTimeToSystemTime(filetime: *const FileTimeFields, time: *mut SystemTimeFields) -> i32;
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
    println!("  -size [+-]N[cwbkMG], -mtime [+-]N, -newer FILE,");
    println!("  -newermt DATE, -newerat DATE (@SECONDS or YYYY-MM-DD[THH:MM[:SS]]),");
    println!("  -mindepth N, -maxdepth N");
    println!("Operators: ! / -not, -a / -and, -o / -or, \\( EXPRESSION \\)");
    println!("Actions: -print, -print0, -delete, -exec COMMAND {{}} \\; | +");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(words: &[&str]) -> Args {
        let argv: Vec<String> = words.iter().map(|w| w.to_string()).collect();
        parse(&argv).unwrap_or_else(|error| panic!("parse {words:?} failed: {error}"))
    }

    fn parse_err(words: &[&str]) -> String {
        let argv: Vec<String> = words.iter().map(|w| w.to_string()).collect();
        parse(&argv).expect_err(&format!("parse {words:?} should fail"))
    }

    #[test]
    fn size_rounds_up_to_the_unit() {
        let exact = parse_size("5k").unwrap();
        assert!(exact.matches(5 * 1024));
        // Rounding up means one extra byte spills into a sixth unit, while a
        // byte short still occupies five.
        assert!(!exact.matches(5 * 1024 + 1));
        assert!(exact.matches(5 * 1024 - 1));
        assert!(!exact.matches(4 * 1024));
        assert_eq!(exact.bound, Bound::Equal);
    }

    #[test]
    fn size_units_and_bounds() {
        let default_blocks = parse_size("1").unwrap();
        assert_eq!(default_blocks.unit, 512);
        assert!(default_blocks.matches(1));
        assert!(default_blocks.matches(512));

        let less = parse_size("-1c").unwrap();
        assert!(less.matches(0));
        assert!(!less.matches(1));

        let greater = parse_size("+1k").unwrap();
        assert!(greater.matches(2 * 1024));
        assert!(!greater.matches(1024));
    }

    #[test]
    fn invalid_size_is_rejected() {
        assert!(parse_size("1x").is_err());
        assert!(parse_size("abc").is_err());
        assert!(parse_size("").is_err());
    }

    #[test]
    fn or_binds_looser_than_implicit_and() {
        // -name a -o -name b -print  ==  (a -o (b -a print))
        let args = parse_args(&["-name", "a", "-o", "-name", "b", "-print"]);
        match args.expr {
            Expr::Or(left, right) => {
                assert!(matches!(*left, Expr::Primary(Node::Name(ref name)) if name == "a"));
                assert!(matches!(*right, Expr::And(..)));
            }
            other => panic!("expected Or at the root, got {other:?}"),
        }
    }

    #[test]
    fn not_binds_tighter_than_and() {
        // ! -name x -print  ==  ((! name x) -a print)
        let args = parse_args(&["!", "-name", "x", "-print"]);
        match args.expr {
            Expr::And(left, right) => {
                assert!(matches!(*left, Expr::Not(_)));
                assert!(matches!(*right, Expr::Primary(Node::Print)));
            }
            other => panic!("expected And at the root, got {other:?}"),
        }
    }

    #[test]
    fn groups_override_precedence() {
        // ( a -o b ) -print  ==  ((a -o b) -a print)
        let args = parse_args(&["(", "-name", "a", "-o", "-name", "b", ")", "-print"]);
        match args.expr {
            Expr::And(left, right) => {
                assert!(matches!(*left, Expr::Or(..)));
                assert!(matches!(*right, Expr::Primary(Node::Print)));
            }
            other => panic!("expected And at the root, got {other:?}"),
        }
    }

    #[test]
    fn implicit_print_wraps_the_whole_expression() {
        let args = parse_args(&["-name", "a", "-o", "-name", "b"]);
        match args.expr {
            Expr::And(left, right) => {
                assert!(matches!(*left, Expr::Or(..)));
                assert!(matches!(*right, Expr::Primary(Node::Print)));
            }
            other => panic!("expected And at the root, got {other:?}"),
        }
        // An explicit action anywhere disables it.
        let args = parse_args(&["-name", "a", "-o", "-print0"]);
        assert!(matches!(args.expr, Expr::Or(..)));
    }

    #[test]
    fn empty_expression_prints_everything() {
        let args = parse_args(&["."]);
        assert_eq!(args.paths, [PathBuf::from(".")]);
        assert!(matches!(args.expr, Expr::Primary(Node::Print)));
    }

    #[test]
    fn stray_close_paren_before_any_expression_is_a_start_point() {
        let args = parse_args(&[".", ")", "-print"]);
        assert!(args.paths.contains(&PathBuf::from(")")));
    }

    #[test]
    fn expression_errors_match_gnu_wording() {
        assert_eq!(parse_err(&["!"]), "expected an expression after '!'");
        assert_eq!(parse_err(&["-not"]), "expected an expression after '-not'");
        assert_eq!(
            parse_err(&["-name", "x", "-a"]),
            "expected an expression after '-a'"
        );
        assert_eq!(
            parse_err(&["-name", "x", "-or"]),
            "expected an expression after '-or'"
        );
        assert_eq!(
            parse_err(&["-o", "-print"]),
            "invalid expression; you have used a binary operator '-o' with nothing before it."
        );
        assert_eq!(
            parse_err(&["!", "-a", "-print"]),
            "invalid expression; you have used a binary operator '-a' with nothing before it."
        );
        assert_eq!(
            parse_err(&["(", "-name", "x"]),
            "invalid expression; I was expecting to find a ')' somewhere but did not see one."
        );
        assert_eq!(
            parse_err(&["(", ")"]),
            "invalid expression; empty parentheses are not allowed."
        );
        assert_eq!(parse_err(&["-name", "x", ")"]), "you have too many ')'");
        assert_eq!(
            parse_err(&["(", "-name", "x", "-o", ")"]),
            "expected an expression between '-o' and ')'"
        );
        assert_eq!(
            parse_err(&["(", "!", ")"]),
            "expected an expression between '!' and ')'"
        );
    }

    #[test]
    fn newer_predicate_errors() {
        assert_eq!(parse_err(&["-newer"]), "missing argument to '-newer'");
        assert_eq!(
            parse_err(&["-newermt"]),
            "The '-newermt' test needs an argument"
        );
        assert_eq!(
            parse_err(&["-newerat"]),
            "The '-newerat' test needs an argument"
        );
        assert_eq!(
            parse_err(&["-newermt", "not-a-date"]),
            "I cannot figure out how to interpret 'not-a-date' as a date or time"
        );
    }

    #[test]
    fn newer_node_kinds() {
        // Newer tests are not actions, so the implicit -print wraps them.
        let args = parse_args(&["-newermt", "@0", "-newerat", "@1000"]);
        match args.expr {
            Expr::And(left, right) => {
                assert!(matches!(*right, Expr::Primary(Node::Print)));
                match *left {
                    Expr::And(modified, accessed) => {
                        assert!(matches!(
                            *modified,
                            Expr::Primary(Node::Newer {
                                kind: StampKind::Modified,
                                ..
                            })
                        ));
                        assert!(matches!(
                            *accessed,
                            Expr::Primary(Node::Newer {
                                kind: StampKind::Accessed,
                                ..
                            })
                        ));
                    }
                    other => panic!("expected And of the newer tests, got {other:?}"),
                }
            }
            other => panic!("expected And at the root, got {other:?}"),
        }
    }

    #[test]
    fn date_subset_parses_and_rejects() {
        assert_eq!(parse_date("@0"), Some(UNIX_EPOCH));
        assert_eq!(
            parse_date("@1577836800"),
            epoch_to_system_time(1_577_836_800)
        );
        // An empty date means today at 00:00 local time, like GNU.
        let today = parse_date("").unwrap();
        assert!(today <= SystemTime::now());
        assert!(today > parse_date("2000-01-01").unwrap());
        // Ordering holds regardless of the local timezone.
        let before = parse_date("2019-12-31T23:59:59").unwrap();
        let exact = parse_date("2020-01-01").unwrap();
        let after = parse_date("2020-01-01 00:00:01").unwrap();
        assert!(before < exact && exact < after);
        assert!(parse_date("1970-01-01T00:00:01").unwrap() > parse_date("1970-01-01").unwrap());

        for bad in [
            "not-a-date",
            "2020-13-01",
            "2020-01-32",
            "2020-02-30",
            "2020-01",
            "2020-01-01T25:00",
            "2020-01-01 12:60",
            "@abc",
            "@12.5",
            "2020-01-01Z",
        ] {
            assert_eq!(parse_date(bad), None, "{bad} should not parse");
        }
    }
}
