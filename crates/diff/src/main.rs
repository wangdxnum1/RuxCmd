mod cli;

use cli::Cli;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process;

/// GNU appends this line after any output line whose input had no trailing
/// `\n` (normal format `<`/`>` lines and unified format `-`/`+`/context
/// lines). The marker line itself is newline-terminated.
const NO_NEWLINE_MARKER: &str = "\\ No newline at end of file";

/// One input line: the text without its terminator, plus whether the original
/// line was terminated by `\n`. GNU diff treats a missing final newline as
/// part of the line, so `has_newline` takes part in line equality and drives
/// the "\ No newline at end of file" marker.
struct Line {
    text: String,
    has_newline: bool,
}

fn main() {
    let cli = cli::parse_args();

    if cli.version {
        println!("diff 0.1.0");
        process::exit(0);
    }

    let file1 = match &cli.file1 {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:");
            eprintln!("  <FILE1>");
            eprintln!("  <FILE2>");
            eprintln!();
            eprintln!("Usage: diff [OPTIONS] <FILE1> <FILE2>");
            process::exit(1);
        }
    };

    let file2 = match &cli.file2 {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:");
            eprintln!("  <FILE2>");
            eprintln!();
            eprintln!("Usage: diff [OPTIONS] <FILE1> <FILE2>");
            process::exit(1);
        }
    };

    // GNU: a file operand paired with a directory compares that file with the
    // file of the same name inside the directory.
    let meta1 = fs::metadata(file1).ok();
    let meta2 = fs::metadata(file2).ok();
    let (left, right) = match (&meta1, &meta2) {
        (Some(a), Some(b)) if a.is_dir() && !b.is_dir() => {
            let name = Path::new(file2).file_name().unwrap_or_default();
            let joined = Path::new(file1).join(name);
            (ruxcmd_core::paths::display(&joined), file2.clone())
        }
        (Some(a), Some(b)) if !a.is_dir() && b.is_dir() => {
            let name = Path::new(file1).file_name().unwrap_or_default();
            let joined = Path::new(file2).join(name);
            (file1.clone(), ruxcmd_core::paths::display(&joined))
        }
        _ => (file1.clone(), file2.clone()),
    };

    let left_is_dir = fs::metadata(&left).map(|m| m.is_dir()).unwrap_or(false);
    let right_is_dir = fs::metadata(&right).map(|m| m.is_dir()).unwrap_or(false);

    if left_is_dir && right_is_dir {
        let (found, trouble) = diff_directories(Path::new(&left), Path::new(&right), &cli);
        // GNU diff exit codes: 0 = identical, 1 = differences found, 2 = trouble.
        if trouble {
            process::exit(2);
        }
        if found {
            process::exit(1);
        }
        process::exit(0);
    }

    let lines1 = match read_input(&left) {
        Ok(lines) => lines,
        Err(message) => {
            eprintln!("diff: {}: {}", left, message);
            process::exit(2);
        }
    };

    let lines2 = match read_input(&right) {
        Ok(lines) => lines,
        Err(message) => {
            eprintln!("diff: {}: {}", right, message);
            process::exit(2);
        }
    };

    let found_diff = if cli.unified {
        print_unified_diff(&left, &right, &lines1, &lines2, &cli)
    } else {
        print_normal_diff(&lines1, &lines2, &cli)
    };

    // GNU diff exit codes: 0 = identical, 1 = differences found, 2 = trouble.
    if found_diff {
        process::exit(1);
    }
}

/// GNU-style single-line io error: the OS message without Rust's
/// "(os error N)" suffix.
fn io_message(error: &std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::NotFound => "No such file or directory".to_string(),
        std::io::ErrorKind::PermissionDenied => "Permission denied".to_string(),
        _ => error.to_string(),
    }
}

/// GNU directory comparison: one merged alphabetical pass over the entries of
/// both trees.  With -r subdirectories are compared recursively, without it
/// they are only reported.  Returns (differences_found, trouble).
fn diff_directories(dir1: &Path, dir2: &Path, cli: &Cli) -> (bool, bool) {
    use std::collections::BTreeSet;

    let mut names: BTreeSet<String> = BTreeSet::new();
    let mut trouble = false;
    for dir in [dir1, dir2] {
        match fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            names.insert(entry.file_name().to_string_lossy().into_owned());
                        }
                        Err(error) => {
                            eprintln!(
                                "diff: {}: {}",
                                ruxcmd_core::paths::display(dir),
                                io_message(&error)
                            );
                            trouble = true;
                        }
                    }
                }
            }
            Err(error) => {
                eprintln!(
                    "diff: {}: {}",
                    ruxcmd_core::paths::display(dir),
                    io_message(&error)
                );
                trouble = true;
            }
        }
    }

    let mut found = false;
    for name in &names {
        let path1 = dir1.join(name);
        let path2 = dir2.join(name);
        let is_dir1 = path1.is_dir();
        let is_dir2 = path2.is_dir();
        match (path1.exists(), path2.exists()) {
            (true, false) => {
                println!("Only in {}: {}", ruxcmd_core::paths::display(dir1), name);
                found = true;
            }
            (false, true) => {
                println!("Only in {}: {}", ruxcmd_core::paths::display(dir2), name);
                found = true;
            }
            (false, false) => {}
            (true, true) if is_dir1 && is_dir2 => {
                if cli.recursive {
                    let (sub_found, sub_trouble) = diff_directories(&path1, &path2, cli);
                    found |= sub_found;
                    trouble |= sub_trouble;
                } else {
                    // GNU reports common subdirectories without recursing, and
                    // they alone do not count as a difference.
                    println!(
                        "Common subdirectories: {} and {}",
                        ruxcmd_core::paths::display(&path1),
                        ruxcmd_core::paths::display(&path2)
                    );
                }
            }
            // A file on one side and a directory on the other: GNU reports
            // the type mismatch on stderr and counts it as a difference.
            (true, true) if is_dir1 != is_dir2 => {
                if is_dir1 {
                    println!(
                        "File {} is a directory while file {} is a regular file",
                        ruxcmd_core::paths::display(&path1),
                        ruxcmd_core::paths::display(&path2)
                    );
                } else {
                    println!(
                        "File {} is a regular file while file {} is a directory",
                        ruxcmd_core::paths::display(&path1),
                        ruxcmd_core::paths::display(&path2)
                    );
                }
                found = true;
            }
            (true, true) => {
                let (differed, sub_trouble) = diff_entry_files(&path1, &path2, cli);
                found |= differed;
                trouble |= sub_trouble;
            }
        }
    }
    (found, trouble)
}

/// Compare two files at the same relative position in both trees, printing
/// the `diff [options] left right` header GNU emits before each differing
/// pair.  Returns (differences_found, trouble).
fn diff_entry_files(path1: &Path, path2: &Path, cli: &Cli) -> (bool, bool) {
    let (lines1, trouble1) = read_entry(path1);
    let (lines2, trouble2) = read_entry(path2);
    if trouble1 || trouble2 {
        return (true, true);
    }
    let identical = lines1.len() == lines2.len()
        && lines1
            .iter()
            .zip(lines2.iter())
            .all(|(a, b)| lines_equal(a, b, cli));
    if identical {
        return (false, false);
    }
    let options = header_options(&cli.header_flags);
    println!(
        "diff{} {} {}",
        options,
        ruxcmd_core::paths::display(path1),
        ruxcmd_core::paths::display(path2)
    );
    let differed = if cli.unified {
        print_unified_diff(
            &ruxcmd_core::paths::display(path1),
            &ruxcmd_core::paths::display(path2),
            &lines1,
            &lines2,
            cli,
        )
    } else {
        print_normal_diff(&lines1, &lines2, cli)
    };
    (differed, false)
}

/// Read one entry of a directory walk; read errors are reported and mark the
/// walk as troubled without stopping it.
fn read_entry(path: &Path) -> (Vec<Line>, bool) {
    match read_input(&path.display().to_string()) {
        Ok(lines) => (lines, false),
        Err(message) => {
            eprintln!("diff: {}: {}", ruxcmd_core::paths::display(path), message);
            (Vec::new(), true)
        }
    }
}

/// Directory-walk header options: GNU echoes the comparison flags as typed
/// (`diff -ru ...`, `diff -B -i -r ...`).
fn header_options(flags: &[String]) -> String {
    if flags.is_empty() {
        String::new()
    } else {
        format!(" {}", flags.join(" "))
    }
}

/// GNU unified-format header timestamp, when the platform can produce one.
fn unified_timestamp(name: &str) -> Option<String> {
    let when = if name == "-" {
        std::time::SystemTime::now()
    } else {
        std::fs::metadata(name).ok()?.modified().ok()?
    };
    let local = ruxcmd_core::timefmt::local_time(when)?;
    Some(ruxcmd_core::timefmt::strftime(
        "%Y-%m-%d %H:%M:%S.%N %z",
        &local,
    ))
}

/// Read one operand into lines. `-` stands for stdin; otherwise the operand
/// must be a regular file (directory recursion is not supported).
fn read_input(name: &str) -> Result<Vec<Line>, String> {
    if name == "-" {
        let stdin = std::io::stdin();
        return read_lines(BufReader::new(stdin.lock()), name);
    }

    let path = Path::new(name);
    if path.is_dir() {
        return Err("Is a directory".to_string());
    }

    let file = File::open(path).map_err(|e| io_message(&e))?;
    read_lines(BufReader::new(file), name)
}

/// Byte-level line reading (grep's `read_until` pattern): split on `\n` so
/// the newline termination of the final line survives, which
/// `read_to_string(..).lines()` used to discard. A `\r` before a `\n` is
/// dropped to keep the old `str::lines()` behavior for CRLF input.
fn read_lines<R: BufRead>(mut reader: R, name: &str) -> Result<Vec<Line>, String> {
    let mut lines = Vec::new();
    let mut buf = Vec::new();

    loop {
        buf.clear();
        match reader.read_until(b'\n', &mut buf) {
            Ok(0) => return Ok(lines),
            Ok(_) => {}
            Err(e) => return Err(format!("{}: {}", name, e)),
        }

        let has_newline = buf.last() == Some(&b'\n');
        if has_newline {
            buf.pop();
            if buf.last() == Some(&b'\r') {
                buf.pop();
            }
        }

        match String::from_utf8(std::mem::take(&mut buf)) {
            Ok(text) => lines.push(Line { text, has_newline }),
            // read_to_string used to reject non-UTF-8 input; keep it strict.
            Err(_) => return Err("stream did not contain valid UTF-8".to_string()),
        }
    }
}

/// One step of the LCS backtrack in forward file order.
enum Op {
    /// The same line appears in both files; advances both line pointers.
    Common,
    /// Index into `lines1` of a deleted line.
    Delete(usize),
    /// Index into `lines2` of an added line.
    Add(usize),
}

fn normalize_line(line: &str, cli: &Cli) -> String {
    let mut result = line.to_string();

    if cli.ignore_case {
        result = result.to_lowercase();
    }

    if cli.ignore_whitespace {
        result = result.chars().filter(|c| !c.is_whitespace()).collect();
    }

    result
}

/// GNU treats the missing final newline as part of the line: `b` at EOF and
/// `b\n` are different lines even when the text matches. The exception is
/// `-b`, which folds trailing whitespace and thereby also ignores the
/// final-newline difference (verified against GNU diffutils 3.12).
fn lines_equal(a: &Line, b: &Line, cli: &Cli) -> bool {
    if normalize_line(&a.text, cli) != normalize_line(&b.text, cli) {
        return false;
    }
    cli.ignore_whitespace || a.has_newline == b.has_newline
}

/// `n1,n2` for a multi-line span, `n` for a single line (GNU range folding).
fn format_range(first: usize, last: usize) -> String {
    if first == last {
        first.to_string()
    } else {
        format!("{},{}", first, last)
    }
}

/// Emit one normal-format command for a diff segment: `d` for pure deletes,
/// `a` for pure adds, `c` for a mix. The command is followed by the `< `
/// deletion lines, the `---` separator (only `c` has both sides) and the
/// `> ` addition lines. `anchor1`/`anchor2` are the previous common line
/// numbers of each file (0 when the segment starts the file).
fn print_segment(
    anchor1: usize,
    anchor2: usize,
    dels: &[usize],
    adds: &[usize],
    lines1: &[Line],
    lines2: &[Line],
) {
    let header = if dels.is_empty() {
        format!(
            "{}a{}",
            anchor1,
            format_range(adds[0] + 1, adds[adds.len() - 1] + 1)
        )
    } else if adds.is_empty() {
        format!(
            "{}d{}",
            format_range(dels[0] + 1, dels[dels.len() - 1] + 1),
            anchor2
        )
    } else {
        format!(
            "{}c{}",
            format_range(dels[0] + 1, dels[dels.len() - 1] + 1),
            format_range(adds[0] + 1, adds[adds.len() - 1] + 1)
        )
    };
    println!("{}", header);

    for idx in dels {
        println!("< {}", lines1[*idx].text);
        if !lines1[*idx].has_newline {
            println!("{}", NO_NEWLINE_MARKER);
        }
    }
    if !dels.is_empty() && !adds.is_empty() {
        println!("---");
    }
    for idx in adds {
        println!("> {}", lines2[*idx].text);
        if !lines2[*idx].has_newline {
            println!("{}", NO_NEWLINE_MARKER);
        }
    }
}

fn print_normal_diff(lines1: &[Line], lines2: &[Line], cli: &Cli) -> bool {
    let n = lines1.len();
    let m = lines2.len();

    let mut dp = vec![vec![0; m + 1]; n + 1];

    for i in 1..=n {
        for j in 1..=m {
            let skip1 = cli.ignore_blank_lines && lines1[i - 1].text.trim().is_empty();
            let skip2 = cli.ignore_blank_lines && lines2[j - 1].text.trim().is_empty();

            if lines_equal(&lines1[i - 1], &lines2[j - 1], cli) {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else if skip1 || skip2 {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut i = n;
    let mut j = m;
    let mut ops: Vec<Op> = Vec::new();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 {
            let skip1 = cli.ignore_blank_lines && lines1[i - 1].text.trim().is_empty();
            let skip2 = cli.ignore_blank_lines && lines2[j - 1].text.trim().is_empty();

            if lines_equal(&lines1[i - 1], &lines2[j - 1], cli) || (skip1 && skip2) {
                ops.push(Op::Common);
                i -= 1;
                j -= 1;
            } else if skip1 {
                i -= 1;
            } else if skip2 {
                j -= 1;
            } else if dp[i - 1][j] > dp[i][j - 1] {
                ops.push(Op::Delete(i - 1));
                i -= 1;
            } else {
                ops.push(Op::Add(j - 1));
                j -= 1;
            }
        } else if i > 0 {
            ops.push(Op::Delete(i - 1));
            i -= 1;
        } else {
            ops.push(Op::Add(j - 1));
            j -= 1;
        }
    }

    ops.reverse();

    // Group the op sequence into diff segments: maximal runs of Delete/Add
    // with no Common line in between. GNU normal format emits exactly one
    // command per segment; a single common line always separates commands.
    // `i`/`j` are the next unprocessed 1-based line numbers of each file, so
    // `i - 1`/`j - 1` are the line numbers the anchors point at.
    let mut i: usize = 1;
    let mut j: usize = 1;
    let mut found_diff = false;

    let mut seg_del: Vec<usize> = Vec::new();
    let mut seg_add: Vec<usize> = Vec::new();

    for op in &ops {
        match *op {
            Op::Common => {
                if !seg_del.is_empty() || !seg_add.is_empty() {
                    print_segment(i - 1, j - 1, &seg_del, &seg_add, lines1, lines2);
                    found_diff = true;
                    seg_del.clear();
                    seg_add.clear();
                }
                i += 1;
                j += 1;
            }
            Op::Delete(idx) => {
                seg_del.push(idx);
                i += 1;
            }
            Op::Add(idx) => {
                seg_add.push(idx);
                j += 1;
            }
        }
    }

    // Trailing segment: the ops can end on a diff segment with no following
    // Common line, e.g. a diff at the end of the file.
    if !seg_del.is_empty() || !seg_add.is_empty() {
        print_segment(i - 1, j - 1, &seg_del, &seg_add, lines1, lines2);
        found_diff = true;
    }

    found_diff
}

fn print_unified_diff(
    file1: &str,
    file2: &str,
    lines1: &[Line],
    lines2: &[Line],
    cli: &Cli,
) -> bool {
    let n = lines1.len();
    let m = lines2.len();

    let mut dp = vec![vec![0; m + 1]; n + 1];

    for i in 1..=n {
        for j in 1..=m {
            let skip1 = cli.ignore_blank_lines && lines1[i - 1].text.trim().is_empty();
            let skip2 = cli.ignore_blank_lines && lines2[j - 1].text.trim().is_empty();

            if lines_equal(&lines1[i - 1], &lines2[j - 1], cli) {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else if skip1 || skip2 {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut i = n;
    let mut j = m;
    // (line1, line2, is_common, text, has_newline); common lines share the
    // newline status of both sides because equality requires them to match.
    let mut diffs: Vec<(usize, usize, bool, String, bool)> = Vec::new();

    while i > 0 || j > 0 {
        if i > 0 && j > 0 {
            let skip1 = cli.ignore_blank_lines && lines1[i - 1].text.trim().is_empty();
            let skip2 = cli.ignore_blank_lines && lines2[j - 1].text.trim().is_empty();

            if lines_equal(&lines1[i - 1], &lines2[j - 1], cli) || (skip1 && skip2) {
                diffs.push((
                    i,
                    j,
                    true,
                    lines1[i - 1].text.clone(),
                    lines1[i - 1].has_newline,
                ));
                i -= 1;
                j -= 1;
            } else if skip1 {
                i -= 1;
            } else if skip2 {
                j -= 1;
            } else if dp[i - 1][j] > dp[i][j - 1] {
                diffs.push((
                    i,
                    0,
                    false,
                    lines1[i - 1].text.clone(),
                    lines1[i - 1].has_newline,
                ));
                i -= 1;
            } else {
                diffs.push((
                    0,
                    j,
                    false,
                    lines2[j - 1].text.clone(),
                    lines2[j - 1].has_newline,
                ));
                j -= 1;
            }
        } else if i > 0 {
            diffs.push((
                i,
                0,
                false,
                lines1[i - 1].text.clone(),
                lines1[i - 1].has_newline,
            ));
            i -= 1;
        } else {
            diffs.push((
                0,
                j,
                false,
                lines2[j - 1].text.clone(),
                lines2[j - 1].has_newline,
            ));
            j -= 1;
        }
    }

    diffs.reverse();

    let found_diff = diffs.iter().any(|d| !d.2);

    // GNU prints nothing at all — not even the ---/+++ headers — when the
    // inputs are identical.
    if !found_diff {
        return false;
    }

    match unified_timestamp(file1) {
        Some(stamp) => println!("--- {file1}\t{stamp}"),
        None => println!("--- {file1}"),
    }
    match unified_timestamp(file2) {
        Some(stamp) => println!("+++ {file2}\t{stamp}"),
        None => println!("+++ {file2}"),
    }

    let mut i = 1;
    let mut j = 1;
    let mut in_hunk = false;
    let mut hunk_start = 0;
    let mut hunk_lines1 = 0;
    let mut hunk_lines2 = 0;
    let mut hunk_content: Vec<String> = Vec::new();

    for diff in &diffs {
        if diff.2 {
            if in_hunk {
                hunk_lines1 += 1;
                hunk_lines2 += 1;
                hunk_content.push(format!(" {}", diff.3));
                if !diff.4 {
                    hunk_content.push(NO_NEWLINE_MARKER.to_string());
                }
            }
            i += 1;
            j += 1;
        } else {
            if !in_hunk {
                in_hunk = true;
                hunk_start = i.min(j);
                hunk_lines1 = 0;
                hunk_lines2 = 0;
                hunk_content.clear();
            }

            if diff.0 > 0 {
                hunk_lines1 += 1;
                hunk_content.push(format!("-{}", diff.3));
                if !diff.4 {
                    hunk_content.push(NO_NEWLINE_MARKER.to_string());
                }
                i += 1;
            } else {
                hunk_lines2 += 1;
                hunk_content.push(format!("+{}", diff.3));
                if !diff.4 {
                    hunk_content.push(NO_NEWLINE_MARKER.to_string());
                }
                j += 1;
            }
        }
    }

    if in_hunk {
        print_hunk(hunk_start, hunk_lines1, hunk_lines2, &hunk_content);
    }

    found_diff
}

fn print_hunk(start: usize, lines1: usize, lines2: usize, content: &[String]) {
    // GNU omits a count of exactly 1 (`@@ -1 +2,3 @@`) and keeps `,0` for
    // empty sides (`@@ -0,0 +1 @@`).
    let range = |start: usize, count: usize| {
        if count == 1 {
            format!("{start}")
        } else {
            format!("{start},{count}")
        }
    };
    println!("@@ -{} +{} @@", range(start, lines1), range(start, lines2));

    for line in content {
        println!("{}", line);
    }
}
