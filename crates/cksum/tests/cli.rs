use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use tempfile::tempdir;

// Reference values verified against GNU coreutils cksum (POSIX CRC-32).
const CKSUM_123456789: &str = "930766865 9";
const CKSUM_EMPTY: &str = "4294967295 0";
const CKSUM_HELLO: &str = "3287646509 5";

#[test]
fn known_vector_123456789() {
    cargo_bin_cmd!("cksum")
        .write_stdin("123456789")
        .assert()
        .success()
        .stdout(format!("{CKSUM_123456789} -\n"));
}

#[test]
fn empty_input_yields_the_complemented_initial_crc() {
    cargo_bin_cmd!("cksum")
        .write_stdin("")
        .assert()
        .success()
        .stdout(format!("{CKSUM_EMPTY} -\n"));
}

#[test]
fn computes_checksums_for_each_file() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("nine.txt"), b"123456789").unwrap();
    fs::write(dir.path().join("hello.txt"), b"hello").unwrap();
    cargo_bin_cmd!("cksum")
        .current_dir(dir.path())
        .args(["nine.txt", "hello.txt"])
        .assert()
        .success()
        .stdout(format!(
            "{CKSUM_123456789} nine.txt\n{CKSUM_HELLO} hello.txt\n"
        ));
}

#[test]
fn dash_argument_reads_stdin_named_dash() {
    cargo_bin_cmd!("cksum")
        .arg("-")
        .write_stdin("hello")
        .assert()
        .success()
        .stdout(format!("{CKSUM_HELLO} -\n"));
}

#[test]
fn binary_bytes_are_checksummed_verbatim() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("data.bin"), b"a\0b\xffc".as_slice()).unwrap();
    cargo_bin_cmd!("cksum")
        .current_dir(dir.path())
        .arg("data.bin")
        .assert()
        .success()
        .stdout("1746395454 5 data.bin\n");
}

#[test]
fn algorithm_flag_accepts_crc() {
    cargo_bin_cmd!("cksum")
        .args(["-a", "crc"])
        .write_stdin("123456789")
        .assert()
        .success()
        .stdout(format!("{CKSUM_123456789} -\n"));
    cargo_bin_cmd!("cksum")
        .arg("--algorithm=crc")
        .write_stdin("hello")
        .assert()
        .success()
        .stdout(format!("{CKSUM_HELLO} -\n"));
}

#[test]
fn unsupported_algorithm_is_rejected() {
    cargo_bin_cmd!("cksum")
        .args(["-a", "sha256"])
        .assert()
        .code(1)
        .stderr(predicates::str::contains("not supported"));
}

#[test]
fn missing_file_is_reported() {
    let dir = tempdir().unwrap();
    cargo_bin_cmd!("cksum")
        .current_dir(dir.path())
        .arg("no-such-file")
        .assert()
        .code(1)
        .stderr(predicates::str::contains("cksum: no-such-file"));
}
