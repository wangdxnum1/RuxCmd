use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn passes_arguments_after_options_to_child() {
    // Regression (P0 #36): the old wmic main path built `cmd /c <program>` and
    // dropped args.command[1..] entirely, so `nice -n 5 cmd /c echo hi` never
    // printed "hi" (it ran `cmd /c cmd` instead). Fails pre-fix wherever wmic
    // exists; on wmic-less systems (Win11 24H2+) the legacy fallback already
    // forwarded args, which is why the priority test below is the one that
    // fails pre-fix there.
    cargo_bin_cmd!("nice")
        .args(["-n", "5", "cmd", "/c", "echo", "hi"])
        .assert()
        .success()
        .stdout("hi\r\n");
}

#[test]
fn propagates_child_exit_code() {
    // Regression (P0 #36): wmic's `process call create` returns without
    // waiting for the created process, so the old main path always exited 0
    // regardless of the child's exit code.
    cargo_bin_cmd!("nice")
        .args(["-n", "5", "cmd", "/c", "exit", "3"])
        .assert()
        .failure()
        .code(3);
}

#[test]
fn runs_command_without_adjustment() {
    cargo_bin_cmd!("nice")
        .args(["cmd", "/c", "exit", "0"])
        .assert()
        .success();
}

#[test]
fn applies_priority_class_to_child() {
    // Regression (P0 #36): the old wmic path passed an illegal priority value,
    // and the no-wmic fallback (Win11 24H2+) spawned the child without setting
    // any priority class at all, so `-n 5` was silently ignored (child ran at
    // Normal). With the fix, adjustment 5..14 maps to BELOW_NORMAL.
    cargo_bin_cmd!("nice")
        .args([
            "-n",
            "5",
            "--",
            "powershell",
            "-NoProfile",
            "-Command",
            "[System.Diagnostics.Process]::GetCurrentProcess().PriorityClass",
        ])
        .assert()
        .success()
        .stdout(predicates::str::contains("BelowNormal"));
}
