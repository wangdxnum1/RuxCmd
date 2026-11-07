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

// scale support: fixed-point arithmetic with the global scale variable.

#[test]
fn scale_assignment_is_silent() {
    run_bc("scale=2\n").success().stdout("").stderr("");
}

#[test]
fn scale_default_zero_keeps_integer_division() {
    run_bc("10/3\n").success().stdout("3\n");
}

#[test]
fn scale_two_division_prints_two_decimals() {
    run_bc("scale=2; 10/3\n").success().stdout("3.33\n");
}

#[test]
fn scale_five_division_prints_five_decimals() {
    run_bc("scale=5; 1/3\n").success().stdout("0.33333\n");
}

#[test]
fn scale_division_truncates_toward_zero() {
    run_bc("scale=2; 10/4\n-7/3\n")
        .success()
        .stdout("2.50\n-2.33\n");
}

#[test]
fn scale_remainder_is_fractional() {
    // a % b = a - (a/b)*b with a/b truncated at the current scale.
    run_bc("scale=2; 10%3\n").success().stdout("0.01\n");
}

#[test]
fn scale_zero_remainder_stays_integer() {
    run_bc("10%3\n-7%3\n").success().stdout("1\n-1\n");
}

#[test]
fn negative_exponent_yields_fraction_at_scale() {
    run_bc("scale=2; 2^-1\nscale=5; 2^-1\n")
        .success()
        .stdout("0.50\n0.50000\n");
}

#[test]
fn positive_exponent_multiplies_scales() {
    // GNU bc: (1/3)^2 at scale 2 is 0.33*0.33 = 0.1089.
    run_bc("scale=2; (1/3)^2\n").success().stdout("0.1089\n");
}

#[test]
fn sqrt_truncates_to_current_scale() {
    run_bc("scale=2; sqrt(2)\n").success().stdout("1.41\n");
}

#[test]
fn sqrt_at_default_scale_truncates_to_integer() {
    run_bc("sqrt(2)\nsqrt(9)\n").success().stdout("1\n3\n");
}

#[test]
fn sqrt_of_fraction() {
    run_bc("scale=2; sqrt(0.25)\n").success().stdout("0.50\n");
}

#[test]
fn sqrt_of_negative_is_an_error() {
    run_bc("sqrt(-4)\n")
        .failure()
        .code(1)
        .stderr(str::contains("bc: square root of negative number"));
}

#[test]
fn scale_is_readable_and_stored_per_variable() {
    run_bc("scale=3\nscale\na=1/3\na\n")
        .success()
        .stdout("3\n0.333\n");
}

#[test]
fn decimal_literals_keep_their_scale() {
    run_bc("1.5+2\n0.5*0.5\n").success().stdout("3.5\n0.25\n");
}

#[test]
fn scale_out_of_range_is_an_error() {
    run_bc("scale=65\n1+1\n")
        .failure()
        .code(1)
        .stderr(str::contains("bc: scale must be between 0 and 64"));
    run_bc("scale=-1\n").failure().code(1);
}

#[test]
fn scale_max_is_assignable() {
    run_bc("scale=64\nscale\n").success().stdout("64\n");
}

// ibase/obase support: input and output radix like GNU bc.

#[test]
fn obase_two_prints_binary() {
    run_bc("obase=2; 10\n").success().stdout("1010\n");
}

#[test]
fn obase_hex_prints_uppercase_letters() {
    run_bc("obase=16; 255\n").success().stdout("FF\n");
}

#[test]
fn ibase_hex_reads_uppercase_letters() {
    run_bc("ibase=16; FF\n").success().stdout("255\n");
}

#[test]
fn ibase_hex_reads_lowercase_letters() {
    run_bc("ibase=16; ff\n").success().stdout("255\n");
}

#[test]
fn ibase_and_obase_set_together() {
    run_bc("ibase=16; obase=16; FF\n").success().stdout("FF\n");
}

#[test]
fn ibase_binary_reads_binary_digits() {
    run_bc("ibase=2; 1010\n").success().stdout("10\n");
}

#[test]
fn obase_hex_with_scale_keeps_fraction() {
    run_bc("ibase=10; obase=16; scale=4; 255/2\n")
        .success()
        .stdout("7F.8000\n");
}

#[test]
fn ibase_assignments_are_silent_and_bare_names_read_back() {
    // obase=16 prints its own value as 10 (hex for 16) and ibase as A
    // (decimal 10 in hex), matching GNU bc where special variables are
    // printed in the current output base.
    run_bc("obase=16\nobase\nibase\n")
        .success()
        .stdout("10\nA\n");
}

#[test]
fn letters_are_variables_again_below_ibase_eleven() {
    // With ibase back at 10 (via the hex digit A), F is a plain variable.
    run_bc("ibase=16; ibase=A; F\n").success().stdout("0\n");
}

#[test]
fn hex_input_supports_fractions() {
    // 0.8 in hex is 5/10, printed exactly.
    run_bc("ibase=16; obase=10; 0.8\n")
        .success()
        .stdout("0.5\n");
}

#[test]
fn obase_converts_negative_and_zero() {
    run_bc("obase=2; -10\nobase=16; 0\n")
        .success()
        .stdout("-1010\n0\n");
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
