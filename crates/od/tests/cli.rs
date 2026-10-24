use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn t_x1_prints_single_byte_hex_without_ascii_column() {
    cargo_bin_cmd!("od")
        .args(["-t", "x1"])
        .write_stdin("ab\n")
        .assert()
        .success()
        .stdout("0000000 61 62 0a\n0000003\n");
}

#[test]
fn t_u1_d1_o1_single_byte_modes() {
    cargo_bin_cmd!("od")
        .args(["-t", "u1"])
        .write_stdin("abc")
        .assert()
        .success()
        .stdout("0000000  97  98  99\n0000003\n");
    cargo_bin_cmd!("od")
        .args(["-t", "d1"])
        .write_stdin(b"\xff\x0a".as_slice())
        .assert()
        .success()
        .stdout("0000000   -1   10\n0000002\n");
    cargo_bin_cmd!("od")
        .args(["-t", "o1"])
        .write_stdin("abc")
        .assert()
        .success()
        .stdout("0000000 141 142 143\n0000003\n");
}

#[test]
fn t_a_prints_named_characters() {
    cargo_bin_cmd!("od")
        .args(["-t", "a"])
        .write_stdin(b"\x00\x7f\x80\xff".as_slice())
        .assert()
        .success()
        .stdout("0000000 nul del nul del\n0000004\n");
}

#[test]
fn t_f4_prints_real_floats() {
    // IEEE-754 little-endian bytes of 1.0f32.
    cargo_bin_cmd!("od")
        .args(["-t", "f4"])
        .write_stdin(vec![0x00, 0x00, 0x80, 0x3f])
        .assert()
        .success()
        .stdout(format!("0000000{}1\n0000004\n", " ".repeat(15)));
    // 2.5f32
    cargo_bin_cmd!("od")
        .args(["-t", "f4"])
        .write_stdin(vec![0x00, 0x00, 0x20, 0x40])
        .assert()
        .success()
        .stdout(format!("0000000{}2.5\n0000004\n", " ".repeat(13)));
}

#[test]
fn t_f8_prints_real_floats() {
    // IEEE-754 little-endian bytes of 1.0f64.
    cargo_bin_cmd!("od")
        .args(["-t", "f8"])
        .write_stdin(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f])
        .assert()
        .success()
        .stdout(format!("0000000{}1\n0000010\n", " ".repeat(24)));
}

#[test]
fn default_mode_has_no_ascii_column_and_matches_t_o2() {
    let out_default = cargo_bin_cmd!("od")
        .write_stdin("ab\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let out_o2 = cargo_bin_cmd!("od")
        .args(["-t", "o2"])
        .write_stdin("ab\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(out_default, out_o2);
    assert_eq!(String::from_utf8(out_default).unwrap(), "0000000 061141 000012\n0000003\n");
}

#[test]
fn t_c_prints_gnu_style_escapes() {
    cargo_bin_cmd!("od")
        .args(["-t", "c"])
        .write_stdin(b"a\nb\x01\x80".as_slice())
        .assert()
        .success()
        .stdout("0000000   a  \\n   b 001 200\n0000005\n");
}

#[test]
fn t_x1z_appends_ascii_trailer() {
    cargo_bin_cmd!("od")
        .args(["-t", "x1z"])
        .write_stdin("ab")
        .assert()
        .success()
        .stdout(format!(
            "0000000 61 62{}>ab<\n0000002\n",
            " ".repeat(44)
        ));
}

#[test]
fn words_are_little_endian_and_d_is_signed() {
    cargo_bin_cmd!("od")
        .args(["-t", "u2"])
        .write_stdin("ab")
        .assert()
        .success()
        .stdout("0000000 25185\n0000002\n");
    cargo_bin_cmd!("od")
        .args(["-t", "d2"])
        .write_stdin(b"\xff\xff".as_slice())
        .assert()
        .success()
        .stdout("0000000     -1\n0000002\n");
    cargo_bin_cmd!("od")
        .args(["-t", "x4"])
        .write_stdin(vec![0x01, 0x02, 0x03, 0x04])
        .assert()
        .success()
        .stdout("0000000 04030201\n0000004\n");
}

#[test]
fn bare_t_x_defaults_to_four_byte_words() {
    cargo_bin_cmd!("od")
        .args(["-t", "x"])
        .write_stdin("ab")
        .assert()
        .success()
        .stdout("0000000 00006261\n0000002\n");
}

#[test]
fn address_radixes_and_no_address() {
    cargo_bin_cmd!("od")
        .args(["-A", "d", "-t", "o2"])
        .write_stdin("ab")
        .assert()
        .success()
        .stdout("0000000 061141\n0000002\n");
    cargo_bin_cmd!("od")
        .args(["-A", "n", "-t", "x1"])
        .write_stdin("ab")
        .assert()
        .success()
        .stdout(" 61 62\n");
}

#[test]
fn empty_input_prints_final_offset_only() {
    cargo_bin_cmd!("od")
        .write_stdin("")
        .assert()
        .success()
        .stdout("0000000\n");
}

#[test]
fn invalid_type_specs_are_rejected() {
    cargo_bin_cmd!("od")
        .args(["-t", "c1"])
        .write_stdin("ab")
        .assert()
        .code(1)
        .stderr(predicates::str::contains("invalid character '1'"));
    cargo_bin_cmd!("od")
        .args(["-t", "x3"])
        .write_stdin("ab")
        .assert()
        .code(1)
        .stderr(predicates::str::contains("3-byte integral type"));
    cargo_bin_cmd!("od")
        .args(["-t", "q1"])
        .write_stdin("ab")
        .assert()
        .code(1)
        .stderr(predicates::str::contains("invalid character 'q'"));
}

#[test]
fn skip_past_end_is_an_error() {
    cargo_bin_cmd!("od")
        .args(["-j", "5"])
        .write_stdin("a")
        .assert()
        .code(1)
        .stderr(predicates::str::contains("cannot skip past end"));
}
