mod cli;

use cli::{Args, Node};
use regex::Regex;
use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match cli::parse(&argv) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("find: {}", message);
            std::process::exit(1);
        }
    };

    // GNU evaluates actions in expression order; -delete also implies -depth.
    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    };

    // GNU appends an implicit -print when the expression has no action.
    let mut nodes = args.nodes.clone();
    if !nodes.iter().any(Node::is_action) {
        nodes.push(Node::Print);
    }

    let batch_count = nodes.len();
    let mut walker = Walker {
        args: &args,
        nodes,
        out: BufWriter::new(io::stdout().lock()),
        batch: vec![Vec::new(); batch_count],
        had_error: false,
    };

    for path in &paths {
        walker.walk(path, 0);
    }
    walker.finish();

    if walker.had_error {
        std::process::exit(1);
    }
}

struct Walker<'a> {
    args: &'a Args,
    /// Expression nodes in evaluation order, including the implicit -print.
    nodes: Vec<Node>,
    out: BufWriter<io::StdoutLock<'static>>,
    /// Paths collected for each `-exec ... {} +` action, keyed by node index.
    batch: Vec<Vec<String>>,
    had_error: bool,
}

impl Walker<'_> {
    fn walk(&mut self, path: &Path, depth: usize) {
        let mindepth = self.args.mindepth.unwrap_or(0);
        let maxdepth = self.args.maxdepth.unwrap_or(usize::MAX);
        if depth > maxdepth {
            return;
        }

        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(e) => {
                eprintln!("find: {}: {}", path.display(), e);
                self.had_error = true;
                return;
            }
        };
        let reparse = is_reparse_point(&metadata);
        let is_dir = metadata.is_dir();

        // -delete implies GNU's -depth: children go before their parent.
        if self.args.delete {
            if is_dir && !reparse && depth < maxdepth {
                self.walk_children(path, depth);
            }
            if depth >= mindepth {
                self.eval(path, &metadata, is_dir, reparse);
            }
            return;
        }

        if depth >= mindepth {
            self.eval(path, &metadata, is_dir, reparse);
        }

        // Like GNU, reparse points (symlinks and junctions) are not followed.
        if is_dir && !reparse && depth < maxdepth {
            self.walk_children(path, depth);
        }
    }

    fn walk_children(&mut self, path: &Path, depth: usize) {
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    self.walk(&entry.path(), depth + 1);
                }
            }
            Err(e) => {
                eprintln!("find: {}: {}", path.display(), e);
                self.had_error = true;
            }
        }
    }

    /// Evaluates the expression left to right with GNU's implicit `-and`
    /// short-circuit: a failing test stops the rest of the expression.
    fn eval(&mut self, path: &Path, metadata: &fs::Metadata, is_dir: bool, reparse: bool) -> bool {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        for index in 0..self.nodes.len() {
            let ok = match self.nodes[index].clone() {
                Node::Name(pattern) => {
                    let (target, pattern) = if self.args.ignore_case {
                        (name.to_lowercase(), pattern.to_lowercase())
                    } else {
                        (name.clone(), pattern)
                    };
                    if pattern.contains('*') || pattern.contains('?') {
                        glob_match(&target, &pattern)
                    } else {
                        target == pattern
                    }
                }
                Node::Regex(re) => re.is_match(&name),
                // A junction is a directory on NTFS, but this build classifies
                // every reparse point as a link, mirroring `-type l`.
                Node::Type(kind) => match kind {
                    'd' => is_dir && !reparse,
                    'f' => metadata.is_file(),
                    'l' => reparse,
                    _ => true,
                },
                Node::Size(spec) => spec.matches(metadata.len()),
                Node::Mtime(spec) => match age_days(metadata) {
                    Some(days) => spec.bound.matches(days, spec.days as i64),
                    None => false,
                },
                Node::Print => {
                    self.write_path(path, false);
                    true
                }
                Node::Print0 => {
                    self.write_path(path, true);
                    true
                }
                Node::Delete => self.delete_entry(path, is_dir),
                Node::Exec(action) => {
                    let text = path_text(path);
                    if action.batch {
                        self.batch[index].push(text);
                        true
                    } else {
                        let status = Command::new(&action.program)
                            .args(action.args.iter().map(|arg| arg.replace("{}", &text)))
                            .status();
                        self.record_status(&action.program, status)
                    }
                }
            };
            if !ok {
                return false;
            }
        }
        true
    }

    fn write_path(&mut self, path: &Path, null: bool) {
        let text = if self.args.basename {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        } else {
            path_text(path)
        };
        let _ = self.out.write_all(text.as_bytes());
        let _ = self.out.write_all(if null { b"\0" } else { b"\n" });
    }

    fn delete_entry(&mut self, path: &Path, is_dir: bool) -> bool {
        let result = if is_dir {
            fs::remove_dir(path)
        } else {
            fs::remove_file(path)
        };
        match result {
            Ok(()) => true,
            Err(e) => {
                eprintln!("find: cannot delete '{}': {}", path.display(), e);
                self.had_error = true;
                false
            }
        }
    }

    fn record_status(
        &mut self,
        program: &str,
        status: io::Result<std::process::ExitStatus>,
    ) -> bool {
        match status {
            Ok(status) if status.success() => true,
            Ok(_) => {
                self.had_error = true;
                false
            }
            Err(e) => {
                eprintln!("find: {}: {}", program, e);
                self.had_error = true;
                false
            }
        }
    }

    fn finish(&mut self) {
        let _ = self.out.flush();
        for index in 0..self.nodes.len() {
            let Node::Exec(action) = self.nodes[index].clone() else {
                continue;
            };
            if !action.batch || self.batch[index].is_empty() {
                continue;
            }
            let paths = std::mem::take(&mut self.batch[index]);
            // Parsing guaranteed `{}` is the last argument, so it expands to
            // every collected path as separate arguments (GNU's `+` form).
            let fixed = &action.args[..action.args.len() - 1];
            let status = Command::new(&action.program)
                .args(fixed)
                .args(&paths)
                .status();
            self.record_status(&action.program, status);
        }
    }
}

fn path_text(path: &Path) -> String {
    path.display().to_string()
}

/// Whole days since the last modification; a future timestamp counts as -1.
fn age_days(metadata: &fs::Metadata) -> Option<i64> {
    let modified = metadata.modified().ok()?;
    match SystemTime::now().duration_since(modified) {
        Ok(elapsed) => Some((elapsed.as_secs() / 86_400) as i64),
        Err(_) => Some(-1),
    }
}

/// Symlinks and NTFS junctions are both reparse points; Windows marks them
/// with FILE_ATTRIBUTE_REPARSE_POINT on the entry itself.
#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

fn glob_match(name: &str, pattern: &str) -> bool {
    // Translate glob wildcards and escape every other regex metacharacter,
    // so a malformed pattern can no longer panic the regex engine.
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
    match Regex::new(&regex) {
        Ok(re) => re.is_match(name),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cli::{Bound, parse_size};

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
}
