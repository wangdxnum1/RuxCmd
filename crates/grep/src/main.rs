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

/// Base-name glob used by `--include`/`--exclude`/`--exclude-dir`. `*` and `?`
/// are wildcards; every other character is literal, matching GNU's fnmatch for
/// the patterns these options accept in practice.
fn glob_match(name: &str, pattern: &str) -> bool {
    match glob_regex(pattern) {
        Ok(re) => re.is_match(name),
        Err(_) => false,
    }
}

fn glob_regex(pattern: &str) -> Result<Regex, regex::Error> {
    let mut regex = String::from("^");
    for c in pattern.chars() {
        match c {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            '.' | '(' | ')' | '[' | ']' | '{' | '}' | '+' | '^' | '$' | '|' | '\\' => {
                regex.push('\\');
                regex.push(c);
            }
            _ => regex.push(c),
        }
    }
    regex.push('$');
    Regex::new(&regex)
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

    for line in ByteLines::with_separator(reader, record_separator(args)) {
        let raw = match line {
            Ok(raw) => raw,
            Err(e) => {
                eprintln!("grep: {}: read error: {}", name, e);
                break;
            }
        };
        line_num += 1;

        // After -m's quota is filled, only the trailing context still prints.
        if limit_reached && after_remaining == 0 {
            break;
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
    for line in ByteLines::with_separator(reader, record_separator(args)) {
        let raw = match line {
            Ok(raw) => raw,
            Err(e) => {
                eprintln!("grep: {}: read error: {}", name, e);
                break;
            }
        };
        if line_selected(matcher, args, &raw) {
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
