mod cli;

use clap::Parser;
use std::fs;
use std::path::Path;

fn main() {
    let args = cli::Args::parse();

    if args.files.is_empty() {
        eprintln!("chmod: missing file operand");
        std::process::exit(1);
    }

    let spec = parse_mode(&args.mode).unwrap_or_else(|e| {
        eprintln!("chmod: {}", e);
        std::process::exit(1);
    });

    let mut exit_code = 0i32;

    for file in &args.files {
        if let Err(e) = chmod_file(file, &spec, &args) {
            eprintln!("chmod: {}", e);
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

const PERM_R: u8 = 0b100;
const PERM_W: u8 = 0b010;
const PERM_X: u8 = 0b001;

/// One symbolic-mode clause `[ugoa]*[+-=][rwx]*`. The who list is validated
/// during parsing; on Windows a single readonly bit covers every class, so it
/// is not carried per clause.
struct Clause {
    op: char, // '+', '-', '='
    perms: u8,
}

enum ModeKind {
    Octal(u32),
    Symbolic(Vec<Clause>),
}

struct ModeSpec {
    /// The original mode text, used for verbose output of symbolic modes.
    text: String,
    kind: ModeKind,
}

fn parse_mode(mode_str: &str) -> Result<ModeSpec, String> {
    let invalid = || format!("invalid mode: '{}'", mode_str);

    if !mode_str.is_empty() && mode_str.bytes().all(|b| b.is_ascii_digit()) {
        let mode = u32::from_str_radix(mode_str, 8)
            .map_err(|_| format!("invalid mode: '{}'", mode_str))?;
        if mode > 0o7777 {
            return Err(format!("mode '{}' out of range", mode_str));
        }
        return Ok(ModeSpec {
            text: mode_str.to_string(),
            kind: ModeKind::Octal(mode),
        });
    }

    let mut clauses = Vec::new();
    for part in mode_str.split(',') {
        let clause = parse_clause(part).map_err(|_| invalid())?;
        clauses.push(clause);
    }

    Ok(ModeSpec {
        text: mode_str.to_string(),
        kind: ModeKind::Symbolic(clauses),
    })
}

fn parse_clause(part: &str) -> Result<Clause, ()> {
    let who_len = part
        .chars()
        .take_while(|c| matches!(c, 'u' | 'g' | 'o' | 'a'))
        .count();
    let mut rest = part[who_len..].chars();

    let op = rest.next().ok_or(())?;
    if !matches!(op, '+' | '-' | '=') {
        return Err(());
    }

    let mut perms = 0u8;
    for c in rest {
        match c {
            'r' => perms |= PERM_R,
            'w' => perms |= PERM_W,
            'x' => perms |= PERM_X,
            _ => return Err(()),
        }
    }

    Ok(Clause { op, perms })
}

fn chmod_file(path: &Path, spec: &ModeSpec, args: &cli::Args) -> Result<(), String> {
    let meta = fs::symlink_metadata(path).map_err(|_| {
        format!(
            "cannot access '{}': No such file or directory",
            path.display()
        )
    })?;

    if args.recursive && meta.is_dir() {
        chmod_recursive(path, spec, args)?;
    }

    apply_mode(path, spec)
        .map_err(|e| format!("cannot change permissions of '{}': {}", path.display(), e))?;

    if args.verbose {
        report(path, spec);
    }

    Ok(())
}

fn chmod_recursive(dir: &Path, spec: &ModeSpec, args: &cli::Args) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("cannot read directory '{}': {}", dir.display(), e))?;

    for entry in entries {
        let entry =
            entry.map_err(|e| format!("cannot read entry in '{}': {}", dir.display(), e))?;

        let entry_path = entry.path();

        // symlink_metadata never follows the entry: a junction or directory
        // symlink pointing at an ancestor must not be recursed into.
        let is_dir = fs::symlink_metadata(&entry_path)
            .map(|m| m.is_dir())
            .unwrap_or(false);

        if is_dir {
            chmod_recursive(&entry_path, spec, args)?;
        }

        apply_mode(&entry_path, spec).map_err(|e| {
            format!(
                "cannot change permissions of '{}': {}",
                entry_path.display(),
                e
            )
        })?;

        if args.verbose {
            report(&entry_path, spec);
        }
    }

    Ok(())
}

fn report(path: &Path, spec: &ModeSpec) {
    match &spec.kind {
        ModeKind::Octal(mode) => println!("mode of '{}' changed to {:04o}", path.display(), mode),
        ModeKind::Symbolic(_) => println!("mode of '{}' changed to {}", path.display(), spec.text),
    }
}

fn apply_mode(path: &Path, spec: &ModeSpec) -> std::io::Result<()> {
    match &spec.kind {
        ModeKind::Octal(mode) => set_permissions(path, *mode),
        ModeKind::Symbolic(clauses) => {
            let meta = fs::metadata(path)?;
            let mut perms = meta.permissions();

            #[cfg(windows)]
            {
                // Windows maps every class onto one readonly bit: removing the
                // write permission (op '-' or '=' without 'w') sets readonly,
                // granting it ('+' or '=' with 'w') clears it. 'r' and 'x'
                // have no Windows equivalent and never touch the bit.
                let mut readonly = perms.readonly();
                for clause in clauses {
                    let has_w = clause.perms & PERM_W != 0;
                    match clause.op {
                        '+' => {
                            if has_w {
                                readonly = false;
                            }
                        }
                        '-' => {
                            if has_w {
                                readonly = true;
                            }
                        }
                        _ => readonly = !has_w,
                    }
                }
                perms.set_readonly(readonly);
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                // Vestigial unix path; who classes collapse to 'a'.
                let mut mode = perms.mode() & 0o7777;
                for clause in clauses {
                    let bits = u32::from(clause.perms) * 0o111;
                    match clause.op {
                        '+' => mode |= bits,
                        '-' => mode &= !bits,
                        _ => mode = bits,
                    }
                }
                perms.set_mode(mode);
            }

            fs::set_permissions(path, perms)
        }
    }
}

#[cfg(windows)]
fn set_permissions(path: &Path, mode: u32) -> std::io::Result<()> {
    let mut perms = fs::metadata(path)?.permissions();

    let readonly = (mode & 0o222) == 0;
    perms.set_readonly(readonly);

    fs::set_permissions(path, perms)
}

#[cfg(unix)]
fn set_permissions(path: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let perms = fs::Permissions::from_mode(mode);
    fs::set_permissions(path, perms)
}
