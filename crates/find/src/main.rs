mod cli;

use cli::{Args, ExecAction, Expr, Node, StampKind};
use regex::Regex;
use ruxcmd_core::command_batch::CommandBudget;
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

    let mut walker = Walker {
        args: &args,
        out: BufWriter::new(io::stdout().lock()),
        // One buffer per `-exec ... +` action, keyed by its parse-time id.
        batch: (0..args.exec_count).map(|_| ExecBatch::default()).collect(),
        had_error: false,
    };

    for path in &paths {
        walker.walk(&args.expr, path, 0);
    }
    walker.finish(&args.expr);

    if walker.had_error {
        std::process::exit(1);
    }
}

struct Walker<'a> {
    args: &'a Args,
    out: BufWriter<io::StdoutLock<'static>>,
    /// Paths collected for each `-exec ... {} +` action, keyed by action id.
    batch: Vec<ExecBatch>,
    had_error: bool,
}

#[derive(Default)]
struct ExecBatch {
    items: Vec<String>,
    budget: Option<CommandBudget>,
    initialized: bool,
}

impl Walker<'_> {
    fn walk(&mut self, expr: &Expr, path: &Path, depth: usize) {
        let mindepth = self.args.mindepth.unwrap_or(0);
        let maxdepth = self.args.maxdepth.unwrap_or(usize::MAX);
        if depth > maxdepth {
            return;
        }

        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(e) => {
                eprintln!("find: {}: {}", path.display(), cli::io_message(&e));
                self.had_error = true;
                return;
            }
        };
        let reparse = is_reparse_point(&metadata);
        let is_dir = metadata.is_dir();

        // -delete implies GNU's -depth: children go before their parent.
        if self.args.delete {
            if is_dir && !reparse && depth < maxdepth {
                self.walk_children(expr, path, depth);
            }
            if depth >= mindepth {
                self.eval(expr, path, &metadata, is_dir, reparse);
            }
            return;
        }

        if depth >= mindepth {
            self.eval(expr, path, &metadata, is_dir, reparse);
        }

        // Like GNU, reparse points (symlinks and junctions) are not followed.
        if is_dir && !reparse && depth < maxdepth {
            self.walk_children(expr, path, depth);
        }
    }

    fn walk_children(&mut self, expr: &Expr, path: &Path, depth: usize) {
        match fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    self.walk(expr, &entry.path(), depth + 1);
                }
            }
            Err(e) => {
                eprintln!("find: {}: {}", path.display(), cli::io_message(&e));
                self.had_error = true;
            }
        }
    }

    /// Evaluates the expression left to right with GNU's short-circuit rules:
    /// `-a` stops at the first failure, `-o` at the first success, and `!`
    /// negates. Tests and actions are all leaves, so `-print -delete` keeps
    /// its command-line order.
    fn eval(
        &mut self,
        expr: &Expr,
        path: &Path,
        metadata: &fs::Metadata,
        is_dir: bool,
        reparse: bool,
    ) -> bool {
        match expr {
            Expr::And(left, right) => {
                self.eval(left, path, metadata, is_dir, reparse)
                    && self.eval(right, path, metadata, is_dir, reparse)
            }
            Expr::Or(left, right) => {
                self.eval(left, path, metadata, is_dir, reparse)
                    || self.eval(right, path, metadata, is_dir, reparse)
            }
            Expr::Not(inner) => !self.eval(inner, path, metadata, is_dir, reparse),
            Expr::Primary(node) => self.eval_node(node, path, metadata, is_dir, reparse),
        }
    }

    fn eval_node(
        &mut self,
        node: &Node,
        path: &Path,
        metadata: &fs::Metadata,
        is_dir: bool,
        reparse: bool,
    ) -> bool {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        match node {
            Node::Name(pattern) => {
                let ignore_case = self.args.ignore_case;
                let target = if ignore_case {
                    name.to_lowercase()
                } else {
                    name
                };
                let pattern = if ignore_case {
                    pattern.to_lowercase()
                } else {
                    pattern.clone()
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
            Node::Newer { instant, kind } => {
                let stamp = match kind {
                    StampKind::Modified => metadata.modified(),
                    StampKind::Accessed => metadata.accessed(),
                };
                match stamp {
                    // Strictly newer, like GNU.
                    Ok(stamp) => stamp > *instant,
                    Err(_) => false,
                }
            }
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
                    self.queue_exec(action, text);
                    true
                } else {
                    let status = Command::new(&action.program)
                        .args(action.args.iter().map(|arg| arg.replace("{}", &text)))
                        .status();
                    self.record_status(&action.program, status)
                }
            }
        }
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

    fn finish(&mut self, expr: &Expr) {
        let _ = self.out.flush();
        self.flush_batches(expr);
    }

    fn queue_exec(&mut self, action: &ExecAction, text: String) {
        let id = action.id;
        if !self.batch[id].initialized {
            self.batch[id].initialized = true;
            let fixed: Vec<_> = std::iter::once(action.program.as_str())
                .chain(
                    action.args[..action.args.len() - 1]
                        .iter()
                        .map(String::as_str),
                )
                .collect();
            match CommandBudget::new(&fixed) {
                Ok(budget) => self.batch[id].budget = Some(budget),
                Err(error) => {
                    eprintln!("find: {}: {error}", action.program);
                    self.had_error = true;
                }
            }
        }
        let Some(budget) = &self.batch[id].budget else {
            return;
        };
        if !budget.can_fit_item(&text) {
            self.flush_exec(action);
            eprintln!("find: argument exceeds command-line limit: '{text}'");
            self.had_error = true;
            return;
        }
        if !self.batch[id].budget.as_mut().unwrap().try_add(&text) {
            self.flush_exec(action);
            assert!(self.batch[id].budget.as_mut().unwrap().try_add(&text));
        }
        self.batch[id].items.push(text);
    }

    fn flush_exec(&mut self, action: &ExecAction) {
        let batch = &mut self.batch[action.id];
        if batch.items.is_empty() {
            return;
        }
        let paths = std::mem::take(&mut batch.items);
        batch.budget.as_mut().unwrap().reset();
        let status = Command::new(&action.program)
            .args(&action.args[..action.args.len() - 1])
            .args(paths)
            .status();
        self.record_status(&action.program, status);
    }

    /// Flush remaining `+` batches at the end, in tree order.
    fn flush_batches(&mut self, expr: &Expr) {
        match expr {
            Expr::Primary(node) => {
                if let Node::Exec(action) = node
                    && action.batch
                {
                    self.flush_exec(action);
                }
            }
            Expr::Not(inner) => self.flush_batches(inner),
            Expr::And(left, right) | Expr::Or(left, right) => {
                self.flush_batches(left);
                self.flush_batches(right);
            }
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
