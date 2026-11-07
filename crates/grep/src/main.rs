mod cli;

use clap::Parser;
use regex::Regex;
use ruxcmd_core::lines::ByteLines;
use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// How lines are matched: a compiled regex, or a literal fixed string (-F).
enum Matcher {
    Regex(Regex),
    Fixed {
        needle: String,
        icase: bool,
        word: bool,
        line: bool,
    },
}

impl Matcher {
    fn is_match(&self, hay: &str) -> bool {
        match self {
            Matcher::Regex(re) => re.is_match(hay),
            Matcher::Fixed {
                needle,
                icase,
                word,
                line,
            } => {
                let (hay_folded, needle_folded) = if *icase {
                    (hay.to_lowercase(), needle.to_lowercase())
                } else {
                    (hay.to_owned(), needle.clone())
                };
                if *line {
                    hay_folded == needle_folded
                } else if *word {
                    needle_folded.is_empty() || fixed_word_match(&hay_folded, &needle_folded)
                } else {
                    hay_folded.contains(&needle_folded)
                }
            }
        }
    }
}

/// Flags every read chunk that contains a NUL byte, so line processing can
/// switch to GNU binary-file mode (message instead of line output, NUL-split
/// segments counted as lines).  Wrapping the BufRead means the flag turns on
/// before any line from that chunk is parsed, which matches GNU's per-block
/// decision.
struct BinaryDetect<R> {
    inner: R,
    /// Set once a chunk with a NUL byte has been seen; shared so line
    /// processing inside the iterator loop can observe it live.
    flag: std::rc::Rc<std::cell::Cell<bool>>,
    /// -a/--text and -z/--null-data disable binary treatment.
    active: bool,
}

impl<R: BufRead> std::io::Read for BinaryDetect<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<R: BufRead> BufRead for BinaryDetect<R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        let buf = self.inner.fill_buf()?;
        if self.active && buf.contains(&0) {
            self.flag.set(true);
        }
        Ok(buf)
    }
    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt);
    }
}

/// GNU's binary-file notice; stdin gets the spelled-out form.
fn binary_label(name: &str) -> &str {
    if name == "<stdin>" {
        "(standard input)"
    } else {
        name
    }
}

fn build_matcher(args: &cli::Args) -> Result<Matcher, String> {
    if args.fixed_strings {
        return Ok(Matcher::Fixed {
            needle: args.pattern.clone(),
            icase: args.ignore_case,
            word: args.word_regexp,
            line: args.line_regexp,
        });
    }

    // The regex crate implements an ERE-like syntax, so -E needs no translation.
    let mut pattern = args.pattern.clone();
    if args.word_regexp {
        pattern = format!("\\b(?:{})\\b", pattern);
    }
    if args.line_regexp {
        pattern = format!("^(?:{})$", pattern);
    }
    if args.ignore_case {
        pattern = format!("(?i){}", pattern);
    }
    Regex::new(&pattern)
        .map(Matcher::Regex)
        .map_err(|e| format!("invalid regular expression: {}", e))
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b >= 0x80
}

/// Whole-word containment for fixed strings: at least one occurrence must not
/// be flanked by word characters (approximating the regex crate's `\b`).
fn fixed_word_match(hay: &str, needle: &str) -> bool {
    let hay_bytes = hay.as_bytes();
    let mut start = 0usize;
    while let Some(rel) = hay[start..].find(needle) {
        let s = start + rel;
        let e = s + needle.len();
        let before_ok = s == 0 || !is_word_byte(hay_bytes[s - 1]);
        let after_ok = e >= hay_bytes.len() || !is_word_byte(hay_bytes[e]);
        if before_ok && after_ok {
            return true;
        }
        start = s + 1;
        while start < hay.len() && !hay.is_char_boundary(start) {
            start += 1;
        }
    }
    false
}

/// Non-overlapping occurrences of a fixed string, for `-o` output.
fn fixed_occurrences(hay: &str, needle: &str, icase: bool) -> Vec<String> {
    let mut hits = Vec::new();
    if needle.is_empty() {
        return hits;
    }
    let (hay_text, needle_text) = if icase {
        (hay.to_lowercase(), needle.to_lowercase())
    } else {
        (hay.to_owned(), needle.to_owned())
    };
    let mut start = 0usize;
    while let Some(rel) = hay_text[start..].find(&needle_text) {
        let s = start + rel;
        let e = s + needle_text.len();
        hits.push(hay_text[s..e].to_string());
        start = e;
    }
    hits
}

fn main() {
    let args = cli::Args::parse();

    if args.fixed_strings && args.extended_regexp {
        eprintln!("grep: conflicting matchers specified");
        std::process::exit(2);
    }

    let matcher = match build_matcher(&args) {
        Ok(m) => m,
        Err(msg) => {
            eprintln!("grep: {}", msg);
            std::process::exit(2);
        }
    };

    // GNU -m 0 stops before opening any input: no output, exit status 1.
    if args.max_count == Some(0) {
        std::process::exit(1);
    }

    let multiple_files = args.files.len() > 1;
    let mut total_count = 0;

    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    if args.files.is_empty() {
        let show_name = args.with_filename && !args.no_filename;
        let stdin = std::io::stdin();
        let count = grep_reader(
            stdin.lock(),
            "<stdin>",
            &matcher,
            &args,
            show_name,
            &mut out,
        );
        if count_output(&args) {
            emit_count(&mut out, "<stdin>", show_name, count);
        }
        total_count += count;
    } else {
        for file in &args.files {
            let file_name = file.display().to_string();
            if file.is_dir() {
                if args.recursive {
                    total_count += grep_directory(file, &matcher, &args, &mut out);
                } else {
                    eprintln!("grep: {}: Is a directory", file_name);
                }
            } else {
                // GNU behavior: a single file operand prints bare lines;
                // several operands (or -H) prefix each line with the name.
                if !file_allowed(file, &args) {
                    continue;
                }
                let show_name = !args.no_filename && (args.with_filename || multiple_files);
                let count = match grep_file(file, &file_name, &matcher, &args, show_name, &mut out)
                {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("grep: {}: {}", file_name, e);
                        continue;
                    }
                };
                if count_output(&args) {
                    emit_count(&mut out, &file_name, show_name, count);
                }
                total_count += count;
            }
        }
    }

    let _ = out.flush();

    std::process::exit(if total_count > 0 { 0 } else { 1 });
}

fn grep_directory(dir: &Path, matcher: &Matcher, args: &cli::Args, out: &mut dyn Write) -> u64 {
    let mut total_count = 0;

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if !dir_excluded(&path, args) {
                    total_count += grep_directory(&path, matcher, args, out);
                }
            } else {
                if !file_allowed(&path, args) {
                    continue;
                }
                let file_name = path.display().to_string();
                let show_name = !args.no_filename;
                if let Ok(count) = grep_file(&path, &file_name, matcher, args, show_name, out) {
                    // A file filtered out by --include/--exclude prints no line.
                    if count_output(args) {
                        emit_count(out, &file_name, show_name, count);
                    }
                    total_count += count;
                }
            }
        }
    }

    total_count
}

/// -c output applies unless it is suppressed by -q or overridden by -l/-L.
fn count_output(args: &cli::Args) -> bool {
    args.count && !args.quiet && !args.files_with_matches && !args.files_without_match
}

/// GNU -c prints one count per input, with `name:count` when the name prefix
/// applies to this input; a zero count is still printed.
fn emit_count(out: &mut dyn Write, name: &str, show_name: bool, count: u64) {
    if show_name {
        let _ = writeln!(out, "{}:{}", name, count);
    } else {
        let _ = writeln!(out, "{}", count);
    }
}

/// Base-name glob used by `--include`/`--exclude`/`--exclude-dir`, translated
/// with fnmatch(3) semantics (what GNU grep applies to these options): `*`
/// matches any run of characters including none, `?` one character, `[abc]`
/// and `[a-z]` a set or range, `[!abc]` its complement (GNU also accepts the
/// regex-style `[^abc]`), a backslash escapes the next character, and every
/// remaining character is literal.
fn glob_match(name: &str, pattern: &str) -> bool {
    match glob_regex(pattern) {
        Ok(re) => re.is_match(name),
        Err(_) => false,
    }
}

fn glob_regex(pattern: &str) -> Result<Regex, regex::Error> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut regex = String::from("^");
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' => {
                regex.push_str(".*");
                i += 1;
            }
            '?' => {
                regex.push('.');
                i += 1;
            }
            '\\' => {
                // fnmatch escapes the next character; a trailing backslash is
                // itself literal.
                if i + 1 < chars.len() {
                    push_regex_literal(&mut regex, chars[i + 1]);
                    i += 2;
                } else {
                    regex.push_str("\\\\");
                    i += 1;
                }
            }
            '[' => match bracket_end(&chars, i) {
                Some(end) => {
                    push_bracket(&chars, i, end, &mut regex);
                    i = end + 1;
                }
                // POSIX: an unterminated bracket expression is a literal `[`.
                None => {
                    push_regex_literal(&mut regex, '[');
                    i += 1;
                }
            },
            c => {
                push_regex_literal(&mut regex, c);
                i += 1;
            }
        }
    }
    regex.push('$');
    Regex::new(&regex)
}

/// Index of the `]` terminating the bracket expression whose `[` sits at
/// `start`, or None when the pattern ends without one. A `]` immediately
/// after `[` (or after the negation) is a literal member, not the terminator,
/// and an escaped `]` never terminates either.
fn bracket_end(chars: &[char], start: usize) -> Option<usize> {
    let mut j = start + 1;
    if j < chars.len() && (chars[j] == '!' || chars[j] == '^') {
        j += 1;
    }
    if j < chars.len() && chars[j] == ']' {
        j += 1;
    }
    while j < chars.len() {
        if chars[j] == '\\' {
            // fnmatch escapes carry inside brackets (GNU grep: `[\a-z]` is
            // the range a-z), so skip the escaped character wholesale.
            j += 2;
        } else if chars[j] == ']' {
            return Some(j);
        } else {
            j += 1;
        }
    }
    None
}

/// Emit the bracket expression spanning `chars[start..end]`, where `end`
/// points at the terminating `]`. fnmatch and regex bracket expressions
/// share range/dash semantics, so members pass through except `!` (negation
/// in globs, regex wants `^`) and the characters the regex engine reads
/// specially inside classes.
fn push_bracket(chars: &[char], start: usize, end: usize, regex: &mut String) {
    regex.push('[');
    let mut j = start + 1;
    if chars[j] == '!' || chars[j] == '^' {
        regex.push('^');
        j += 1;
    }
    while j < end {
        if chars[j] == '\\' && j + 1 < end {
            // An escaped character is one literal member. Emit it plain so
            // ranges like [\a-z] survive (regex would read \a as BEL); only
            // a dash needs escaping, or it would turn into a range operator
            // ([a\-z] is {a, -, z} for fnmatch).
            match chars[j + 1] {
                '-' => regex.push_str("\\-"),
                c => push_class_member(regex, c),
            }
            j += 2;
        } else {
            push_class_member(regex, chars[j]);
            j += 1;
        }
    }
    regex.push(']');
}

/// Emit one character as a class member. Unescaped dashes and ranges keep
/// their fnmatch meaning verbatim; the rest are the characters a regex class
/// would misread (`]`, `[`, `&`, `^`, `\`).
fn push_class_member(regex: &mut String, c: char) {
    if matches!(c, ']' | '[' | '&' | '^' | '\\') {
        regex.push('\\');
    }
    regex.push(c);
}

/// Escape a character that is a regex metacharacter outside a class.
fn push_regex_literal(regex: &mut String, c: char) {
    if matches!(
        c,
        '.' | '(' | ')' | '[' | ']' | '{' | '}' | '+' | '^' | '$' | '|' | '\\' | '*' | '?'
    ) {
        regex.push('\\');
    }
    regex.push(c);
}

fn file_allowed(path: &Path, args: &cli::Args) -> bool {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let included = args.include.is_empty()
        || args
            .include
            .iter()
            .any(|pattern| glob_match(&name, pattern));
    included
        && !args
            .exclude
            .iter()
            .any(|pattern| glob_match(&name, pattern))
}

fn dir_excluded(path: &Path, args: &cli::Args) -> bool {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    args.exclude_dir
        .iter()
        .any(|pattern| glob_match(&name, pattern))
}

fn grep_file(
    path: &Path,
    name: &str,
    matcher: &Matcher,
    args: &cli::Args,
    show_name: bool,
    out: &mut dyn Write,
) -> Result<u64, String> {
    let file = fs::File::open(path).map_err(|e| format!("cannot open file: {}", e))?;
    let reader = BufReader::new(file);
    Ok(grep_reader(reader, name, matcher, args, show_name, out))
}

/// Read lines as raw bytes: matching happens on lossy UTF-8, output echoes the
/// original bytes. A read error is reported on stderr instead of silently
/// truncating the rest of the input.
fn grep_reader<R: BufRead>(
    reader: R,
    name: &str,
    matcher: &Matcher,
    args: &cli::Args,
    show_name: bool,
    out: &mut dyn Write,
) -> u64 {
    if args.files_with_matches {
        return grep_files_with_matches(reader, name, matcher, args, out);
    }
    if args.files_without_match {
        return grep_files_without_match(reader, name, matcher, args, out);
    }
    if args.max_count == Some(0) {
        return 0;
    }

    let before = args.before_context.or(args.context).unwrap_or(0);
    let after = args.after_context.or(args.context).unwrap_or(0);
    let uses_context = before > 0 || after > 0;

    let mut count = 0u64;
    let mut line_num = 0u64;
    let mut pending: VecDeque<(u64, Vec<u8>)> = VecDeque::new();
    let mut after_remaining = 0usize;
    let mut last_printed: Option<u64> = None;
    let mut limit_reached = false;

    let binary_flag = std::rc::Rc::new(std::cell::Cell::new(false));
    let mut detector = BinaryDetect {
        inner: reader,
        flag: binary_flag.clone(),
        // -z makes NUL the record terminator on purpose; -a forces text.
        active: !args.null_data && !args.text,
    };
    let mut binary = false;
    for line in ByteLines::with_separator(&mut detector, record_separator(args)) {
        let raw = match line {
            Ok(raw) => raw,
            Err(e) => {
                eprintln!("grep: {}: read error: {}", name, e);
                break;
            }
        };
        if binary_flag.get() && !binary {
            binary = true;
        }
        line_num += 1;

        // After -m's quota is filled, only the trailing context still prints.
        if limit_reached && after_remaining == 0 {
            break;
        }

        if binary {
            // GNU binary-file mode: NUL-split segments act as lines for
            // counting, and no line output is produced at all.
            for segment in raw.split(|b| *b == 0) {
                let hay = String::from_utf8_lossy(segment);
                if matcher.is_match(&hay) != args.invert_match {
                    count += 1;
                    if args.quiet {
                        return count;
                    }
                    if args.max_count.is_some_and(|max| count >= max) {
                        break;
                    }
                }
            }
            continue;
        }

        let hay = String::from_utf8_lossy(&raw);
        let selected = matcher.is_match(&hay) != args.invert_match;

        if selected && !limit_reached {
            count += 1;
            if args.quiet {
                break;
            }

            if !args.count {
                if uses_context {
                    while let Some((before_num, before_raw)) = pending.pop_front() {
                        if let Some(printed) = last_printed
                            && before_num > printed + 1
                        {
                            emit_separator(out, args);
                        }
                        emit_line(out, name, show_name, before_num, &before_raw, '-', args);
                        last_printed = Some(before_num);
                    }
                    if let Some(printed) = last_printed
                        && line_num > printed + 1
                    {
                        emit_separator(out, args);
                    }
                }
                if args.only_matching {
                    let hits: Vec<String> = match matcher {
                        Matcher::Regex(re) => {
                            re.find_iter(&hay).map(|m| m.as_str().to_string()).collect()
                        }
                        Matcher::Fixed { needle, icase, .. } => {
                            fixed_occurrences(&hay, needle, *icase)
                        }
                    };
                    for hit in &hits {
                        emit_line(out, name, show_name, line_num, hit.as_bytes(), ':', args);
                    }
                } else {
                    emit_line(out, name, show_name, line_num, &raw, ':', args);
                }
                last_printed = Some(line_num);
            }
            after_remaining = after;
            // GNU stops counting at the NUMth match (-m 0 never reads) but
            // still prints the trailing context it owes for that match.
            if args.max_count.is_some_and(|max| count >= max) {
                limit_reached = true;
            }
        } else if after_remaining > 0 {
            after_remaining -= 1;
            if !args.count {
                if let Some(printed) = last_printed
                    && line_num > printed + 1
                {
                    emit_separator(out, args);
                }
                emit_line(out, name, show_name, line_num, &raw, '-', args);
                last_printed = Some(line_num);
            }
        } else if before > 0 {
            pending.push_back((line_num, raw));
            if pending.len() > before {
                pending.pop_front();
            }
        }
    }

    if binary_flag.get() && count > 0 && !args.quiet && !args.count {
        let _ = writeln!(out, "Binary file {} matches", binary_label(name));
    }
    count
}

/// -l: report the file name at the first match and stop reading.
fn grep_files_with_matches<R: BufRead>(
    reader: R,
    name: &str,
    matcher: &Matcher,
    args: &cli::Args,
    out: &mut dyn Write,
) -> u64 {
    let binary_flag = std::rc::Rc::new(std::cell::Cell::new(false));
    let mut detector = BinaryDetect {
        inner: reader,
        flag: binary_flag.clone(),
        active: !args.null_data && !args.text,
    };
    for line in ByteLines::with_separator(&mut detector, record_separator(args)) {
        let raw = match line {
            Ok(raw) => raw,
            Err(e) => {
                eprintln!("grep: {}: read error: {}", name, e);
                break;
            }
        };
        let selected = if binary_flag.get() {
            raw.split(|b| *b == 0).any(|segment| {
                let hay = String::from_utf8_lossy(segment);
                matcher.is_match(&hay) != args.invert_match
            })
        } else {
            line_selected(matcher, args, &raw)
        };
        if selected {
            if !args.quiet {
                let _ = writeln!(out, "{}", name);
            }
            return 1;
        }
    }
    0
}

/// -L: report the file name only when no line was selected. GNU keeps the exit
/// status tied to whether lines matched, so the match count is still returned.
fn grep_files_without_match<R: BufRead>(
    reader: R,
    name: &str,
    matcher: &Matcher,
    args: &cli::Args,
    out: &mut dyn Write,
) -> u64 {
    let mut count = 0u64;
    if args.max_count != Some(0) {
        for line in ByteLines::with_separator(reader, record_separator(args)) {
            let raw = match line {
                Ok(raw) => raw,
                Err(e) => {
                    eprintln!("grep: {}: read error: {}", name, e);
                    break;
                }
            };
            if line_selected(matcher, args, &raw) {
                count += 1;
                if args.max_count.is_some_and(|max| count >= max) {
                    break;
                }
            }
        }
    }
    if count == 0 && !args.quiet {
        let _ = writeln!(out, "{}", name);
    }
    count
}

fn line_selected(matcher: &Matcher, args: &cli::Args, raw: &[u8]) -> bool {
    matcher.is_match(&String::from_utf8_lossy(raw)) != args.invert_match
}

/// `-z` switches both the input and the output record terminator to NUL.
fn record_separator(args: &cli::Args) -> u8 {
    if args.null_data { 0 } else { b'\n' }
}

fn emit_line(
    out: &mut dyn Write,
    name: &str,
    show_name: bool,
    line_num: u64,
    raw: &[u8],
    context_sep: char,
    args: &cli::Args,
) {
    if show_name {
        let _ = write!(out, "{}:", name);
    }
    if args.line_number {
        // The separator sits between the number and the content:
        // `3:match` vs `3-context` (GNU layout).
        let _ = write!(out, "{}{}", line_num, context_sep);
    }
    let _ = out.write_all(raw);
    let _ = out.write_all(&[record_separator(args)]);
}

fn emit_separator(out: &mut dyn Write, args: &cli::Args) {
    let _ = out.write_all(b"--");
    let _ = out.write_all(&[record_separator(args)]);
}

#[cfg(test)]
mod glob_tests {
    use super::glob_match;

    // Expectations cross-checked against GNU grep 3.0 / glibc fnmatch(3).
    #[test]
    fn fnmatch_wildcards_and_literals() {
        assert!(glob_match("main.c", "*.c"));
        assert!(!glob_match("main.rs", "*.c"));
        assert!(glob_match("a1b.txt", "a?b.txt"));
        assert!(!glob_match("ab.txt", "a?b.txt"));
        // A glob dot is literal, not regex-any.
        assert!(glob_match("main.c", "main.c"));
        assert!(!glob_match("mainXc", "main.c"));
        assert!(glob_match("main.c", "main\\.c"));
        assert!(!glob_match("mainXc", "main\\.c"));
        // Backslash escapes the next character; a trailing one is literal.
        assert!(glob_match("a?b.txt", "a\\?b.txt"));
        assert!(!glob_match("a1b.txt", "a\\?b.txt"));
        assert!(glob_match("\\", "\\\\"));
    }

    #[test]
    fn fnmatch_bracket_sets_ranges_and_negation() {
        assert!(glob_match("main.c", "*.[ch]"));
        assert!(glob_match("main.h", "*.[ch]"));
        assert!(!glob_match("main.rs", "*.[ch]"));
        assert!(glob_match("test.txt", "[st]*"));
        assert!(glob_match("start.log", "[st]*"));
        assert!(!glob_match("main.c", "[st]*"));
        assert!(glob_match("apple.txt", "[a-c]*"));
        assert!(glob_match("banana.txt", "[a-c]*"));
        assert!(!glob_match("main.c", "[a-c]*"));
        // Negation: glob writes !, GNU also accepts regex's ^.
        assert!(glob_match("banana.txt", "[!a]*"));
        assert!(!glob_match("apple.txt", "[!a]*"));
        assert!(glob_match("banana.txt", "[^a]*"));
        assert!(!glob_match("apple.txt", "[^a]*"));
    }

    #[test]
    fn fnmatch_bracket_edge_cases() {
        // POSIX: an unterminated bracket expression is a literal `[`.
        assert!(glob_match("[abc", "[abc"));
        assert!(!glob_match("abc", "[abc"));
        // A `]` right after `[` (or after the negation) is a literal member.
        assert!(glob_match("]", "[]x]"));
        assert!(glob_match("x", "[]x]"));
        assert!(!glob_match("]y", "[]x]"));
        assert!(glob_match("x]y", "[]x]*"));
        assert!(glob_match("x", "[!]]"));
        assert!(!glob_match("]", "[!]]"));
        // Leading and trailing dashes are literal members.
        assert!(glob_match("-x.txt", "[a-]*"));
        assert!(glob_match("-x.txt", "[-a]*"));
        assert!(glob_match("-x.txt", "[a\\-z]*"));
        assert!(!glob_match("-x.txt", "[a-z]*"));
    }

    #[test]
    fn fnmatch_escapes_inside_brackets() {
        // GNU grep 3.0: `[\a-z]` escapes the a and keeps the a-z range.
        assert!(glob_match("a1b.txt", "[\\a-z]*"));
        assert!(glob_match("test.txt", "[\\a-z]*"));
        // `[a\z]` is the set {a, z}: b/m/s/t do not match.
        assert!(glob_match("a1b.txt", "[a\\z]*"));
        assert!(!glob_match("banana.txt", "[a\\z]*"));
        // Escaped class metacharacters stay literal members.
        assert!(glob_match("ac", "[a\\]b]c"));
        assert!(glob_match("]c", "[a\\]b]c"));
        assert!(!glob_match("ad", "[a\\]b]c"));
        assert!(glob_match("a[b", "a[\\[]b"));
        assert!(!glob_match("aXb", "a[\\[]b"));
        assert!(glob_match("a", "[a\\\\b]"));
        assert!(glob_match("\\", "[a\\\\b]"));
        assert!(!glob_match("ab", "[a\\\\b]"));
        assert!(glob_match("^", "[a\\^b]"));
        assert!(glob_match("a&b", "a[&]b"));
    }
}
