#[path = "../../../scripts/tests/support/batch_probe.rs"]
mod support;
use assert_cmd::cargo::cargo_bin_cmd;
use support::Probe;

#[test]
fn quoted_and_escaped_arguments_reach_child_exactly() {
    let probe = Probe::new();
    cargo_bin_cmd!("xargs")
        .args(probe.prefix("0"))
        .write_stdin("\"alpha beta\" '' one\\ two '中文😀'\n")
        .assert()
        .success();
    assert_eq!(
        probe.records(),
        [vec!["alpha beta", "", "one two", "中文😀"]]
    );
}

#[test]
fn nul_mode_preserves_empty_items_and_literal_content() {
    let probe = Probe::new();
    cargo_bin_cmd!("xargs")
        .arg("-0")
        .args(probe.prefix("0"))
        .write_stdin("a\0\0中文 ' \\\r\n\0tail")
        .assert()
        .success();
    assert_eq!(probe.records(), [vec!["a", "", "中文 ' \\\r\n", "tail"]]);
}

#[test]
fn replace_uses_lines_and_replaces_every_marker_once() {
    let probe = Probe::new();
    cargo_bin_cmd!("xargs")
        .args(["-I", "{}"])
        .args(probe.prefix("0"))
        .arg("{}:{}")
        .write_stdin("  alpha beta\n\n'中文 😀'\n")
        .assert()
        .success();
    assert_eq!(
        probe.records(),
        [vec!["alpha beta:alpha beta"], vec!["中文 😀:中文 😀"]]
    );
}

#[test]
fn braces_are_literal_without_replace_and_child_options_are_preserved() {
    let probe = Probe::new();
    cargo_bin_cmd!("xargs")
        .args(probe.prefix("0"))
        .args(["{}", "-r", "-I"])
        .write_stdin("item\n")
        .assert()
        .success();
    assert_eq!(probe.records(), [vec!["{}", "-r", "-I", "item"]]);
}

#[test]
fn empty_input_modes_and_default_echo() {
    let probe = Probe::new();
    cargo_bin_cmd!("xargs")
        .arg("-r")
        .args(probe.prefix("0"))
        .write_stdin("")
        .assert()
        .success();
    cargo_bin_cmd!("xargs")
        .args(["-I", "{}"])
        .args(probe.prefix("0"))
        .write_stdin("")
        .assert()
        .success();
    assert!(probe.records().is_empty());
    cargo_bin_cmd!("xargs")
        .args(probe.prefix("0"))
        .write_stdin("")
        .assert()
        .success();
    assert_eq!(probe.records(), [Vec::<String>::new()]);
    let result = cargo_bin_cmd!("xargs")
        .write_stdin("a b\n")
        .assert()
        .success();
    assert_eq!(result.get_output().stdout, b"a b\n");
}

#[test]
fn child_failures_accumulate_but_255_stops() {
    let probe = Probe::new();
    cargo_bin_cmd!("xargs")
        .args(["-n", "1"])
        .args(probe.prefix("by-item"))
        .write_stdin("fail ok stop later\n")
        .assert()
        .code(124);
    assert_eq!(probe.records(), [vec!["fail"], vec!["ok"], vec!["stop"]]);
    let probe = Probe::new();
    cargo_bin_cmd!("xargs")
        .args(["-n", "1"])
        .args(probe.prefix("by-item"))
        .write_stdin("fail ok\n")
        .assert()
        .code(123);
    assert_eq!(probe.records(), [vec!["fail"], vec!["ok"]]);
}

#[test]
fn invalid_input_and_missing_program_fail_explicitly() {
    for data in [
        vec![0xff],
        b"'unfinished".to_vec(),
        b"unfinished\\".to_vec(),
        b"a\0b".to_vec(),
    ] {
        let probe = Probe::new();
        cargo_bin_cmd!("xargs")
            .args(probe.prefix("0"))
            .write_stdin(data)
            .assert()
            .code(1);
        assert!(probe.records().is_empty());
    }
    cargo_bin_cmd!("xargs")
        .arg("__ruxcmd_missing_program_1794__")
        .write_stdin("a")
        .assert()
        .code(127);
    cargo_bin_cmd!("xargs").args(["-n", "0"]).assert().code(1);
}

#[test]
fn explicit_n_still_obeys_length_and_every_argument_survives() {
    let probe = Probe::new();
    let items: Vec<_> = (0..30)
        .map(|i| format!("{i}-{}😀\\", "中".repeat(500)))
        .collect();
    cargo_bin_cmd!("xargs")
        .args(["-0", "-n", "30"])
        .args(probe.prefix("0"))
        .write_stdin(items.join("\0"))
        .assert()
        .success();
    let records = probe.records();
    assert!(records.len() > 1);
    assert_eq!(records.into_iter().flatten().collect::<Vec<_>>(), items);
}

#[test]
fn oversized_item_and_expanded_template_never_execute() {
    for replacement in [false, true] {
        let probe = Probe::new();
        let mut cmd = cargo_bin_cmd!("xargs");
        if replacement {
            cmd.args(["-I", "{}"]).args(probe.prefix("0")).arg("{}:{}");
        } else {
            cmd.arg("-0").args(probe.prefix("0"));
        }
        cmd.write_stdin("a".repeat(if replacement { 4500 } else { 9000 }))
            .assert()
            .code(1);
        assert!(probe.records().is_empty());
    }
}

#[test]
fn replacement_is_sized_after_substitution_and_option_order_matches_gnu() {
    let probe = Probe::new();
    let marker = "X".repeat(1500);
    cargo_bin_cmd!("xargs")
        .args(["-I", &marker])
        .args(probe.prefix("0"))
        .arg(marker.repeat(6))
        .write_stdin("a\n")
        .assert()
        .success();
    assert_eq!(probe.records(), [vec!["aaaaaa"]]);
    for (options, expected) in [
        (vec!["-I", "{}", "-n", "1"], vec![vec!["a b"], vec!["c d"]]),
        (vec!["-n", "2", "-I", "{}"], vec![vec!["a b"], vec!["c d"]]),
        (
            vec!["-I", "{}", "-n", "2"],
            vec![vec!["{}", "a", "b"], vec!["{}", "c", "d"]],
        ),
    ] {
        let probe = Probe::new();
        cargo_bin_cmd!("xargs")
            .args(options)
            .args(probe.prefix("0"))
            .arg("{}")
            .write_stdin("a b\nc d\n")
            .assert()
            .success();
        assert_eq!(probe.records(), expected);
    }
}

#[test]
fn commands_start_at_n_before_eof_and_do_not_consume_input() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    let probe = Probe::new();
    let mut child = Command::new(env!("CARGO_BIN_EXE_xargs"))
        .args(["-0", "-n", "1"])
        .args(probe.prefix("stdin"))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    input.write_all(b"first\0").unwrap();
    input.flush().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !probe.log.exists() {
        if Instant::now() >= deadline {
            child.kill().unwrap();
            drop(input);
            child.wait().unwrap();
            panic!("first batch did not execute before EOF, or inherited stdin blocked it");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    input.write_all(b"second\0").unwrap();
    drop(input);
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        probe.records(),
        [vec!["first", "stdin:"], vec!["second", "stdin:"]]
    );
}

#[test]
fn fixed_prefix_and_non_executable_file_fail_without_child_execution() {
    let probe = Probe::new();
    cargo_bin_cmd!("xargs")
        .args(probe.prefix("0"))
        .arg("x".repeat(8000))
        .write_stdin("a")
        .assert()
        .code(1);
    assert!(probe.records().is_empty());
    let invalid = probe.dir.path().join("not-executable.txt");
    std::fs::write(&invalid, "text").unwrap();
    cargo_bin_cmd!("xargs")
        .arg(invalid)
        .write_stdin("a")
        .assert()
        .code(126);
}

#[test]
fn replace_aliases_literal_windows_arguments_and_invalid_nul_encoding() {
    let probe = Probe::new();
    for options in [vec!["-i"], vec!["--replace"], vec!["--replace={}"]] {
        cargo_bin_cmd!("xargs")
            .args(options)
            .args(probe.prefix("0"))
            .arg("{}:{}")
            .write_stdin("a b\n")
            .assert()
            .success();
    }
    assert_eq!(probe.records(), vec![vec!["a b:a b"]; 3]);
    let probe = Probe::new();
    let items = ["a\"b", "space and tail\\", "{}", "😀"];
    cargo_bin_cmd!("xargs")
        .arg("-0")
        .args(probe.prefix("0"))
        .write_stdin(items.join("\0"))
        .assert()
        .success();
    assert_eq!(probe.records(), [items]);
    let probe = Probe::new();
    cargo_bin_cmd!("xargs")
        .args(["-0", "-r"])
        .args(probe.prefix("0"))
        .write_stdin(vec![0xff, 0])
        .assert()
        .code(1);
    assert!(probe.records().is_empty());
}
