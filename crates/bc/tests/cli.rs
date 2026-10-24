use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str;
use std::fs;

fn run_bc(input: &str) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("bc").write_stdin(input).assert()
}

// P0 #31 regression: expressions containing operators used to tokenize on
// whitespace only, so `1+2` stayed a single token and always printed 0.
#[test]
fn addition_without_spaces() {
    run_bc("1+2\n").success().stdout("3\n");
}

#[test]
fn multiplication_binds_tighter_than_addition() {
    run_bc("2+3*4\n").success().stdout("14\n");
}

#[test]
fn parentheses_override_precedence() {
    run_bc("(1+2)*3\n").success().stdout("9\n");
}

#[test]
fn division_truncates_toward_zero() {
    // GNU bc default scale=0: integer division truncating toward zero.
    run_bc("10/3\n").success().stdout("3\n");
}

#[test]
fn unary_minus_on_leading_operand() {
    run_bc("-5+3\n").success().stdout("-2\n");
}

#[test]
fn exponentiation() {
    run_bc("2^10\n").success().stdout("1024\n");
}

#[test]
fn modulo_keeps_sign_of_dividend() {
    // bc % at scale 0 equals a - (a/b)*b, so the sign follows the dividend,
    // which matches Rust's integer remainder.
    run_bc("10%3\n-7%3\n").success().stdout("1\n-1\n");
}

#[test]
fn assignment_is_silent_and_visible_on_next_line() {
    run_bc("a=5\na*2\n").success().stdout("10\n");
}

#[test]
fn assignment_and_expression_on_one_line() {
    run_bc("a=5; a*2\n").success().stdout("10\n");
}

#[test]
fn division_by_zero_reports_error_and_exits_one() {
    run_bc("1/0\n")
        .failure()
        .code(1)
        .stdout("")
        .stderr(str::contains("bc: division by zero"));
}

#[test]
fn quit_stops_processing_in_stdin_mode() {
    run_bc("1+2\nquit\n2+2\n").success().stdout("3\n");
}

#[test]
fn syntax_error_exits_one() {
    run_bc("1+\n")
        .failure()
        .code(1)
        .stderr(str::contains("bc: syntax error"));
}

#[test]
fn scale_is_reported_and_ignored_without_exit() {
    run_bc("scale=2\n1+2\n")
        .success()
        .stdout("3\n")
        .stderr("bc: scale is not supported in this build\n");
}

#[test]
fn ibase_and_obase_are_reported_and_ignored() {
    run_bc("ibase=16\nobase=2\n1+1\n")
        .success()
        .stdout("2\n")
        .stderr(
            "bc: ibase is not supported in this build\n\
             bc: obase is not supported in this build\n",
        );
}

#[test]
fn define_is_reported_and_ignored_without_exit() {
    run_bc("define f(x) { return x*2 }\n1+1\n")
        .success()
        .stdout("2\n")
        .stderr(str::contains("bc: define is not supported in this build"));
}

#[test]
fn function_calls_are_reported_and_ignored_without_exit() {
    run_bc("foo(3)\n1+1\n")
        .success()
        .stdout("2\n")
        .stderr("bc: foo is not supported in this build\n");
}

#[test]
fn l_flag_is_reported_unsupported() {
    cargo_bin_cmd!("bc")
        .arg("-l")
        .write_stdin("1+1\n")
        .assert()
        .success()
        .stdout("2\n")
        .stderr("bc: -l is not supported in this build\n");
}

#[test]
fn file_mode_evaluates_each_line() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("prog.bc");
    fs::write(&file, "1+2\n2*3\n").unwrap();
    cargo_bin_cmd!("bc")
        .arg(&file)
        .assert()
        .success()
        .stdout("3\n6\n");
}

#[test]
fn quit_stops_file_processing() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("prog.bc");
    fs::write(&file, "1+2\nquit\n2+2\n").unwrap();
    cargo_bin_cmd!("bc")
        .arg(&file)
        .assert()
        .success()
        .stdout("3\n");
}

// File-mode error policy: report the error to stderr, keep evaluating the
// remaining statements, and exit 1 once the whole input is consumed.
#[test]
fn file_mode_reports_errors_keeps_going_and_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("bad.bc");
    fs::write(&file, "1+2\n1/0\n2+2\n").unwrap();
    cargo_bin_cmd!("bc")
        .arg(&file)
        .assert()
        .failure()
        .code(1)
        .stdout("3\n4\n")
        .stderr("bc: division by zero\n");
}

#[test]
fn missing_file_exits_one() {
    cargo_bin_cmd!("bc")
        .arg("definitely-missing.bc")
        .assert()
        .failure()
        .code(1)
        .stderr(str::contains("bc: cannot open"));
}
