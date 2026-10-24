mod cli;

use clap::Parser;
use cli::Args;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// rundll32 export used when no external sync is available. Note that
/// ProcessIdleTasks is a best-effort nudge, not a file-buffer flush.
const FALLBACK_DLL_ENTRY: &str = "advapi32.dll,ProcessIdleTasks";

fn main() {
    let _args = Args::parse();

    std::process::exit(run());
}

/// Resolve and run the external `sync`, propagating its exit code; fall back
/// to `rundll32` when PATH holds no usable external sync.
fn run() -> i32 {
    let self_exe = current_exe_canonical();
    let paths: Vec<PathBuf> = match env::var_os("PATH") {
        Some(value) => env::split_paths(&value)
            .filter(|dir| !dir.as_os_str().is_empty())
            .collect(),
        None => Vec::new(),
    };

    match resolve_external(&paths, &self_exe) {
        Some(external) => run_external(&external, &[]),
        None => run_fallback(),
    }
}

/// Canonical path of the running executable, used to detect self-recursion.
fn current_exe_canonical() -> PathBuf {
    match env::current_exe() {
        Ok(exe) => fs::canonicalize(&exe).unwrap_or(exe),
        Err(_) => PathBuf::new(),
    }
}

/// Find the external `sync.exe` to shell out to, scanning `paths` (the entries
/// of PATH, in order).
///
/// Candidates that resolve to `self_exe` are rejected (case-insensitively,
/// after canonicalization) so this tool never re-spawns itself when its own
/// install directory is on PATH. Returns the canonicalized path of the first
/// usable candidate, or None when PATH holds no external sync.
fn resolve_external(paths: &[PathBuf], self_exe: &Path) -> Option<PathBuf> {
    // Normalize the self reference too: callers may hand in a non-canonical
    // spelling (e.g. an 8.3 short name from TEMP), which would never match
    // the canonicalized candidates.
    let self_norm = fs::canonicalize(self_exe).unwrap_or_else(|_| self_exe.to_path_buf());
    for dir in paths {
        let candidate = dir.join("sync.exe");
        if !candidate.is_file() {
            continue;
        }
        let Ok(resolved) = fs::canonicalize(&candidate) else {
            continue;
        };
        if is_same_path(&resolved, &self_norm) {
            continue;
        }
        return Some(resolved);
    }
    None
}

/// Compare two paths the way Windows treats them: case-insensitively, after
/// normalizing separators and stripping the `\\?\` verbatim prefixes that
/// `fs::canonicalize` produces.
fn is_same_path(a: &Path, b: &Path) -> bool {
    fn normalize(path: &Path) -> String {
        let text = path.to_string_lossy().replace('/', "\\");
        let stripped = match text.strip_prefix(r"\\?\UNC\") {
            Some(rest) => format!(r"\\{rest}"),
            None => match text.strip_prefix(r"\\?\") {
                Some(rest) => rest.to_string(),
                None => text,
            },
        };
        stripped.to_ascii_lowercase()
    }
    normalize(a) == normalize(b)
}

/// Run `program` with `args`, forward its stdout/stderr, and return its exit
/// code (1 when spawning failed or the status carries no exit code).
fn run_external(program: &Path, args: &[&str]) -> i32 {
    let output = match Command::new(program).args(args).output() {
        Ok(output) => output,
        Err(error) => {
            eprintln!("sync: failed to run {}: {error}", program.display());
            return 1;
        }
    };
    if !output.stdout.is_empty() {
        let _ = io::stdout().write_all(&output.stdout);
        let _ = io::stdout().flush();
    }
    if !output.stderr.is_empty() {
        let _ = io::stderr().write_all(&output.stderr);
        let _ = io::stderr().flush();
    }
    output.status.code().unwrap_or(1)
}

/// Best-effort fallback: nudge the system with ProcessIdleTasks, print an
/// English notice to stderr, and exit 0 (documented in docs/compatibility.md).
fn run_fallback() -> i32 {
    eprintln!("sync: no usable external sync found in PATH; falling back to rundll32 {FALLBACK_DLL_ENTRY}");
    match Command::new("rundll32").arg(FALLBACK_DLL_ENTRY).output() {
        Ok(output) => {
            if !output.stderr.is_empty() {
                let _ = io::stderr().write_all(&output.stderr);
                let _ = io::stderr().flush();
            }
        }
        Err(error) => eprintln!("sync: rundll32 fallback failed: {error}"),
    }
    0
}

#[cfg(test)]
mod tests {
    use super::resolve_external;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    #[test]
    fn rejects_candidate_matching_self() {
        let dir = tempdir().unwrap();
        let candidate = dir.path().join("sync.exe");
        fs::write(&candidate, b"").unwrap();
        let found = resolve_external(&[dir.path().to_path_buf()], &candidate);
        assert_eq!(found, None);
    }

    #[test]
    fn rejects_candidate_matching_self_case_insensitively() {
        let dir = tempdir().unwrap();
        let candidate = dir.path().join("sync.exe");
        fs::write(&candidate, b"").unwrap();
        let upper_case = dir.path().join("SYNC.EXE");
        let found = resolve_external(&[dir.path().to_path_buf()], &upper_case);
        assert_eq!(found, None);
    }

    #[test]
    fn rejects_self_and_returns_next_candidate() {
        let self_dir = tempdir().unwrap();
        let self_candidate = self_dir.path().join("sync.exe");
        fs::write(&self_candidate, b"").unwrap();

        let other_dir = tempdir().unwrap();
        let foreign = other_dir.path().join("sync.exe");
        fs::write(&foreign, b"").unwrap();

        let paths: Vec<PathBuf> = vec![
            self_dir.path().to_path_buf(),
            other_dir.path().to_path_buf(),
        ];
        let found = resolve_external(&paths, &self_candidate);
        assert_eq!(found, Some(fs::canonicalize(&foreign).unwrap()));
    }

    #[test]
    fn skips_missing_candidates() {
        let empty_dir = tempdir().unwrap();
        let other_dir = tempdir().unwrap();
        let foreign = other_dir.path().join("sync.exe");
        fs::write(&foreign, b"").unwrap();

        let paths = vec![
            empty_dir.path().to_path_buf(),
            other_dir.path().to_path_buf(),
        ];
        let unrelated_self = PathBuf::from(r"Z:\definitely-not-our-sync.exe");
        let found = resolve_external(&paths, &unrelated_self);
        assert_eq!(found, Some(fs::canonicalize(&foreign).unwrap()));

        assert_eq!(
            resolve_external(&[empty_dir.path().to_path_buf()], &unrelated_self),
            None
        );
    }

    #[test]
    fn external_command_exit_code_is_propagated() {
        use super::run_external;

        let cmd = Path::new("cmd");
        assert_eq!(run_external(cmd, &["/c", "exit 0"]), 0);
        assert_eq!(run_external(cmd, &["/c", "exit 7"]), 7);
        assert_eq!(run_external(cmd, &["/c", "exit 42"]), 42);
    }

    #[test]
    fn missing_external_command_reports_failure_exit_code() {
        use super::run_external;

        // A candidate that exists on PATH but cannot be executed must not
        // silently report success; 1 is the GNU sync failure code.
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing-external.exe");
        assert_eq!(run_external(&missing, &[]), 1);
    }
}
