use assert_cmd::cargo::cargo_bin_cmd;
use std::process::Output;

// P0 #42 regression. The old implementation always queried the *current* user
// (env USERNAME via `net user`) and used the USER positional argument only for
// display, so `groups someone-else` exited 0 while attributing the current
// user's groups to someone else. Its fallback path also fabricated groups from
// env vars. The tests below pin the new contract: other users are rejected and
// the current user gets real token groups on a single line.

fn run_groups(args: &[&str]) -> Output {
    cargo_bin_cmd!("groups")
        .args(args)
        .output()
        .expect("groups runs")
}

#[test]
fn other_user_argument_is_rejected_with_error() {
    cargo_bin_cmd!("groups")
        .arg("ruxcmd_no_such_local_user")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "cannot query other users on windows",
        ));
}

#[test]
fn current_user_prints_token_groups_on_one_line() {
    let out = run_groups(&[]);
    assert!(out.status.success(), "groups with no args must exit 0");
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let line = stdout.trim_end().to_string();
    assert!(!line.is_empty(), "groups must print the group list");
    assert!(
        !line.contains('\n'),
        "GNU groups prints one space separated line, got {stdout:?}"
    );
    for name in line.split(' ') {
        assert!(!name.trim().is_empty(), "no empty group names in {line:?}");
    }
    // A Windows process token always carries the well-known S-1-1-0 (Everyone)
    // and S-1-5-11 (Authenticated Users) groups; the old env-based fallback
    // could never list the real token content.
    assert!(
        line.contains("Everyone") || line.contains("Authenticated Users"),
        "token groups should include Everyone/Authenticated Users, got {line:?}"
    );
}

#[test]
fn current_user_name_argument_is_accepted_case_insensitively() {
    let me = std::env::var("USERNAME").unwrap_or_default();
    if me.is_empty() {
        return;
    }
    let out = run_groups(&[me.to_uppercase().as_str()]);
    assert!(
        out.status.success(),
        "passing the current user name (any case) must succeed, got {:?}",
        out.status
    );
}
