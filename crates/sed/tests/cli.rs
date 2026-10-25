//! Round 12 batch E1: P1 remediation tests for sed.
//! Command set p/d/q, addresses (line number / $ / /substring/ and ranges),
//! positional-argument script form, `-n`, atomic `-i`, CRLF and non-UTF-8
//! byte preservation. Std-only on purpose (no new dependencies).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, SystemTime};

fn sed(args: &[&str], stdin: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sed"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // sed may exit early on a usage error before reading stdin; ignore EPIPE.
    let _ = child.stdin.as_mut().unwrap().write_all(stdin);
    drop(child.stdin.take());
    child.wait_with_output().unwrap()
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sed-e1-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_bytes(path: &Path, data: &[u8]) {
    std::fs::write(path, data).unwrap();
}

fn dir_entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

// --- P1: positional-argument script form: `sed 's/a/b/' file` ---

#[test]
fn positional_script_with_single_file() {
    let dir = temp_dir("positional");
    let file = dir.join("data.txt");
    write_bytes(&file, b"apple pie\ncherry\n");

    let out = sed(&["s/apple/banana/", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"banana pie\ncherry\n");
}

#[test]
fn positional_script_reads_stdin_when_no_file() {
    let out = sed(&["s/a/b/"], b"banana\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"bbnana\n");
}

#[test]
fn positional_script_with_multiple_files() {
    let dir = temp_dir("positional-multi");
    let a = dir.join("a.txt");
    let b = dir.join("b.txt");
    write_bytes(&a, b"cat\n");
    write_bytes(&b, b"cat\n");

    let out = sed(
        &["s/cat/dog/", a.to_str().unwrap(), b.to_str().unwrap()],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"dog\ndog\n");
}

#[test]
fn expression_flag_still_works_and_combines_in_order() {
    let dir = temp_dir("expr-compat");
    let file = dir.join("data.txt");
    write_bytes(&file, b"abc\n");

    let out = sed(
        &["-e", "s/a/x/", "-e", "s/b/y/", file.to_str().unwrap()],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"xyc\n");
}

#[test]
fn with_expression_flag_positionals_are_all_files() {
    let dir = temp_dir("expr-files");
    let file = dir.join("data.txt");
    write_bytes(&file, b"abc\n");

    let out = sed(&["-e", "s/a/x/", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"xbc\n");
}

// --- P1: -n quiet mode and the p command ---

#[test]
fn quiet_mode_with_line_address_print() {
    let dir = temp_dir("n-p");
    let file = dir.join("data.txt");
    write_bytes(&file, b"one\ntwo\nthree\n");

    let out = sed(&["-n", "2p", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"two\n");
}

#[test]
fn quiet_mode_with_substring_address_print() {
    let dir = temp_dir("n-substr-p");
    let file = dir.join("data.txt");
    write_bytes(&file, b"no match\nhas foo here\nalso foo again\nlast\n");

    let out = sed(&["-n", "/foo/p", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"has foo here\nalso foo again\n");
}

#[test]
fn without_quiet_mode_all_lines_are_echoed() {
    let dir = temp_dir("echo");
    let file = dir.join("data.txt");
    write_bytes(&file, b"one\ntwo\n");

    let out = sed(&["2p", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    // GNU prints every line automatically; p duplicates the addressed line.
    assert_eq!(out.stdout, b"one\ntwo\ntwo\n");
}

// --- P1: d command with line-number, substring and $ addresses ---

#[test]
fn delete_line_by_number() {
    let dir = temp_dir("3d");
    let file = dir.join("data.txt");
    write_bytes(&file, b"one\ntwo\nthree\nfour\n");

    let out = sed(&["3d", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"one\ntwo\nfour\n");
}

#[test]
fn delete_lines_matching_substring_address() {
    let dir = temp_dir("substr-d");
    let file = dir.join("data.txt");
    write_bytes(&file, b"keep\n-drop-\nkeep too\n");

    let out = sed(&["/-drop-/d", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"keep\nkeep too\n");
}

#[test]
fn delete_last_line_with_dollar_address() {
    let dir = temp_dir("dollar-d");
    let file = dir.join("data.txt");
    write_bytes(&file, b"one\ntwo\nlast\n");

    let out = sed(&["$d", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"one\ntwo\n");
}

#[test]
fn delete_range_to_last_line() {
    let dir = temp_dir("range-d");
    let file = dir.join("data.txt");
    write_bytes(&file, b"one\ntwo\nthree\nfour\n");

    let out = sed(&["2,$d", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"one\n");
}

// --- P1: addresses on s and ranges ---

#[test]
fn substitute_on_line_range_is_global() {
    let dir = temp_dir("range-s");
    let file = dir.join("data.txt");
    write_bytes(&file, b"a-a\nb-b\nb-b\nb-b\ne-e\n");

    let out = sed(&["2,4s/b/B/g", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"a-a\nB-B\nB-B\nB-B\ne-e\n");
}

#[test]
fn substitute_on_last_line_with_dollar_address() {
    let dir = temp_dir("dollar-s");
    let file = dir.join("data.txt");
    write_bytes(&file, b"end here\nend here too\n");

    let out = sed(&["$s/end/finish/", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"end here\nfinish here too\n");
}

// --- P1: q command (quit after printing the current line) ---

#[test]
fn quit_stops_processing_after_current_line() {
    let dir = temp_dir("q");
    let file = dir.join("data.txt");
    write_bytes(&file, b"one\ntwo\nthree\nfour\n");

    let out = sed(&["2q", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"one\ntwo\n");
}

// --- P1: multiple commands via semicolon inside one script ---

#[test]
fn semicolon_separated_commands() {
    let dir = temp_dir("semicolon");
    let file = dir.join("data.txt");
    write_bytes(&file, b"one\ntwo\nthree\nfour\nfive\n");

    let out = sed(&["2d;4d", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"one\nthree\nfive\n");
}

#[test]
fn semicolon_after_substitute_keeps_pattern_semicolons() {
    let dir = temp_dir("semicolon-s");
    let file = dir.join("data.txt");
    write_bytes(&file, b"a;b\nend\n");

    let out = sed(&["s/a;b/x/;s/end/done/", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"x\ndone\n");
}

// --- P1: unknown commands must error instead of being silently ignored ---

#[test]
fn unknown_command_errors_with_exit_code() {
    let dir = temp_dir("unknown-cmd");
    let file = dir.join("data.txt");
    write_bytes(&file, b"one\n");

    let out = sed(&["y/abc/xyz/", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(out.stdout, b"");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unknown command"), "stderr was: {}", stderr);
}

#[test]
fn missing_script_errors() {
    let out = sed(&[], b"data\n");
    assert_eq!(out.status.code(), Some(1));
    assert!(!out.stderr.is_empty());
}

#[test]
fn unterminated_address_errors() {
    let dir = temp_dir("unterm-addr");
    let file = dir.join("data.txt");
    write_bytes(&file, b"one\n");

    let out = sed(&["/foo", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(out.stdout, b"");
}

// --- P1: unreadable file reports an error with exit code 1 and keeps going ---

#[test]
fn unreadable_file_errors_but_remaining_files_process() {
    let dir = temp_dir("bad-file");
    let good = dir.join("good.txt");
    write_bytes(&good, b"cat\n");
    let missing = dir.join("missing.txt");

    let out = sed(
        &[
            "s/cat/dog/",
            missing.to_str().unwrap(),
            good.to_str().unwrap(),
        ],
        b"",
    );
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(out.stdout, b"dog\n");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("can't read"), "stderr was: {}", stderr);
}

// --- P1: -i edits in place via temp file + atomic rename ---

#[test]
fn in_place_replaces_content_changes_mtime_and_leaves_no_temp_files() {
    let dir = temp_dir("in-place");
    let file = dir.join("data.txt");
    write_bytes(&file, b"one\ntwo\nthree\n");

    // Pin an old mtime so the rewrite is provably a fresh file.
    let old = SystemTime::now() - Duration::from_secs(24 * 3600);
    std::fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_modified(old)
        .unwrap();

    let out = sed(&["-i", "s/two/TWO/", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(std::fs::read(&file).unwrap(), b"one\nTWO\nthree\n");

    let mtime = std::fs::metadata(&file).unwrap().modified().unwrap();
    assert!(mtime > old, "in-place edit must produce a new mtime");

    // Temp file must be cleaned up: the directory holds only the edited file.
    assert_eq!(dir_entries(&dir), vec!["data.txt".to_string()]);
}

#[test]
fn in_place_with_backup_suffix_keeps_original_copy() {
    let dir = temp_dir("in-place-bak");
    let file = dir.join("data.txt");
    write_bytes(&file, b"one\ntwo\n");

    // Attached GNU form: -i.bak (a bare `-i .bak` is not a suffix in GNU either).
    let out = sed(&["-i.bak", "s/one/ONE/", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(std::fs::read(&file).unwrap(), b"ONE\ntwo\n");
    assert_eq!(
        std::fs::read(dir.join("data.txt.bak")).unwrap(),
        b"one\ntwo\n"
    );
    assert_eq!(
        dir_entries(&dir),
        vec!["data.txt".to_string(), "data.txt.bak".to_string()]
    );
}

#[test]
fn in_place_long_form_with_equals_suffix() {
    let dir = temp_dir("in-place-long");
    let file = dir.join("data.txt");
    write_bytes(&file, b"one\n");

    let out = sed(
        &["--in-place=.orig", "s/one/1/", file.to_str().unwrap()],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(std::fs::read(&file).unwrap(), b"1\n");
    assert_eq!(std::fs::read(dir.join("data.txt.orig")).unwrap(), b"one\n");
}

// --- P1: CRLF and missing trailing newline handling ---

#[test]
fn in_place_preserves_crlf_line_endings() {
    let dir = temp_dir("crlf");
    let file = dir.join("data.txt");
    write_bytes(&file, b"alpha\r\nbeta\r\n");

    let out = sed(&["-i", "s/alpha/ALPHA/", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(std::fs::read(&file).unwrap(), b"ALPHA\r\nbeta\r\n");
}

#[test]
fn stdout_preserves_missing_trailing_newline_like_gnu() {
    // Round 13 difftest: GNU sed preserves the input's missing trailing
    // newline; the earlier "always ends with newline" assumption was wrong.
    let dir = temp_dir("tail-nl");
    let file = dir.join("data.txt");
    write_bytes(&file, b"one\ntwo"); // no trailing newline

    let out = sed(&["s/two/TWO/", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"one\nTWO");
}

// --- P1: non-UTF-8 lines are matched lossily and passed through byte-exact ---

#[test]
fn non_utf8_line_substitute_preserves_raw_bytes() {
    let dir = temp_dir("non-utf8");
    let file = dir.join("bin.txt");
    // "caf\xFFe foo\nplain\n" — 0xFF makes the first line invalid UTF-8.
    write_bytes(&file, b"caf\xFFE foo\nplain\n");

    let out = sed(&["s/foo/bar/", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"caf\xFFE bar\nplain\n");
}

#[test]
fn non_utf8_line_deleted_by_substring_address() {
    let dir = temp_dir("non-utf8-d");
    let file = dir.join("bin.txt");
    write_bytes(&file, b"a\xFFb drop\nkeep\n");

    let out = sed(&["/drop/d", file.to_str().unwrap()], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"keep\n");
}

#[test]
fn non_utf8_stdin_passthrough() {
    let out = sed(&["s/a/b/"], b"x\xFFa\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"x\xFFb\n");
}
