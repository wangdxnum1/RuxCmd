use assert_cmd::cargo::cargo_bin_cmd;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

// cargo test puts target/debug first on PATH, and this workspace builds its
// own echo.exe/ping.exe there, so resolve system binaries by absolute path
// under %SystemRoot% to test the real Windows tools.
fn cmd_exe() -> String {
    format!(
        "{}\\System32\\cmd.exe",
        std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into())
    )
}

fn ping_exe() -> String {
    format!(
        "{}\\System32\\ping.exe",
        std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into())
    )
}

#[test]
fn runs_command_to_success() {
    cargo_bin_cmd!("nohup")
        .args([cmd_exe().as_str(), "/c", "exit", "0"])
        .assert()
        .success();
}

#[test]
fn propagates_child_exit_code() {
    cargo_bin_cmd!("nohup")
        .args([cmd_exe().as_str(), "/c", "exit", "3"])
        .assert()
        .failure()
        .code(3);
}

#[test]
fn passes_stdout_through_when_not_a_terminal() {
    // assert_cmd pipes our stdout, so nohup's is_terminal() check is false and
    // the child's output must arrive on nohup's stdout unchanged. The
    // nohup.out path (stdout is a terminal) cannot be triggered headlessly.
    cargo_bin_cmd!("nohup")
        .args([cmd_exe().as_str(), "/c", "echo", "hello"])
        .assert()
        .success()
        .stdout("hello\r\n");
}

#[test]
fn redirects_child_stderr_to_stdout() {
    // stderr is always pointed at stdout, even when stdout itself was not
    // redirected: both echoes must land on nohup's single stdout stream.
    // (`&` and `>&2` are written without a leading space because cmd's echo
    // would otherwise include that space in the text it prints.)
    cargo_bin_cmd!("nohup")
        .args([cmd_exe().as_str(), "/c", "echo out& echo err>&2"])
        .assert()
        .success()
        .stdout("out\r\nerr\r\n");
}

#[test]
fn reports_missing_program_with_exit_127() {
    cargo_bin_cmd!("nohup")
        .args(["definitely-not-a-real-program-xyz"])
        .assert()
        .failure()
        .code(127)
        .stderr(predicates::str::contains(
            "nohup: definitely-not-a-real-program-xyz",
        ));
}

#[test]
fn detached_child_completes_on_its_own() {
    // Spawn nohup (cmd /c ping -n 3, then write a done marker) and drop the
    // child handle without ever waiting on it. Windows has no SIGHUP to
    // deliver, so this cannot literally kill the child; what it proves is
    // that the breakaway/new-process-group spawn succeeds and the whole
    // detached chain finishes by itself while the test only polls.
    let dir = tempfile::tempdir().unwrap();
    let script = format!(
        "echo started > started.marker && {} -n 3 127.0.0.1 >nul && echo done > done.marker",
        ping_exe()
    );
    let child = Command::new(env!("CARGO_BIN_EXE_nohup"))
        .args([cmd_exe().as_str(), "/c", script.as_str()])
        .current_dir(dir.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    drop(child);

    let deadline = Instant::now() + Duration::from_secs(60);
    while Instant::now() < deadline {
        if dir.path().join("done.marker").exists() {
            break;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    assert!(
        dir.path().join("started.marker").exists(),
        "detached child never started"
    );
    assert!(
        dir.path().join("done.marker").exists(),
        "detached child did not finish within 60s"
    );
}

#[test]
fn help_and_version_work() {
    cargo_bin_cmd!("nohup")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage"));
    cargo_bin_cmd!("nohup")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("nohup"));
}
