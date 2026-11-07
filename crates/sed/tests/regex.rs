use assert_cmd::cargo::cargo_bin_cmd;

fn run(script: &str, stdin: &[u8]) -> (String, Option<i32>) {
    run_args(&[script], stdin)
}

fn run_args(args: &[&str], stdin: &[u8]) -> (String, Option<i32>) {
    let out = cargo_bin_cmd!("sed")
        .args(args)
        .write_stdin(stdin)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        out.status.code(),
    )
}

#[test]
fn basic_regex_with_dot_and_star() {
    // GNU: `sed 's/f.*o/XX/'` matches the whole run.
    let (out, code) = run("s/f.*o/XX/", b"fabco\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "XX\n");
}

#[test]
fn group_capture_in_replacement() {
    let (out, code) = run("s/\\(ab\\)\\(c\\)/\\2\\1/", b"abcd\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "cabd\n");
}

#[test]
fn ampersand_inserts_whole_match() {
    let (out, code) = run("s/[0-9][0-9]*/<&>/", b"a42b\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "a<42>b\n");
}

#[test]
fn escaped_ampersand_is_literal() {
    let (out, code) = run("s/42/\\&/", b"a42b\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "a&b\n");
}

#[test]
fn global_flag_replaces_every_match() {
    let (out, code) = run("s/o/0/g", b"foo boo\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "f00 b00\n");
}

#[test]
fn nth_occurrence_flag() {
    // GNU: `s/o/0/2` replaces only the second occurrence.
    let (out, code) = run("s/o/0/2", b"foo\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "fo0\n");
}

#[test]
fn print_flag_echoes_after_change() {
    // -n plus s///p prints only lines that changed; banana contains no `b`
    // uppercase-able... it does: b->B changes banana, so use a marker that
    // only matches the first line.
    let out = cargo_bin_cmd!("sed")
        .args(["-n", "s/^a/A/p"])
        .write_stdin(b"apple\nbanana\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "Apple\n");
}

#[test]
fn case_insensitive_flag() {
    let (out, code) = run("s/hello/bye/i", b"HELLO world\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "bye world\n");
}

#[test]
fn interval_expression() {
    let (out, code) = run("s/x\\{2,3\\}/Y/g", b"x xx xxx\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "x Y Y\n");
}

#[test]
fn regex_address_selects_lines() {
    let (out, code) = run("/^b/d", b"apple\nbanana\ncherry\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "apple\ncherry\n");
}

#[test]
fn invalid_bre_fails_at_startup() {
    cargo_bin_cmd!("sed")
        .arg("s/a\\(b/X/")
        .write_stdin(b"a\n")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("sed: -e expression #1"));
}

#[test]
fn back_reference_matches_repeated_group() {
    // GNU: `\(a\)\1` requires two equal letters; the first "aa" of "aaa"
    // becomes X and the unmatched tail survives.
    let (out, code) = run("s/\\(a\\)\\1/X/", b"aaa\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "Xa\n");
}

#[test]
fn back_reference_chain_matches_repetition() {
    // GNU: `\(b\)\(c\)\1` matches "bcb"; \1\2\1 rebuilds it, so the line is
    // unchanged, while a distinguishing replacement shows both captures.
    let (out, code) = run("s/\\(b\\)\\(c\\)\\1/\\1\\2\\1/", b"abcbcb\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "abcbcb\n");

    let (out, code) = run("s/\\(b\\)\\(c\\)\\1/<\\1\\2>/", b"abcbcb\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "a<bc>cb\n");
}

#[test]
fn back_reference_across_words() {
    // GNU: the pattern requires "hello world hello"; \1 re-inserts the
    // first group, \2 the second.
    let (out, code) = run("s/\\(hello\\) \\(world\\) \\1/\\1/", b"hello world hello\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "hello\n");

    let (out, code) = run("s/\\(hello\\) \\(world\\) \\1/\\2/", b"hello world hello\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "world\n");
}

#[test]
fn back_reference_in_address() {
    // Addresses take back references like the `s` command, in both dialects.
    let (out, code) = run("/\\(abc\\)\\1/d", b"abcabc\nxyz\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "xyz\n");

    let (out, code) = run_args(&["-E", r"/(abc)\1/d"], b"abcabc\nxyz\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "xyz\n");
}

#[test]
fn invalid_back_reference_fails_at_startup() {
    // GNU: "Invalid back reference" for forward refs and out-of-range refs,
    // in BRE and in -E ERE (GNU extension back references).
    for args in [
        vec!["s/\\(a\\)\\2/X/"],
        vec!["s/\\1(a)/X/"],
        vec!["-E", r"s/(a)\2/X/"],
        vec!["-E", r"s/\1(a)/X/"],
    ] {
        cargo_bin_cmd!("sed")
            .args(&args)
            .write_stdin(b"aa\n")
            .assert()
            .failure()
            .code(1)
            .stderr(predicates::str::contains("Invalid back reference"));
    }
}

#[test]
fn non_utf8_bytes_survive_substitution() {
    let out = cargo_bin_cmd!("sed")
        .arg("s/b/X/")
        .write_stdin([0x61, 0xFF, 0x62, 0x63, b'\n'])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, vec![0x61, 0xFF, b'X', b'c', b'\n']);
}

#[test]
fn replacement_newline_escape() {
    // GNU: `\n` in a replacement emits a real newline.
    let (out, code) = run("s/,/\\n/", b"a,b\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "a\nb\n");
}

#[test]
fn escaped_delimiter_in_pattern() {
    // `\/` inside an address is a literal slash.
    let (out, code) = run("/a\\/b/d", b"x\na/b\ny\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "x\ny\n");
}

#[test]
fn ere_mode_alternation_and_intervals() {
    // -E: unescaped ERE operators keep their operator meaning.
    let (out, code) = run_args(&["-E", "s/cat|dog/PET/g"], b"cat bird dog\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "PET bird PET\n");

    let (out, code) = run_args(&["-E", "s/o{2}/OO/"], b"foot\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "fOOt\n");
}

#[test]
fn ere_mode_groups_are_unescaped() {
    // In ERE, ( ) are groups without backslashes; \1 in the replacement
    // inserts the first group.
    let out = cargo_bin_cmd!("sed")
        .args(["-E", r"s/(f[o]+t)/<\1>/g"])
        .write_stdin(b"foot bar boot\n")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "<foot> bar boot\n");
}

#[test]
fn ere_mode_back_references_match() {
    // GNU extension: back references work in -E ERE too, in patterns and
    // in addresses. The whole line "aa" is the match, so it becomes X.
    let (out, code) = run_args(&["-E", r"s/(a)\1/X/"], b"aa\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "X\n");

    let (out, code) = run_args(&["-E", r"s/(abc)\1/X/"], b"abcabc\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "X\n");

    // A reference to a nested group that is already closed is valid.
    let (out, code) = run_args(&["-E", r"s/(a(X)b)\2/<\1>/"], b"aXbXb\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "<aXb>b\n");
}

#[test]
fn ere_mode_r_alias_and_bre_unchanged() {
    // -r is the historical alias.
    let (out, code) = run_args(&["-r", "s/o{2}/OO/"], b"foot\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "fOOt\n");

    // BRE stays literal: + is a plain plus sign without -E.
    let (out, code) = run("s/o+/0/", b"foot\n");
    assert_eq!(code, Some(0));
    assert_eq!(out, "foot\n");
}
