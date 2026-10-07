//! Regression tests for P0 #49: `tty` must decide via `IsTerminal` (std), not
//! the `CON` environment variable, which Windows never sets.
//!
//! Under assert_cmd the child's stdin/stdout/stderr are all pipes
//! (`assert_cmd::Command` spawns with `Stdio::piped()` for every stream), so
//! every test below runs in a guaranteed "no tty anywhere" environment.

use assert_cmd::cargo::cargo_bin_cmd;

// --- P0 #49: the main path must use IsTerminal, not the CON env var ---

#[test]
fn tty_with_piped_stdin_prints_not_a_tty_and_exits_1() {
    cargo_bin_cmd!("tty")
        .assert()
        .code(1)
        .stdout("not a tty\n")
        .stderr("");
}

#[test]
fn tty_ignores_the_con_environment_variable() {
    // The pre-fix implementation printed $CON and exited 0 when it was set;
    // detection must come from IsTerminal instead.
    cargo_bin_cmd!("tty")
        .env("CON", r"\\.\CON")
        .assert()
        .code(1)
        .stdout("not a tty\n")
        .stderr("");
}

// --- -a: success only when some stream is a tty ---

#[test]
fn tty_a_exits_1_when_no_stream_is_a_tty() {
    // Pre-fix `-a` always exited 0, even when no stream was a tty.
    cargo_bin_cmd!("tty")
        .arg("-a")
        .assert()
        .code(1)
        .stdout("not a tty\nnot a tty\nnot a tty\n")
        .stderr("");
}

// --- -s: silent, exit code only ---

#[test]
fn tty_silent_prints_nothing_but_still_fails() {
    cargo_bin_cmd!("tty")
        .arg("-s")
        .assert()
        .code(1)
        .stdout("")
        .stderr("");
}

#[test]
fn tty_a_silent_prints_nothing_but_still_fails() {
    cargo_bin_cmd!("tty")
        .args(["-a", "-s"])
        .assert()
        .code(1)
        .stdout("")
        .stderr("");
}
