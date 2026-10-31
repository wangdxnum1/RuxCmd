use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::process::Output;
use tempfile::TempDir;

fn awk(args: &[&str], stdin: &[u8]) -> Output {
    cargo_bin_cmd!("awk")
        .args(args)
        .write_stdin(stdin)
        .output()
        .unwrap()
}

// --- user functions -------------------------------------------------------

#[test]
fn function_definition_call_and_return() {
    let out = awk(
        &["function add(a, b) { return a + b } BEGIN { print add(2, 3) }"],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "5\n");
}

#[test]
fn recursion_works() {
    let out = awk(
        &[
            "function fact(n) { if (n <= 1) return 1; return n * fact(n - 1) } BEGIN { print fact(6) }",
        ],
        b"",
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "720\n");
}

#[test]
fn parameters_are_dynamic_locals() {
    // The caller's variable keeps its value after the call.
    let out = awk(
        &["function twice(x) { x = x * 2; return x } BEGIN { v = 10; print twice(v), v }"],
        b"",
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "20 10\n");
}

#[test]
fn missing_arguments_default_to_empty() {
    let out = awk(
        &["function f(a, b) { return sprintf(\"%s-%s\", a, b) } BEGIN { print f(\"only\") }"],
        b"",
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "only-\n");
}

// --- output redirection ----------------------------------------------------

#[test]
fn print_to_file_truncate_and_append() {
    let dir = TempDir::new().unwrap();
    let out_file = dir.path().join("out.txt");
    let out_str = out_file.to_string_lossy().replace('\\', "/");

    let prog = format!(
        "BEGIN {{ print \"r1\" > \"{}\"; print \"r2\" > \"{}\"; print \"app\" >> \"{}\" }}",
        out_str, out_str, out_str
    );
    let out = awk(&[&prog], b"");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        fs::read(&out_file).unwrap(),
        b"r1\nr2\napp\n",
        "truncate then append accumulate in one open handle"
    );
}

#[test]
fn pipe_to_command() {
    // Pipe through the Windows sort filter; output arrives on our stdout.
    let out = awk(
        &["BEGIN { print \"b\" | \"sort\"; print \"a\" | \"sort\"; close(\"sort\") }"],
        b"",
    );
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains('a') && text.contains('b'), "stdout: {text}");
}

#[test]
fn print_to_dev_stderr() {
    let out = awk(&["BEGIN { print \"E\" > \"/dev/stderr\" }"], b"");
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stderr).contains('E'));
}

#[test]
fn close_builtin_returns_status() {
    // close() on an unopened name returns -1.
    let out = awk(&["BEGIN { print close(\"nowhere\") }"], b"");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "-1\n");
}

#[test]
fn printf_supports_redirection() {
    let dir = TempDir::new().unwrap();
    let out_file = dir.path().join("fmt.txt");
    let prog = format!(
        "BEGIN {{ printf \"%s=%d\\n\", \"k\", 7 > \"{}\" }}",
        out_file.to_string_lossy().replace('\\', "/")
    );
    let out = awk(&[&prog], b"");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(fs::read(&out_file).unwrap(), b"k=7\n");
}
