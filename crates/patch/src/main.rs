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

    let hunks = parse_patch(&patch_content);

    if hunks.is_empty() {
        eprintln!("Error: No valid hunks found in patch file");
        process::exit(1);
    }

    // GNU patch default: no stripping (equivalent to -p0) when the option
    // is omitted.
    let strip = cli.strip.unwrap_or(0);

    let target_file = match &cli.target_file {
        Some(f) => f.clone(),
        None => extract_target_file(&patch_content, strip),
    };

    if target_file.is_empty() {
        eprintln!("error: no target file specified and cannot extract from patch");
        eprintln!("");
        eprintln!("Usage: patch [OPTIONS] <PATCH_FILE> [TARGET_FILE]");
        process::exit(1);
    }

    // A creation patch (`--- /dev/null`, or hunks whose old range is
    // `@@ -0,0`) applies to a file that does not exist yet; treat its old
    // content as empty so the file is created when the result is written.
    let is_creation = is_new_file_patch(&patch_content)
        || hunks.iter().all(|h| h.old_start == 0 && h.old_count == 0);

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

    for hunk in hunks {
        let result = if cli.reverse {
            apply_reverse_hunk(&target_content, &hunk)
        } else {
            apply_hunk(&target_content, &hunk)
        };

        match result {
            Ok(new_content) => {
                target_content = new_content;
            }
            Err(reject) => {
                rejects.push(reject);
                if !cli.force {
                    break;
                }
            }
        }
    }

    if !rejects.is_empty() && !cli.force {
        let reject_file = format!("{}.rej", target_file);
        if let Err(e) = write(&reject_file, rejects.join("\n\n")) {
            eprintln!("Error writing reject file {}: {}", reject_file, e);
        } else {
            eprintln!("Rejects written to {}", reject_file);
        }
        process::exit(1);
    }

    if let Err(e) = write(&target_file, target_content) {
        eprintln!("Error writing to target file {}: {}", target_file, e);
        process::exit(1);
    }

    if !rejects.is_empty() {
        eprintln!("Applied patch with rejects (force mode)");
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
}

fn parse_patch(content: &str) -> Vec<Hunk> {
    let mut hunks = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if lines[i].starts_with("@@") {
            if let Some(hunk) = parse_hunk(&lines, &mut i) {
                hunks.push(hunk);
            }
        } else {
            i += 1;
        }
    }

    hunks
}

fn parse_hunk(lines: &[&str], i: &mut usize) -> Option<Hunk> {
    let header = lines[*i];
    *i += 1;

    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let old_part = parts[1];
    let new_part = parts[2];

    let old_info: Vec<&str> = old_part[1..].split(',').collect();
    let new_info: Vec<&str> = new_part[1..].split(',').collect();

    let old_start: usize = match old_info[0].parse() {
        Ok(n) => n,
        Err(_) => return None,
    };

    let old_count: usize = if old_info.len() > 1 {
        match old_info[1].parse() {
            Ok(n) => n,
            Err(_) => 1,
        }
    } else {
        1
    };

    let new_start: usize = match new_info[0].parse() {
        Ok(n) => n,
        Err(_) => return None,
    };

    let new_count: usize = if new_info.len() > 1 {
        match new_info[1].parse() {
            Ok(n) => n,
            Err(_) => 1,
        }
    } else {
        1
    };

    let mut hunk_lines = Vec::new();

    while *i < lines.len() {
        let line = lines[*i];
        if line.is_empty()
            || line.starts_with("@@")
            || line.starts_with("---")
            || line.starts_with("+++")
        {
            break;
        }

        if line.is_empty() {
            hunk_lines.push((' ', String::new()));
        } else {
            let prefix = line.chars().next().unwrap_or(' ');
            let content = if line.len() > 1 { &line[1..] } else { "" };
            hunk_lines.push((prefix, content.to_string()));
        }

        *i += 1;
    }

    Some(Hunk {
        old_start,
        old_count,
        new_start,
        new_count,
        lines: hunk_lines,
    })
}

fn extract_target_file(content: &str, strip: usize) -> String {
    for line in content.lines() {
        if line.starts_with("+++ ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return strip_path_components(parts[1], strip);
            }
        }
    }
    String::new()
}

/// True when the patch's `---` header names `/dev/null` (git-style file
/// creation).
fn is_new_file_patch(content: &str) -> bool {
    for line in content.lines() {
        if line.starts_with("--- ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                return parts[1] == "/dev/null";
            }
        }
    }
    false
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
    };

    apply_hunk(content, &reversed_hunk)
}
