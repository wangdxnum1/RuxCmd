use assert_cmd::cargo::cargo_bin_cmd;
#[test]
fn matching_lines_and_exit_codes() {
    cargo_bin_cmd!("grep")
        .arg("needle")
        .write_stdin("hay\nneedle\n")
        .assert()
        .success()
        .stdout("needle\n");
    cargo_bin_cmd!("grep")
        .arg("missing")
        .write_stdin("hay\nneedle\n")
        .assert()
        .code(1)
        .stdout("");
}

#[test]
fn count_on_single_file_prints_number() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    std::fs::write(&path, "apple\nbanana\napple pie\ncherry\n").unwrap();

    cargo_bin_cmd!("grep")
        .arg("-c")
        .arg("apple")
        .arg(&path)
        .assert()
        .code(0)
        .stdout("2\n");

    cargo_bin_cmd!("grep")
        .arg("-c")
        .arg("pear")
        .arg(&path)
        .assert()
        .code(1)
        .stdout("0\n");
}

fn grep_stdout_bytes(args: &[&str], file: &std::path::Path) -> (Vec<u8>, Option<i32>) {
    let mut cmd = cargo_bin_cmd!("grep");
    for arg in args {
        cmd.arg(arg);
    }
    cmd.arg(file);
    let output = cmd.assert().get_output().clone();
    let code = output.status.code();
    (output.stdout.clone(), code)
}

// --- P1: non-UTF-8 input must not be silently truncated (Round 12 D1) ---

#[test]
fn non_utf8_file_match_preserves_raw_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bin.txt");
    // "caf\xFFe foo\nplain\n" — the 0xFF byte makes the line invalid UTF-8.
    let data = b"caf\xFFE foo\nplain\n";
    std::fs::write(&path, data).unwrap();

    let (stdout, code) = grep_stdout_bytes(&["foo"], &path);
    assert_eq!(code, Some(0));
    // The whole raw line, 0xFF byte included, is echoed back.
    assert_eq!(stdout, b"caf\xFFE foo\n");
}

#[test]
fn non_utf8_stdin_match_preserves_raw_bytes() {
    let data = b"a\xFFb target\nclean\n";
    let mut cmd = cargo_bin_cmd!("grep");
    cmd.arg("target").write_stdin(data);
    let output = cmd.assert().code(0).get_output().clone();
    assert_eq!(output.stdout.clone(), b"a\xFFb target\n");
}

// --- P1: single-file output must not carry a `file:` prefix ---

#[test]
fn single_file_output_has_no_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    std::fs::write(&path, "alpha\nbeta\nalpha again\n").unwrap();

    cargo_bin_cmd!("grep")
        .arg("alpha")
        .arg(&path)
        .assert()
        .code(0)
        .stdout("alpha\nalpha again\n");
}

#[test]
fn multi_file_output_keeps_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.txt");
    let b = dir.path().join("b.txt");
    std::fs::write(&a, "hit\n").unwrap();
    std::fs::write(&b, "miss\n").unwrap();

    let expected = format!("{}:hit\n", a.display());
    cargo_bin_cmd!("grep")
        .arg("hit")
        .arg(&a)
        .arg(&b)
        .assert()
        .code(0)
        .stdout(expected);
}

#[test]
fn filename_prefix_flags() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.txt");
    let b = dir.path().join("b.txt");
    std::fs::write(&a, "hit\n").unwrap();
    std::fs::write(&b, "hit\n").unwrap();

    // -h suppresses the prefix even with multiple files.
    cargo_bin_cmd!("grep")
        .arg("-h")
        .arg("hit")
        .arg(&a)
        .arg(&b)
        .assert()
        .code(0)
        .stdout("hit\nhit\n");

    // -H forces the prefix even with a single file.
    let expected = format!("{}:hit\n", a.display());
    cargo_bin_cmd!("grep")
        .arg("-H")
        .arg("hit")
        .arg(&a)
        .assert()
        .code(0)
        .stdout(expected);
}

// --- P1: -F fixed-string matching (regex metacharacters taken literally) ---

#[test]
fn fixed_strings_match_literals() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    std::fs::write(&path, "a.c\nabc\nxa.cy\n").unwrap();

    cargo_bin_cmd!("grep")
        .arg("-F")
        .arg("a.c")
        .arg(&path)
        .assert()
        .code(0)
        .stdout("a.c\nxa.cy\n");
}

// --- P1: -w whole-word matching ---

#[test]
fn word_regexp_matches_whole_words() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    std::fs::write(&path, "word\nwords\na word here\nsword\n").unwrap();

    cargo_bin_cmd!("grep")
        .arg("-w")
        .arg("word")
        .arg(&path)
        .assert()
        .code(0)
        .stdout("word\na word here\n");
}

// --- P1: -l lists file names only ---

#[test]
fn files_with_matches_lists_names_only() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.txt");
    let b = dir.path().join("b.txt");
    std::fs::write(&a, "hit one\nhit two\n").unwrap();
    std::fs::write(&b, "miss\n").unwrap();

    let expected = format!("{}\n", a.display());
    cargo_bin_cmd!("grep")
        .arg("-l")
        .arg("hit")
        .arg(&a)
        .arg(&b)
        .assert()
        .code(0)
        .stdout(expected);
}

// --- P1: -A/-B/-C context lines ---

#[test]
fn context_flag_c_prints_surrounding_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    std::fs::write(&path, "one\ntwo\nthree\nfour\nfive\n").unwrap();

    cargo_bin_cmd!("grep")
        .arg("-C1")
        .arg("three")
        .arg(&path)
        .assert()
        .code(0)
        .stdout("two\nthree\nfour\n");
}

#[test]
fn before_and_after_context_flags() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    std::fs::write(&path, "one\ntwo\nthree\nfour\nfive\nsix\n").unwrap();

    cargo_bin_cmd!("grep")
        .arg("-B1")
        .arg("-A1")
        .arg("three")
        .arg(&path)
        .assert()
        .code(0)
        .stdout("two\nthree\nfour\n");
}

#[test]
fn line_number_format_matches_gnu_layout() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    std::fs::write(&path, "one\ntwo\nthree\nfour\nfive\n").unwrap();

    // Match lines use `N:`, context lines use `N-`.
    cargo_bin_cmd!("grep")
        .arg("-n")
        .arg("three")
        .arg(&path)
        .assert()
        .code(0)
        .stdout("3:three\n");

    cargo_bin_cmd!("grep")
        .arg("-n")
        .arg("-C1")
        .arg("three")
        .arg(&path)
        .assert()
        .code(0)
        .stdout("2-two\n3:three\n4-four\n");
}

// --- P1: -E is accepted (the regex engine is already ERE-like) ---

#[test]
fn extended_regexp_flag_is_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.txt");
    std::fs::write(&path, "foo123\nbar\n").unwrap();

    cargo_bin_cmd!("grep")
        .arg("-E")
        .arg("foo[0-9]+")
        .arg(&path)
        .assert()
        .code(0)
        .stdout("foo123\n");
}

// --- P2: fnmatch character classes in --include/--exclude globs ---
// Expectations verified against GNU grep 3.0 (fnmatch(3) semantics).

/// Run a recursive grep over `dir` and return the -l style stdout.
fn grep_recursive_stdout(args: &[&str], pattern: &str, dir: &std::path::Path) -> String {
    let output = cargo_bin_cmd!("grep")
        .args(args)
        .arg(pattern)
        .arg(dir)
        .assert()
        .get_output()
        .clone();
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn include_glob_set_matches_c_and_h_files() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["main.c", "main.h", "main.rs", "readme.md"] {
        std::fs::write(dir.path().join(name), "hello\n").unwrap();
    }

    let out = grep_recursive_stdout(&["-r", "-l", "--include=*.[ch]"], "hello", dir.path());
    assert!(out.contains("main.c"), "main.c missing: {out}");
    assert!(out.contains("main.h"), "main.h missing: {out}");
    assert!(!out.contains("main.rs"), "main.rs leaked: {out}");
    assert!(!out.contains("readme.md"), "readme.md leaked: {out}");
}

#[test]
fn include_glob_leading_set_matches_s_or_t_files() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["test.txt", "start.log", "main.c", "readme.md"] {
        std::fs::write(dir.path().join(name), "hello\n").unwrap();
    }

    let out = grep_recursive_stdout(&["-r", "-l", "--include=[st]*"], "hello", dir.path());
    assert!(out.contains("test.txt"), "test.txt missing: {out}");
    assert!(out.contains("start.log"), "start.log missing: {out}");
    assert!(!out.contains("main.c"), "main.c leaked: {out}");
    assert!(!out.contains("readme.md"), "readme.md leaked: {out}");
}

#[test]
fn exclude_glob_negated_set_keeps_files_starting_with_a() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["apple.txt", "banana.txt", "main.c"] {
        std::fs::write(dir.path().join(name), "hello\n").unwrap();
    }

    let out = grep_recursive_stdout(&["-r", "-l", "--exclude=[!a]*"], "hello", dir.path());
    assert!(out.contains("apple.txt"), "apple.txt missing: {out}");
    assert!(!out.contains("banana.txt"), "banana.txt leaked: {out}");
    assert!(!out.contains("main.c"), "main.c leaked: {out}");
}

#[test]
fn include_glob_range_and_plain_metacharacters() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["apple.txt", "banana.txt", "cherry.log", "main.c"] {
        std::fs::write(dir.path().join(name), "hello\n").unwrap();
    }

    // [a-c] is a range; the dot in *.log stays a literal dot (not regex-any).
    let out = grep_recursive_stdout(&["-r", "-l", "--include=[a-c]*"], "hello", dir.path());
    assert!(out.contains("apple.txt"), "apple.txt missing: {out}");
    assert!(out.contains("banana.txt"), "banana.txt missing: {out}");
    assert!(out.contains("cherry.log"), "cherry.log missing: {out}");
    assert!(!out.contains("main.c"), "main.c leaked: {out}");

    // A literal dot must not act as `.`: "mainXc" does not match "main.c".
    std::fs::write(dir.path().join("mainXc"), "hello\n").unwrap();
    let out = grep_recursive_stdout(&["-r", "-l", "--include=main.c"], "hello", dir.path());
    assert!(out.contains("main.c"), "main.c missing: {out}");
    assert!(
        !out.contains("mainXc"),
        "mainXc leaked through unescaped dot: {out}"
    );
}
