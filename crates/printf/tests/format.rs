use assert_cmd::cargo::cargo_bin_cmd;

fn run(args: &[&str]) -> (Vec<u8>, String, Option<i32>) {
    let out = cargo_bin_cmd!("printf").args(args).output().unwrap();
    (
        out.stdout,
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code(),
    )
}

#[test]
fn width_left_align_and_string_padding() {
    let (out, _err, code) = run(&["%5d|%-6s|", "42", "ab"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"   42|ab    |");
}

#[test]
fn zero_padded_float_with_precision() {
    let (out, _err, code) = run(&["%05.1f", "3.14159"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"003.1");
}

#[test]
fn string_width_and_precision() {
    let (out, _err, code) = run(&["%5.2s", "abcdef"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"   ab");
}

#[test]
fn integer_conversions() {
    let (out, _err, code) = run(&["%X|%o|%x|%i", "255", "8", "255", "-42"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"FF|10|ff|-42");
}

#[test]
fn alternate_form_flags() {
    let (out, _err, code) = run(&["%#x|%#o", "255", "8"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"0xff|010");
}

#[test]
fn scientific_notation_two_digit_exponent() {
    let (out, _err, code) = run(&["%e|%.2E", "1234.5", "1234.5"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"1.234500e+03|1.23E+03");
}

#[test]
fn general_format_strips_trailing_zeros() {
    let (out, _err, code) = run(&["%g|%g|%g|%g", "0.000012", "123456.0", "1234567.0", "0.0001"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"1.2e-05|123456|1.23457e+06|0.0001");
}

#[test]
fn unsigned_wraps_negative() {
    let (out, _err, code) = run(&["%u", "-1"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"18446744073709551615");
}

#[test]
fn char_conversion_takes_first_character() {
    let (out, _err, code) = run(&["%c|%c", "xy", ""]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"x|");
}

#[test]
fn b_directive_expands_escapes_in_argument() {
    let (out, _err, code) = run(&["%b", "a\\tb"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"a\tb");
}

#[test]
fn b_directive_stop_character_halts_all_output() {
    let (out, _err, code) = run(&["[%b]after", "x\\cy"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"[x");
}

#[test]
fn q_directive_quotes_for_shell() {
    let (out, _err, code) = run(&["%q", "it's"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"'it'\\''s'");
}

#[test]
fn percent_literal() {
    let (out, _err, code) = run(&["100%%"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"100%");
}

#[test]
fn format_is_reused_until_args_exhausted() {
    let (out, _err, code) = run(&["%s\n", "a", "b", "c"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"a\nb\nc\n");
}

#[test]
fn reuse_with_mixed_format() {
    let (out, _err, code) = run(&["x%s", "1", "2"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"x1x2");
}

#[test]
fn no_conversion_warns_on_excess_arguments() {
    let (out, err, code) = run(&["y\n", "1", "2", "3"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"y\n");
    assert!(
        err.contains("ignoring excess arguments, starting with '1'"),
        "stderr: {err}"
    );
}

#[test]
fn missing_arguments_become_empty_or_zero() {
    let (out, _err, code) = run(&["%s-%s|%d|%f|"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"-|0|0.000000|");
}

#[test]
fn invalid_numeric_reports_and_exits_one() {
    let (out, err, code) = run(&["[%d]", "zz"]);
    assert_eq!(code, Some(1));
    assert_eq!(out, b"[0]");
    assert!(err.contains("expected a numeric value"), "stderr: {err}");
}

#[test]
fn unknown_conversion_specification_fails() {
    let (_out, err, code) = run(&["%y"]);
    assert_eq!(code, Some(1));
    assert!(
        err.contains("invalid conversion specification"),
        "stderr: {err}"
    );
}

#[test]
fn format_escapes_octal_hex_and_c0() {
    let (out, _err, code) = run(&["\\a\\b\\f\\v\\e\\x41\\101|"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, vec![0x07, 0x08, 0x0C, 0x0B, 0x1B, b'A', b'A', b'|']);
}

#[test]
fn plus_and_space_flags_for_numbers() {
    let (out, _err, code) = run(&["%+d|% d|%+d", "42", "42", "-5"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"+42| 42|-5");
}

#[test]
fn star_takes_width_from_argument() {
    let (out, _err, code) = run(&["%*d|", "5", "42"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"   42|");
}

#[test]
fn integer_precision_pads_digits() {
    let (out, _err, code) = run(&["%.5d|", "42"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"00042|");
}

#[test]
fn float_precision_default_is_six() {
    let (out, _err, code) = run(&["%f", "2"]);
    assert_eq!(code, Some(0));
    assert_eq!(out, b"2.000000");
}
