use assert_cmd::cargo::cargo_bin_cmd;

fn run(args: &[&str]) -> (String, Option<i32>) {
    let out = cargo_bin_cmd!("expr").args(args).output().unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        out.status.code(),
    )
}

#[test]
fn arithmetic_and_precedence() {
    let (out, code) = run(&["1", "+", "2"]);
    assert_eq!((out.as_str(), code), ("3\n", Some(0)));

    let (out, code) = run(&["2", "+", "3", "*", "4"]);
    assert_eq!((out.as_str(), code), ("14\n", Some(0)));

    let (out, code) = run(&["(", "1", "+", "2", ")", "*", "3"]);
    assert_eq!((out.as_str(), code), ("9\n", Some(0)));

    let (out, code) = run(&["7", "/", "2"]);
    assert_eq!((out.as_str(), code), ("3\n", Some(0)));

    let (out, code) = run(&["7", "%", "2"]);
    assert_eq!((out.as_str(), code), ("1\n", Some(0)));
}

#[test]
fn zero_result_exits_one() {
    let (out, code) = run(&["0"]);
    assert_eq!((out.as_str(), code), ("0\n", Some(1)));
}

#[test]
fn comparisons_follow_numeric_then_string_semantics() {
    let (out, code) = run(&["1", "=", "1"]);
    assert_eq!((out.as_str(), code), ("1\n", Some(0)));

    let (out, code) = run(&["1", "=", "2"]);
    assert_eq!((out.as_str(), code), ("0\n", Some(1)));

    let (out, code) = run(&["a", "<", "b"]);
    assert_eq!((out.as_str(), code), ("1\n", Some(0)));

    let (out, code) = run(&["b", "<", "a"]);
    assert_eq!((out.as_str(), code), ("0\n", Some(1)));
}

#[test]
fn and_returns_left_value_like_gnu() {
    // GNU: 2 & 3 yields the left operand; 0 & 3 yields 0 (exit 1).
    let (out, code) = run(&["2", "&", "3"]);
    assert_eq!((out.as_str(), code), ("2\n", Some(0)));

    let (out, code) = run(&["0", "&", "3"]);
    assert_eq!((out.as_str(), code), ("0\n", Some(1)));
}

#[test]
fn or_prefers_valid_left_side() {
    let (out, code) = run(&["1", "|", "0"]);
    assert_eq!((out.as_str(), code), ("1\n", Some(0)));

    let (out, code) = run(&["", "|", ""]);
    assert_eq!((out.as_str(), code), ("0\n", Some(1)));
}

#[test]
fn string_functions() {
    let (out, code) = run(&["length", "hello"]);
    assert_eq!((out.as_str(), code), ("5\n", Some(0)));

    let (out, code) = run(&["substr", "hello", "2", "3"]);
    assert_eq!((out.as_str(), code), ("ell\n", Some(0)));

    let (out, code) = run(&["index", "hello", "l"]);
    assert_eq!((out.as_str(), code), ("3\n", Some(0)));

    let (out, code) = run(&["index", "hello", "z"]);
    assert_eq!((out.as_str(), code), ("0\n", Some(1)));
}

#[test]
fn anchored_bre_match_returns_group_or_length() {
    let (out, code) = run(&["abc", ":", "a\\(.*\\)"]);
    assert_eq!((out.as_str(), code), ("bc\n", Some(0)));

    let (out, code) = run(&["abc", ":", "a"]);
    // No group: GNU returns the length of the match.
    assert_eq!((out.as_str(), code), ("1\n", Some(0)));

    let (out, code) = run(&["abc", ":", "x"]);
    assert_eq!(code, Some(1));
    assert_eq!(out, "\n");
}

#[test]
fn back_reference_matches_repeated_group() {
    // GNU 8.32 ground truth: a back reference `\N` re-matches what group N
    // captured; the result is still the first group's content.
    let (out, code) = run(&["abab", ":", "\\(ab\\)\\1"]);
    assert_eq!((out.as_str(), code), ("ab\n", Some(0)));

    let (out, code) = run(&["abcbc", ":", "a\\(b\\)c\\1"]);
    assert_eq!((out.as_str(), code), ("b\n", Some(0)));

    // Nested groups keep POSIX numbering: GNU `expr aXbXb : '\(a\(X\)b\)\2'`.
    let (out, code) = run(&["aXbXb", ":", "\\(a\\(X\\)b\\)\\2"]);
    assert_eq!((out.as_str(), code), ("aXb\n", Some(0)));

    let (out, code) = run(&["aXaX", ":", "\\(a\\)\\(X\\)\\1\\2"]);
    assert_eq!((out.as_str(), code), ("a\n", Some(0)));

    // The `match` keyword shares the back reference support.
    let (out, code) = run(&["match", "abab", "\\(ab\\)\\1"]);
    assert_eq!((out.as_str(), code), ("ab\n", Some(0)));
}

#[test]
fn back_reference_without_match_returns_empty_one() {
    // GNU: `expr abcabc : 'a\(b\)c\1'` needs "abcb" at the head of the
    // subject, which fails -> empty output, exit 1.
    let (out, code) = run(&["abcabc", ":", "a\\(b\\)c\\1"]);
    assert_eq!((out.as_str(), code), ("\n", Some(1)));
}

#[test]
fn invalid_back_reference_exits_two() {
    // GNU: "Invalid back reference" (exit 2) whenever the Nth group is not
    // closed before the reference: forward refs, out-of-range refs and refs
    // inside the referenced group.
    for pattern in ["\\1", "\\(a\\)\\2", "\\(a\\1\\)"] {
        let out = cargo_bin_cmd!("expr")
            .args(["aa", ":", pattern])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "pattern {pattern}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("back reference"), "stderr: {err}");
    }
}

#[test]
fn division_by_zero_exits_two() {
    let out = cargo_bin_cmd!("expr")
        .args(["1", "/", "0"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("division by zero"));
}

#[test]
fn non_integer_operand_exits_two() {
    let out = cargo_bin_cmd!("expr")
        .args(["abc", "+", "1"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("non-integer argument"));
}

#[test]
fn trailing_tokens_are_a_syntax_error() {
    let out = cargo_bin_cmd!("expr")
        .args(["length", "abc", "def"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("unexpected argument 'def'"), "stderr: {err}");
}
