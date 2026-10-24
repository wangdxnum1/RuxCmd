mod cli;

use clap::Parser;
use cli::Cli;
use std::fs::{read_to_string, write};
use std::process;

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("patch 0.1.0");
        process::exit(0);
    }

    let patch_file = match &cli.patch_file {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:");
            eprintln!("  <PATCH_FILE>");
            eprintln!("");
            eprintln!("Usage: patch [OPTIONS] <PATCH_FILE> [TARGET_FILE]");
            process::exit(1);
        }
    };

    let patch_content = match read_to_string(patch_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading patch file {}: {}", patch_file, e);
            process::exit(1);
        }
    };

    let sections = match parse_patch(&patch_content) {
        Ok(sections) => sections,
        Err(line_no) => {
            eprintln!("patch: **** malformed patch at line {}", line_no);
            process::exit(1);
        }
    };

    let hunks_total: usize = sections.iter().map(|s| s.hunks.len()).sum();
    if hunks_total == 0 {
        eprintln!("Error: No valid hunks found in patch file");
        process::exit(1);
    }

    // GNU patch default: no stripping (equivalent to -p0) when the option
    // is omitted.
    let strip = cli.strip.unwrap_or(0);

    let mut any_rejects = false;

    for section in &sections {
        // Each `---`/`+++` section targets its own file; an explicit
        // TARGET_FILE operand overrides the names for every section.
        let target_file = match &cli.target_file {
            Some(f) => f.clone(),
            None => {
                // A deletion section (`+++ /dev/null`) patches the file
                // named on the `---` line.
                let raw_name = if section.new_name == "/dev/null" && section.old_name != "/dev/null"
                {
                    &section.old_name
                } else {
                    &section.new_name
                };
                strip_path_components(raw_name, strip)
            }
        };

        if target_file.is_empty() {
            eprintln!("error: no target file specified and cannot extract from patch");
            eprintln!("");
            eprintln!("Usage: patch [OPTIONS] <PATCH_FILE> [TARGET_FILE]");
            process::exit(1);
        }

        // A creation section (`--- /dev/null`, or hunks whose old range is
        // `@@ -0,0`) applies to a file that does not exist yet; treat its
        // old content as empty so the file is created when written.
        let is_creation = section.old_name == "/dev/null"
            || section
                .hunks
                .iter()
                .all(|h| h.old_start == 0 && h.old_count == 0);

        let mut target_content = match read_to_string(&target_file) {
            Ok(content) => content,
            Err(e) => {
                if is_creation {
                    String::new()
                } else {
                    eprintln!("Error reading target file {}: {}", target_file, e);
                    process::exit(1);
                }
            }
        };

        let mut rejects: Vec<String> = Vec::new();
        let mut applied = 0usize;

        for hunk in &section.hunks {
            let result = if cli.reverse {
                apply_reverse_hunk(&target_content, hunk)
            } else {
                apply_hunk(&target_content, hunk)
            };

            match result {
                Ok(new_content) => {
                    target_content = new_content;
                    applied += 1;
                }
                Err(_reject_reason) => {
                    // .rej carries the hunk's original lines, not a message.
                    rejects.push(hunk.raw.join("\n"));
                }
            }
        }

        // GNU semantics: hunks that applied are written back even when
        // others were rejected.
        if applied > 0 {
            if let Err(e) = write(&target_file, target_content) {
                eprintln!("Error writing to target file {}: {}", target_file, e);
                process::exit(1);
            }
        }

        if !rejects.is_empty() {
            let reject_file = format!("{}.rej", target_file);
            if let Err(e) = write(&reject_file, rejects.join("\n\n")) {
                eprintln!("Error writing reject file {}: {}", reject_file, e);
            } else {
                eprintln!(
                    "{} out of {} hunks FAILED for {} -- saving rejects to file {}",
                    rejects.len(),
                    section.hunks.len(),
                    target_file,
                    reject_file
                );
            }
            any_rejects = true;
        }
    }

    if any_rejects {
        if cli.force {
            println!("Applied patch with rejects (force mode)");
        } else {
            process::exit(1);
        }
    } else {
        println!("Applied patch successfully");
    }
}

#[derive(Debug)]
struct Hunk {
    old_start: usize,
    old_count: usize,
    new_start: usize,
    new_count: usize,
    lines: Vec<(char, String)>,
    /// The hunk's original patch lines (header and body, including
    /// `\ No newline` markers), used verbatim for .rej files.
    raw: Vec<String>,
}

/// One file's section of a (possibly multi-file) patch: the `---`/`+++`
/// header pair plus every hunk that follows until the next `---` header.
#[derive(Debug)]
struct FileSection {
    old_name: String,
    new_name: String,
    hunks: Vec<Hunk>,
}

/// Parses the patch into per-file sections. Hunks are consumed strictly by
/// the line counts declared in their `@@` header, so empty body lines and
/// lines starting with `---` (deletions of `--...` content) never truncate
/// a hunk. Returns Err(line number) for a malformed patch.
fn parse_patch(content: &str) -> Result<Vec<FileSection>, usize> {
    let lines: Vec<&str> = content.lines().collect();
    let mut sections: Vec<FileSection> = Vec::new();
    let mut current: Option<FileSection> = None;
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("--- ") {
            if i + 1 >= lines.len() || !lines[i + 1].starts_with("+++ ") {
                return Err(i + 1);
            }
            if let Some(prev) = current.take() {
                sections.push(prev);
            }
            current = Some(FileSection {
                old_name: header_path(line),
                new_name: header_path(lines[i + 1]),
                hunks: Vec::new(),
            });
            i += 2;
        } else if line.starts_with("@@") {
            if current.is_none() {
                // Hunks before any ---/+++ header: an implicit section
                // whose target must come from the TARGET_FILE operand.
                current = Some(FileSection {
                    old_name: String::new(),
                    new_name: String::new(),
                    hunks: Vec::new(),
                });
            }
            let hunk = parse_hunk(&lines, &mut i)?;
            current.as_mut().unwrap().hunks.push(hunk);
        } else {
            i += 1;
        }
    }

    if let Some(last) = current {
        sections.push(last);
    }

    Ok(sections)
}

fn header_path(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].to_string()
    } else {
        String::new()
    }
}

/// Consumes exactly the number of body lines declared by the hunk header:
/// context lines count for both sides, `-` lines for old, `+` lines for
/// new; `\ No newline...` markers are carried into `raw` but not counted.
fn parse_hunk(lines: &[&str], i: &mut usize) -> Result<Hunk, usize> {
    let header = lines[*i];
    let header_line_no = *i + 1;
    *i += 1;

    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(header_line_no);
    }

    let old_part = parts[1];
    let new_part = parts[2];

    let old_info: Vec<&str> = old_part[1..].split(',').collect();
    let new_info: Vec<&str> = new_part[1..].split(',').collect();

    let old_start: usize = match old_info[0].parse() {
        Ok(n) => n,
        Err(_) => return Err(header_line_no),
    };

    let old_count: usize = if old_info.len() > 1 {
        match old_info[1].parse() {
            Ok(n) => n,
            Err(_) => return Err(header_line_no),
        }
    } else {
        1
    };

    let new_start: usize = match new_info[0].parse() {
        Ok(n) => n,
        Err(_) => return Err(header_line_no),
    };

    let new_count: usize = if new_info.len() > 1 {
        match new_info[1].parse() {
            Ok(n) => n,
            Err(_) => return Err(header_line_no),
        }
    } else {
        1
    };

    let mut hunk_lines = Vec::new();
    let mut raw = vec![header.to_string()];
    let mut old_seen = 0usize;
    let mut new_seen = 0usize;

    while old_seen < old_count || new_seen < new_count {
        if *i >= lines.len() {
            return Err(header_line_no);
        }
        let line = lines[*i];

        // "\ No newline at end of file" annotates the previous line and is
        // never counted as hunk content.
        if line.starts_with('\\') {
            raw.push(line.to_string());
            *i += 1;
            continue;
        }

        // A bare empty line stands for an empty context line (its leading
        // space is commonly stripped by editors/mail).
        let (prefix, body) = if line.is_empty() {
            (' ', "")
        } else {
            match line.as_bytes()[0] {
                b' ' | b'+' | b'-' => (line.as_bytes()[0] as char, &line[1..]),
                _ => return Err(header_line_no),
            }
        };

        match prefix {
            ' ' => {
                old_seen += 1;
                new_seen += 1;
            }
            '-' => old_seen += 1,
            _ => new_seen += 1,
        }

        hunk_lines.push((prefix, body.to_string()));
        raw.push(line.to_string());
        *i += 1;
    }

    Ok(Hunk {
        old_start,
        old_count,
        new_start,
        new_count,
        lines: hunk_lines,
        raw,
    })
}

/// Strips `strip` leading components from a `---`/`+++` header path, GNU
/// patch style: `-pN` removes the smallest prefix containing N leading
/// slashes, so `b/file.txt` with `-p1` becomes `file.txt`, `/a/b/c` with
/// `-p2` becomes `b/c`, and `C:/dir/file.txt` with `-p1` becomes
/// `dir/file.txt` (drive letter and root slash are stripped together).
/// `/dev/null` is never stripped. Returns an empty string when no name
/// remains after stripping.
fn strip_path_components(path: &str, strip: usize) -> String {
    if strip == 0 || path == "/dev/null" {
        return path.to_string();
    }
    let mut rest = path;
    for _ in 0..strip {
        match rest.find('/') {
            Some(idx) => rest = &rest[idx + 1..],
            None => return String::new(),
        }
    }
    if rest.is_empty() {
        // The path ended in a slash; no file name remains.
        return String::new();
    }
    rest.to_string()
}

fn apply_hunk(content: &str, hunk: &Hunk) -> Result<String, String> {
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // A `@@ -0,0` creation hunk has no old content and starts at line 0;
    // saturate so the 0-based start does not underflow.
    let old_start = hunk.old_start.saturating_sub(1);
    let old_end = old_start + hunk.old_count;

    if old_end > lines.len() {
        return Err(format!(
            "Hunk {} out of range: file has {} lines, hunk expects up to line {}",
            hunk.old_start,
            lines.len(),
            old_end
        ));
    }

    let mut new_lines: Vec<String> = Vec::new();
    let mut line_idx = 0;

    // Lines before the hunk's old range pass through unchanged. The range
    // check above guarantees old_start <= lines.len().
    while line_idx < old_start {
        new_lines.push(lines[line_idx].clone());
        line_idx += 1;
    }

    // Apply the hunk body. Additions may extend past the end of the old
    // content (creation/append hunks); '-' and ' ' lines guard against
    // running past EOF below.
    for (prefix, hunk_line) in hunk.lines.iter() {
        match *prefix {
            '-' => {
                if line_idx < lines.len() && lines[line_idx] == *hunk_line {
                    line_idx += 1;
                } else {
                    return Err(format!(
                        "Hunk {} reject: expected '{}' but got '{}'",
                        hunk.old_start,
                        hunk_line,
                        if line_idx < lines.len() {
                            &lines[line_idx]
                        } else {
                            "EOF"
                        }
                    ));
                }
            }
            '+' => {
                new_lines.push(hunk_line.clone());
            }
            ' ' => {
                if line_idx < lines.len() && lines[line_idx] == *hunk_line {
                    new_lines.push(lines[line_idx].clone());
                    line_idx += 1;
                } else {
                    return Err(format!(
                        "Hunk {} reject: context mismatch '{}'",
                        hunk.old_start, hunk_line
                    ));
                }
            }
            _ => {}
        }
    }

    while line_idx < lines.len() {
        new_lines.push(lines[line_idx].clone());
        line_idx += 1;
    }

    Ok(new_lines.join("\n") + "\n")
}

fn apply_reverse_hunk(content: &str, hunk: &Hunk) -> Result<String, String> {
    let reversed_hunk = Hunk {
        old_start: hunk.new_start,
        old_count: hunk.new_count,
        new_start: hunk.old_start,
        new_count: hunk.old_count,
        lines: hunk
            .lines
            .iter()
            .map(|(prefix, content)| match prefix {
                '-' => ('+', content.clone()),
                '+' => ('-', content.clone()),
                ' ' => (' ', content.clone()),
                _ => (' ', content.clone()),
            })
            .collect(),
        raw: hunk.raw.clone(),
    };

    apply_hunk(content, &reversed_hunk)
}
