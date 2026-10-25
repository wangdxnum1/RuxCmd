use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn nul_field_mode_preserves_embedded_newlines_and_invalid_bytes() {
    for flag in ["-z", "--zero-terminated"] {
        cargo_bin_cmd!("cut")
            .args([flag, "-d", ",", "-f", "2"])
            .write_stdin(b"x,keep\nline\0y,\xff\r\0bare\r\n\0\0".to_vec())
            .assert()
            .success()
            .stdout(b"keep\nline\0\xff\r\0bare\r\n\0\0".to_vec());
        cargo_bin_cmd!("cut")
            .args([flag, "-f", "1"])
            .write_stdin("")
            .assert()
            .success()
            .stdout("");
    }
}

#[test]
fn nul_byte_and_character_selection_keep_original_data() {
    cargo_bin_cmd!("cut")
        .args(["-z", "-b", "1-2"])
        .write_stdin(b"\xff\nX\0ab\r\n".to_vec())
        .assert()
        .success()
        .stdout(b"\xff\n\0ab\0".to_vec());
    cargo_bin_cmd!("cut")
        .args(["-z", "-c", "1"])
        .write_stdin("你a\0好b")
        .assert()
        .success()
        .stdout("你\0好\0");
    cargo_bin_cmd!("cut")
        .args(["-z", "-c", "1"])
        .write_stdin(b"\xffa\0".to_vec())
        .assert()
        .success()
        .stdout(b"\xff\0".to_vec());
}

#[test]
fn nul_only_delimited_and_custom_output_delimiter_work() {
    cargo_bin_cmd!("cut")
        .args([
            "-z",
            "-s",
            "-d",
            ",",
            "-f",
            "1,3",
            "--output-delimiter",
            "|",
        ])
        .write_stdin(b"a,b,c\0bare\0d,e,f".to_vec())
        .assert()
        .success()
        .stdout(b"a|c\0d|f\0".to_vec());
}
