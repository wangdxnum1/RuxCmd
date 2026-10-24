use assert_cmd::cargo::cargo_bin_cmd;

fn system_cmd() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap());
    p.push("System32");
    p.push("cmd.exe");
    p
}

#[test]
fn null_delimited_input() {
    let out = cargo_bin_cmd!("xargs")
        .args(["-0", "-n", "1"])
        .arg(system_cmd())
        .arg("/c")
        .arg("echo")
        .write_stdin("a\0b\0")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    assert!(
        stdout.contains('a') && stdout.contains('b'),
        "stdout: {:?}",
        stdout
    );
}

#[test]
fn long_input_is_batched_without_loss() {
    // 3000 short tokens exceed the 8000-char batch cap: several batches run
    // and every token must survive.
    let mut input = String::new();
    for i in 0..3000 {
        input.push_str(&format!("t{} ", i));
    }
    let out = cargo_bin_cmd!("xargs")
        .arg(system_cmd())
        .arg("/c")
        .arg("echo")
        .write_stdin(input)
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    for i in [0usize, 1499, 2999] {
        assert!(stdout.contains(&format!("t{}", i)), "missing t{}", i);
    }
}

#[test]
fn empty_input_still_runs_once() {
    let out = cargo_bin_cmd!("xargs")
        .arg(system_cmd())
        .arg("/c")
        .arg("echo")
        .arg("RAN")
        .write_stdin("")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    assert!(
        stdout.contains("RAN"),
        "empty input must run once: {:?}",
        stdout
    );
}
