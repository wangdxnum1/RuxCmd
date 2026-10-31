mod cli;

use regex::bytes::Regex;
use ruxcmd_core::bre;
use std::fs;
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// ---------- script model ----------

/// One address: a line number, `$` (last line), or a basic regular
/// expression between slashes (compiled at parse time).
#[derive(Debug, Clone)]
enum Address {
    Line(u64),
    LastLine,
    Regex(Regex),
}

impl Address {
    fn matches(&self, pattern_space: &[u8], line_num: u64, is_last: bool) -> bool {
        match self {
            Address::Line(n) => line_num == *n,
            Address::LastLine => is_last,
            Address::Regex(re) => re.is_match(pattern_space),
        }
    }
}

#[derive(Debug, Clone)]
enum CmdKind {
    Substitute {
        re: Regex,
        replacement: Vec<u8>,
        global: bool,
        print: bool,
        nth: u32,
    },
    Print,
    Delete,
    Quit,
}

#[derive(Debug, Clone)]
struct Command {
    addr1: Option<Address>,
    addr2: Option<Address>,
    kind: CmdKind,
}

// ---------- script parsing ----------

struct ExprParser<'a> {
    src: &'a [u8],
    pos: usize,
    expr_no: usize,
    extended: bool,
}

impl<'a> ExprParser<'a> {
    fn new(src: &'a str, expr_no: usize, extended: bool) -> Self {
        ExprParser {
            src: src.as_bytes(),
            pos: 0,
            expr_no,
            extended,
        }
    }

    fn error(&self, pos: usize, msg: &str) -> String {
        format!(
            "sed: -e expression #{}, char {}: {}",
            self.expr_no,
            pos + 1,
            msg
        )
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn skip_blanks(&mut self) {
        while matches!(self.peek(), Some(b' ') | Some(b'\t')) {
            self.pos += 1;
        }
    }

    fn skip_blanks_and_semicolons(&mut self) {
        while matches!(self.peek(), Some(b' ') | Some(b'\t') | Some(b';')) {
            self.pos += 1;
        }
    }

    /// Read a `/regex/` address. Only `\/` collapses to a literal slash; any
    /// other backslash sequence passes through to the BRE engine.
    fn read_address_regex(&mut self) -> Result<Regex, String> {
        let mut pattern = Vec::new();
        loop {
            match self.peek() {
                None => {
                    return Err(self.error(self.pos, "unterminated address regular expression"));
                }
                Some(b'\\') => {
                    self.pos += 1;
                    match self.peek() {
                        None => {
                            return Err(
                                self.error(self.pos, "unterminated address regular expression")
                            );
                        }
                        Some(b'/') => {
                            pattern.push(b'/');
                            self.pos += 1;
                        }
                        Some(escaped) => {
                            pattern.push(b'\\');
                            pattern.push(escaped);
                            self.pos += 1;
                        }
                    }
                }
                Some(b'/') => {
                    self.pos += 1;
                    break;
                }
                Some(byte) => {
                    pattern.push(byte);
                    self.pos += 1;
                }
            }
        }
        compile_bre(&pattern, self.expr_no, self.extended)
    }

    fn parse_address(&mut self) -> Result<Option<Address>, String> {
        match self.peek() {
            Some(b'$') => {
                self.pos += 1;
                Ok(Some(Address::LastLine))
            }
            Some(c) if c.is_ascii_digit() => {
                let start = self.pos;
                while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                    self.pos += 1;
                }
                let digits = std::str::from_utf8(&self.src[start..self.pos]).unwrap_or_default();
                Ok(Some(Address::Line(digits.parse().unwrap_or(u64::MAX))))
            }
            Some(b'/') => {
                self.pos += 1;
                Ok(Some(Address::Regex(self.read_address_regex()?)))
            }
            _ => Ok(None),
        }
    }

    fn parse_substitute(&mut self) -> Result<CmdKind, String> {
        // Entry position: the delimiter right after the `s`.
        let start = self.pos;
        let delimiter = match self.peek() {
            Some(c) if c != b'\\' && !c.is_ascii_whitespace() => c,
            _ => return Err(self.error(start, "unterminated `s' command")),
        };
        self.pos += 1;
        let pattern = self.read_delimited(delimiter)?;
        let replacement = self.read_delimited(delimiter)?;

        let mut global = false;
        let mut print = false;
        let mut case_insensitive = false;
        let mut nth: Option<u32> = None;
        loop {
            match self.peek() {
                Some(b'g') => {
                    global = true;
                    self.pos += 1;
                }
                Some(b'p') => {
                    print = true;
                    self.pos += 1;
                }
                Some(b'i' | b'I') => {
                    case_insensitive = true;
                    self.pos += 1;
                }
                Some(c) if c.is_ascii_digit() => {
                    let start = self.pos;
                    while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                        self.pos += 1;
                    }
                    let digits =
                        std::str::from_utf8(&self.src[start..self.pos]).unwrap_or_default();
                    nth = digits.parse().ok();
                    if nth.is_none() {
                        return Err(self.error(start, "invalid occurrence number"));
                    }
                }
                Some(c) if c.is_ascii_whitespace() || c == b';' => break,
                Some(_) => return Err(self.error(self.pos, "unknown option to `s'")),
                None => break,
            }
        }

        // Translate the BRE first; prepending (?i) before translation would
        // escape the inline flag into a literal.
        let translated = if self.extended {
            bre::to_regex_ere(&String::from_utf8_lossy(&pattern))
        } else {
            bre::to_regex(&String::from_utf8_lossy(&pattern))
        }
        .map_err(|e| format!("sed: -e expression #{}: {}", self.expr_no, e))?;
        let re = compile_bre_flags(&translated, case_insensitive, self.expr_no)?;

        Ok(CmdKind::Substitute {
            re,
            replacement,
            global,
            print,
            nth: nth.unwrap_or(1),
        })
    }

    /// Read a `delim...delim` section of the `s` command. Only `\delim`
    /// collapses to a literal delimiter; every other backslash sequence is
    /// preserved (the BRE engine and the replacement expander own them).
    fn read_delimited(&mut self, delimiter: u8) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        loop {
            match self.peek() {
                None => return Err(self.error(self.pos, "unterminated `s' command")),
                Some(b'\\') => {
                    self.pos += 1;
                    match self.peek() {
                        None => return Err(self.error(self.pos, "unterminated `s' command")),
                        Some(d) if d == delimiter => {
                            out.push(d);
                            self.pos += 1;
                        }
                        Some(escaped) => {
                            out.push(b'\\');
                            out.push(escaped);
                            self.pos += 1;
                        }
                    }
                }
                Some(c) if c == delimiter => {
                    self.pos += 1;
                    break;
                }
                Some(byte) => {
                    out.push(byte);
                    self.pos += 1;
                }
            }
        }
        Ok(out)
    }
}

fn compile_bre_flags(
    translated: &str,
    case_insensitive: bool,
    expr_no: usize,
) -> Result<Regex, String> {
    let source = if case_insensitive {
        format!("(?i){translated}")
    } else {
        translated.to_string()
    };
    Regex::new(&source).map_err(|e| format!("sed: -e expression #{expr_no}: invalid BRE: {e}"))
}

fn compile_bre(pattern: &[u8], expr_no: usize, extended: bool) -> Result<Regex, String> {
    let source = String::from_utf8_lossy(pattern).into_owned();
    let translated = if extended {
        bre::to_regex_ere(&source)
    } else {
        bre::to_regex(&source)
    }
    .map_err(|e| format!("sed: -e expression #{expr_no}: {e}"))?;
    compile_bre_flags(&translated, false, expr_no)
}

/// Parse one script expression (a single -e argument or the first positional
/// argument) into a command list. Multiple commands may be separated by `;`.
fn parse_expression(src: &str, expr_no: usize, extended: bool) -> Result<Vec<Command>, String> {
    let mut p = ExprParser::new(src, expr_no, extended);
    let mut commands = Vec::new();
    loop {
        p.skip_blanks_and_semicolons();
        if p.pos >= p.src.len() {
            break;
        }
        let addr_start = p.pos;
        let addr1 = p.parse_address()?;
        p.skip_blanks();
        let addr2 = if p.peek() == Some(b',') {
            p.pos += 1;
            p.skip_blanks();
            match p.parse_address()? {
                Some(addr) => Some(addr),
                None => return Err(p.error(addr_start, "unexpected `,'")),
            }
        } else {
            None
        };
        p.skip_blanks();
        let cmd_pos = p.pos;
        let Some(cmd) = p.peek() else {
            return Err(p.error(cmd_pos, "expected a command"));
        };
        p.pos += 1;
        let kind = match cmd {
            b's' => p.parse_substitute()?,
            b'p' => CmdKind::Print,
            b'd' => CmdKind::Delete,
            b'q' => CmdKind::Quit,
            other => {
                return Err(p.error(cmd_pos, &format!("unknown command: `{}`", other as char)));
            }
        };
        commands.push(Command { addr1, addr2, kind });
    }
    Ok(commands)
}

// ---------- substitution ----------

/// Apply one `s` command: replace the first `nth`-th or every match, expand
/// `&` and `\1`-`\9` in the replacement, and report whether anything
/// changed (for the `p` flag).
fn apply_substitute(
    pattern_space: &[u8],
    re: &Regex,
    replacement: &[u8],
    global: bool,
    nth: u32,
) -> (Vec<u8>, bool) {
    let mut out: Vec<u8> = Vec::with_capacity(pattern_space.len());
    let mut last_end = 0usize;
    let mut count = 0u32;
    let mut changed = false;

    for captures in re.captures_iter(pattern_space) {
        count += 1;
        let whole = captures.get(0).unwrap();
        if !global && count != nth {
            continue;
        }
        out.extend_from_slice(&pattern_space[last_end..whole.start()]);
        expand_replacement(&captures, replacement, &mut out);
        last_end = whole.end();
        changed = true;
    }
    out.extend_from_slice(&pattern_space[last_end..]);
    (out, changed)
}

/// Expand the replacement text: `&` inserts the whole match, `\1`-`\9`
/// insert groups, `\\` and `\&` escape themselves, `\n`/`\t`/`\r` emit
/// control characters like GNU sed, and any other `\x` yields `x`.
fn expand_replacement(captures: &regex::bytes::Captures, replacement: &[u8], out: &mut Vec<u8>) {
    let mut i = 0;
    while i < replacement.len() {
        match replacement[i] {
            b'&' => {
                out.extend_from_slice(captures.get(0).unwrap().as_bytes());
                i += 1;
            }
            b'\\' => {
                i += 1;
                match replacement.get(i) {
                    Some(b'&') => {
                        out.push(b'&');
                        i += 1;
                    }
                    Some(b'\\') => {
                        out.push(b'\\');
                        i += 1;
                    }
                    Some(b'n') => {
                        out.push(b'\n');
                        i += 1;
                    }
                    Some(b't') => {
                        out.push(b'\t');
                        i += 1;
                    }
                    Some(b'r') => {
                        out.push(b'\r');
                        i += 1;
                    }
                    Some(n @ b'1'..=b'9') => {
                        if let Some(group) = captures.get((n - b'0') as usize) {
                            out.extend_from_slice(group.as_bytes());
                        }
                        i += 1;
                    }
                    Some(other) => {
                        out.push(*other);
                        i += 1;
                    }
                    None => {
                        out.push(b'\\');
                    }
                }
            }
            other => {
                out.push(other);
                i += 1;
            }
        }
    }
}

/// Split input into lines the way `read_until(b'\n')` would: the trailing
/// newline is dropped (any `\r` stays part of the line so CRLF survives),
/// and a final chunk without a newline is still a line.
fn split_lines(data: &[u8]) -> Vec<&[u8]> {
    let mut lines = Vec::new();
    let mut start = 0;
    for (i, byte) in data.iter().enumerate() {
        if *byte == b'\n' {
            lines.push(&data[start..i]);
            start = i + 1;
        }
    }
    if start < data.len() {
        lines.push(&data[start..]);
    }
    lines
}

// ---------- execution ----------

/// Apply the command list to one whole input stream (a file or stdin).
/// Output lines always end with `\n`, matching GNU sed; invalid UTF-8 is
/// matched as raw bytes and echoed back unchanged.
fn run_stream(
    commands: &[Command],
    data: &[u8],
    quiet: bool,
    out: &mut dyn Write,
) -> io::Result<()> {
    let lines = split_lines(data);
    let total = lines.len() as u64;
    // Open/closed state per command for two-address ranges.
    let mut range_open = vec![false; commands.len()];

    // GNU sed preserves a missing trailing newline on the input's final line.
    let final_line_unterminated = !data.ends_with(b"\n");

    for (index, raw) in lines.iter().enumerate() {
        let line_num = index as u64 + 1;
        let is_last = line_num == total;
        let mut pattern_space = raw.to_vec();
        let mut deleted = false;
        let mut quit = false;
        let skip_final_newline = is_last && final_line_unterminated;

        for (cmd_index, command) in commands.iter().enumerate() {
            let selected = match (&command.addr1, &command.addr2) {
                (None, _) => true,
                (Some(addr1), None) => addr1.matches(&pattern_space, line_num, is_last),
                (Some(addr1), Some(addr2)) => {
                    if !range_open[cmd_index] {
                        if addr1.matches(&pattern_space, line_num, is_last) {
                            // A second address matching this same line
                            // closes the (single-line) range immediately.
                            range_open[cmd_index] =
                                !addr2.matches(&pattern_space, line_num, is_last);
                            true
                        } else {
                            false
                        }
                    } else {
                        if addr2.matches(&pattern_space, line_num, is_last) {
                            range_open[cmd_index] = false;
                        }
                        true
                    }
                }
            };
            if !selected {
                continue;
            }
            match &command.kind {
                CmdKind::Substitute {
                    re,
                    replacement,
                    global,
                    print,
                    nth,
                } => {
                    let (next, changed) =
                        apply_substitute(&pattern_space, re, replacement, *global, *nth);
                    pattern_space = next;
                    if *print && changed {
                        out.write_all(&pattern_space)?;
                        if !skip_final_newline {
                            out.write_all(b"\n")?;
                        }
                    }
                }
                CmdKind::Print => {
                    out.write_all(&pattern_space)?;
                    if !skip_final_newline {
                        out.write_all(b"\n")?;
                    }
                }
                CmdKind::Delete => {
                    deleted = true;
                    break;
                }
                CmdKind::Quit => {
                    quit = true;
                    break;
                }
            }
        }

        if deleted {
            continue;
        }
        if !quiet {
            out.write_all(&pattern_space)?;
            if !skip_final_newline {
                out.write_all(b"\n")?;
            }
        }
        if quit {
            break;
        }
    }
    Ok(())
}

fn edit_file(
    commands: &[Command],
    quiet: bool,
    file: &Path,
    in_place: Option<&str>,
) -> Result<(), String> {
    let data = fs::read(file).map_err(|e| format!("sed: can't read {}: {}", file.display(), e))?;

    if let Some(suffix) = in_place {
        // Path::new("..").file_name() is None — report it instead of panicking.
        let name = match file.file_name() {
            Some(name) => name.to_os_string(),
            None => {
                return Err(format!(
                    "sed: cannot edit in place: {}: no file name component",
                    file.display()
                ));
            }
        };

        if !suffix.is_empty() {
            let mut backup_name = name.clone();
            backup_name.push(suffix);
            let backup = file.with_file_name(backup_name);
            fs::copy(file, &backup)
                .map_err(|e| format!("sed: can't create backup {}: {}", backup.display(), e))?;
        }

        // Write the edited content to a temp file next to the target and
        // rename it over the original, so a failed write never truncates it.
        let temp = file.with_file_name(format!(
            ".{}.sed-tmp-{}",
            name.to_string_lossy(),
            std::process::id()
        ));
        let written = (|| -> io::Result<()> {
            let tmp = fs::File::create(&temp)?;
            let mut out = BufWriter::new(tmp);
            run_stream(commands, &data, quiet, &mut out)?;
            out.flush()?;
            drop(out);
            fs::rename(&temp, file)?;
            Ok(())
        })();
        if written.is_err() {
            let _ = fs::remove_file(&temp);
        }
        written.map_err(|e| format!("sed: {}: {}", file.display(), e))
    } else {
        let stdout = io::stdout();
        let mut out = BufWriter::new(stdout.lock());
        run_stream(commands, &data, quiet, &mut out)
            .and_then(|()| out.flush())
            .map_err(|e| format!("sed: {}: write error: {}", file.display(), e))
    }
}

fn main() -> ExitCode {
    let args = cli::parse_args();

    // -e scripts take precedence; without any, the first positional argument
    // is the script and the rest are the files (GNU `sed 's/a/b/' file`).
    // -f script files contribute their contents as scripts
    let mut all_expressions = args.expressions.clone();
    for sf in &args.script_files {
        match std::fs::read_to_string(sf) {
            Ok(content) => all_expressions.push(content),
            Err(e) => {
                eprintln!("sed: can't read {}: {}", sf, e);
                return ExitCode::FAILURE;
            }
        }
    }
    let (scripts, files): (Vec<String>, Vec<PathBuf>) = if all_expressions.is_empty() {
        match args.operands.split_first() {
            Some((script, rest)) => (
                vec![script.clone()],
                rest.iter().map(PathBuf::from).collect(),
            ),
            None => (Vec::new(), Vec::new()),
        }
    } else {
        (
            all_expressions.clone(),
            args.operands.iter().map(PathBuf::from).collect(),
        )
    };

    if scripts.is_empty() {
        eprintln!("sed: no expression given");
        return ExitCode::FAILURE;
    }

    // Parse every script up front so nothing is processed on a bad command.
    let mut commands: Vec<Command> = Vec::new();
    for (index, script) in scripts.iter().enumerate() {
        match parse_expression(script, index + 1, args.extended) {
            Ok(mut parsed) => commands.append(&mut parsed),
            Err(message) => {
                eprintln!("{message}");
                return ExitCode::FAILURE;
            }
        }
    }

    let in_place = args.in_place.as_deref();
    let mut failed = false;

    if files.is_empty() {
        let mut data = Vec::new();
        if let Err(e) = io::stdin().read_to_end(&mut data) {
            eprintln!("sed: read error on stdin: {e}");
            return ExitCode::FAILURE;
        }
        let stdout = io::stdout();
        let mut out = BufWriter::new(stdout.lock());
        if let Err(e) = run_stream(&commands, &data, args.quiet, &mut out) {
            eprintln!("sed: write error on stdout: {e}");
            failed = true;
        }
        let _ = out.flush();
    } else {
        for file in &files {
            if let Err(message) = edit_file(&commands, args.quiet, file, in_place) {
                eprintln!("{message}");
                failed = true;
            }
        }
    }

    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
