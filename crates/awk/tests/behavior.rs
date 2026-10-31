use assert_cmd::cargo::cargo_bin_cmd;
use std::process::Output;

fn awk(args: &[&str], stdin: &[u8]) -> Output {
    cargo_bin_cmd!("awk")
        .args(args)
        .write_stdin(stdin)
        .output()
        .unwrap()
}

#[test]
fn regex_patterns_use_ere() {
    // ERE features: + repetition, character classes, alternation.
    let out = awk(&["/[0-9]+/ { print \"num\" }"], b"abc\nx42y\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "num\n");

    let out = awk(&["/cat|dog/ { print $0 }"], b"bird\ncat\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "cat\n");
}

#[test]
fn invalid_ere_fails_at_parse() {
    let out = awk(&["/x+/)(/ { print }"], b"a\n");
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("invalid regular expression"), "stderr: {err}");
}

#[test]
fn printf_statement_supports_gnu_specifiers() {
    let out = awk(&["{ printf \"%5d|%-6s|\\n\", 42, $1 }"], b"ab\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "   42|ab    |\n");

    let out = awk(&["{ printf \"%05.1f\\n\", 3.14159 }"], b"x\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "003.1\n");

    let out = awk(&["{ printf \"100%%\\n\" }"], b"x\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "100%\n");
}

#[test]
fn length_builtin() {
    let out = awk(&["{ print length($0), length }"], b"hello\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "5 5\n");
}

#[test]
fn substr_clips_before_start_like_gawk() {
    // gawk: substr("abc", 0, 2) → "ab" (position clipped to 1).
    let out = awk(&["{ print substr(\"abc\", 0, 2) }"], b"x\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "ab\n");

    let out = awk(&["{ print substr($0, 5) }"], b"foo bar\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "bar\n");
}

#[test]
fn index_builtin_is_one_based() {
    let out = awk(
        &["{ print index(\"hello\", \"l\"), index(\"hello\", \"z\") }"],
        b"x\n",
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3 0\n");
}

#[test]
fn sprintf_builtin_formats_into_expressions() {
    let out = awk(&["{ print sprintf(\"%05.1f\", 3.14159) }"], b"x\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "003.1\n");
}

#[test]
fn case_mapping_builtins() {
    let out = awk(&["{ print toupper($0), tolower($0) }"], b"HeLLo\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "HELLO hello\n");
}

#[test]
fn sub_replaces_first_and_resplits_fields() {
    let out = awk(&["{ sub(/b/, \"B\"); print $0, $2 }"], b"a b c\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a B c B\n");
}

#[test]
fn gsub_replaces_all_with_ampersand() {
    let out = awk(&["{ gsub(/[0-9]+/, \"<&>\"); print }"], b"a1b22c\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "a<1>b<22>c\n");

    // gsub returns the number of substitutions.
    let out = awk(&["{ print gsub(/o/, \"0\") }"], b"foo boo\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "4\n");
}

#[test]
fn sub_targets_a_variable_lvalue() {
    let out = awk(&["{ x = \"a1a2\"; sub(/a/, \"X\", x); print x }"], b"x\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "X1a2\n");
}

#[test]
fn adjacent_values_concatenate() {
    let out = awk(&["BEGIN { print \"hello\" \"world\" }"], b"");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "helloworld\n");
}

#[test]
fn three_values_concatenate() {
    let out = awk(&["BEGIN { print \"a\" \"b\" \"c\" }"], b"");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "abc\n");
}

#[test]
fn field_and_literal_concatenate() {
    let out = awk(&["{ print $1 \"-\" $2 }"], b"hello world\n");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "hello-world\n");
}

#[test]
fn concat_binds_looser_than_addition() {
    // "x" 1 + 2 → "x" . (1 + 2) = "x3" (concat looser than +)
    let out = awk(&["BEGIN { print \"x\" 1 + 2 }"], b"");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "x3\n");
}

#[test]
fn concat_binds_tighter_than_comparison() {
    // "a" "b" == "ab" → ("a" . "b") == "ab" → 1 (concat tighter than ==)
    let out = awk(&["BEGIN { print \"a\" \"b\" == \"ab\" }"], b"");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}

#[test]
fn numeric_concat_produces_string() {
    let out = awk(&["BEGIN { print 1 2 }"], b"");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "12\n");
}

#[test]
fn while_loop_executes() {
    let out = awk(&["BEGIN { i = 0; while (i < 3) { print i; i++ } }"], b"");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0\n1\n2\n");
}

#[test]
fn do_while_loop_executes() {
    let out = awk(&["BEGIN { i = 0; do { print i; i++ } while (i < 3) }"], b"");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0\n1\n2\n");
}

#[test]
fn do_while_executes_at_least_once() {
    // do-while runs the body even when the condition is false initially.
    let out = awk(&["BEGIN { i = 10; do { print i } while (i < 3) }"], b"");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "10\n");
}
