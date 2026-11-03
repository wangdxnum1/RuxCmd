use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn message_with_tag_goes_to_stderr() {
    cargo_bin_cmd!("logger")
        .args(["-s", "-t", "mytag", "hello"])
        .assert()
        .success()
        .stderr(predicates::str::contains("mytag: hello"));
}

#[test]
fn reads_stdin_when_no_message() {
    cargo_bin_cmd!("logger")
        .args(["-s"])
        .write_stdin("line1\nline2\n")
        .assert()
        .success()
        .stderr(predicates::str::contains("line1").and(predicates::str::contains("line2")));
}

#[test]
fn no_args_reads_empty_stdin() {
    cargo_bin_cmd!("logger").assert().success();
}

#[test]
fn tag_defaults_to_username() {
    cargo_bin_cmd!("logger")
        .args(["-s", "msg"])
        .assert()
        .success()
        .stderr(predicates::str::contains(": msg"));
}

#[test]
fn forced_fallback_writes_message_and_reason_to_stderr() {
    // Injection hook for the degraded path: without it the event log may be
    // writable, and a failure would depend on host permissions.
    cargo_bin_cmd!("logger")
        .env("RUXCMD_LOGGER_FORCE_STDERR", "1")
        .args(["-t", "fallback", "hello"])
        .assert()
        .success()
        .stderr(
            predicates::str::contains("writing to stderr instead")
                .and(predicates::str::contains("fallback: hello")),
        );
}

#[test]
fn fallback_still_honors_the_stderr_echo_flag() {
    cargo_bin_cmd!("logger")
        .env("RUXCMD_LOGGER_FORCE_STDERR", "1")
        .args(["-s", "-t", "echoed", "twice"])
        .assert()
        .success()
        .stderr(predicates::str::contains("echoed: twice"));
}

#[test]
fn priority_option_accepts_facility_prefix() {
    cargo_bin_cmd!("logger")
        .env("RUXCMD_LOGGER_FORCE_STDERR", "1")
        .args(["-p", "user.err", "-s", "boom"])
        .assert()
        .success()
        .stderr(predicates::str::contains(": boom"));
}

#[test]
fn invalid_priority_is_rejected() {
    cargo_bin_cmd!("logger")
        .env("RUXCMD_LOGGER_FORCE_STDERR", "1")
        .args(["-p", "bogus", "msg"])
        .assert()
        .failure();
}
