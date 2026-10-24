use assert_cmd::cargo::cargo_bin_cmd;
use std::process::Command;

fn system_cmd() -> std::path::PathBuf {
    // cargo test prepends target/debug to PATH, where our own binaries live;
    // use the absolute system path so the child is really cmd.
    let mut p = std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap());
    p.push("System32");
    p.push("cmd.exe");
    p
}

#[test]
fn reports_gnu_format_with_real_times() {
    let out = cargo_bin_cmd!("time")
        .arg(system_cmd())
        .arg("/c")
        .arg("exit")
        .arg("0")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    // GNU format: real/user/sys lines like `0m0.003s`.
    for key in ["real", "user", "sys"] {
        let line = stdout
            .lines()
            .find(|l| l.starts_with(key))
            .unwrap_or_else(|| panic!("missing {} line in {:?}", key, stdout));
        let val = line.split('\t').nth(1).unwrap_or("");
        assert!(
            val.matches('m').count() == 1 && val.ends_with('s'),
            "bad {} value {:?} in {:?}",
            key,
            val,
            stdout
        );
    }
}

#[test]
fn exit_code_passes_through() {
    let out = Command::new(env!("CARGO_BIN_EXE_time"))
        .arg(system_cmd())
        .args(["/c", "exit", "3"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
}

#[test]
fn missing_command_exits_127() {
    let out = Command::new(env!("CARGO_BIN_EXE_time"))
        .arg("definitely_no_such_executable_ruxcmd")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(127));
}
