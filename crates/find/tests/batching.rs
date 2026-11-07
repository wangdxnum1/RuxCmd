#[path = "../../../scripts/tests/support/batch_probe.rs"]
mod support;
use assert_cmd::cargo::cargo_bin_cmd;
use ruxcmd_core::command_batch::{MAX_COMMAND_UNITS, command_units};
use support::Probe;

fn tree(probe: &Probe) -> (std::path::PathBuf, Vec<String>) {
    let root = probe.dir.path().join("source 中文");
    std::fs::create_dir(&root).unwrap();
    let mut expected = Vec::new();
    for i in 0..120 {
        let path = root.join(format!("{i:03}-{}.txt", "a".repeat(95)));
        std::fs::write(&path, b"data").unwrap();
        expected.push(path.display().to_string());
    }
    (root, expected)
}

#[test]
fn batches_preserve_all_paths_and_fixed_arguments_after_child_failure() {
    let probe = Probe::new();
    let (root, mut expected) = tree(&probe);
    cargo_bin_cmd!("find")
        .arg(&root)
        .args(["-type", "f", "-exec"])
        .args(probe.prefix("7"))
        .args(["fixed &", "", "{}", "+"])
        .assert()
        .code(1);
    let records = probe.records();
    assert!(
        records.len() > 1,
        "expected bounded batches, got {}",
        records.len()
    );
    let mut received = Vec::new();
    for record in records {
        assert_eq!(&record[..2], ["fixed &", ""]);
        let command: Vec<_> = probe
            .prefix("7")
            .into_iter()
            .chain(record.clone())
            .collect();
        assert!(command_units(&command) <= MAX_COMMAND_UNITS);
        received.extend_from_slice(&record[2..]);
    }
    received.sort();
    expected.sort();
    assert_eq!(received, expected);
}

#[test]
fn separate_exec_actions_have_independent_batches() {
    let first = Probe::new();
    let second = Probe::new();
    let (root, mut expected) = tree(&first);
    cargo_bin_cmd!("find")
        .arg(root)
        .args(["-type", "f", "-exec"])
        .args(first.prefix("0"))
        .args(["{}", "+", "-exec"])
        .args(second.prefix("0"))
        .args(["{}", "+"])
        .assert()
        .success();
    expected.sort();
    for probe in [&first, &second] {
        assert!(probe.records().len() > 1);
        let mut got: Vec<_> = probe.records().into_iter().flatten().collect();
        got.sort();
        assert_eq!(got, expected);
    }
}

#[test]
fn zero_matches_do_not_execute_and_invalid_prefix_is_reported_once() {
    let probe = Probe::new();
    let (root, _) = tree(&probe);
    cargo_bin_cmd!("find")
        .arg(&root)
        .args(["-name", "missing", "-exec"])
        .args(probe.prefix("0"))
        .args(["{}", "+"])
        .assert()
        .success();
    assert!(probe.records().is_empty());
    let out = cargo_bin_cmd!("find")
        .arg(&root)
        .args(["-type", "f", "-exec"])
        .args(probe.prefix("0"))
        .arg("x".repeat(MAX_COMMAND_UNITS))
        .args(["{}", "+"])
        .assert()
        .code(1);
    assert!(probe.records().is_empty());
    assert_eq!(
        String::from_utf8_lossy(&out.get_output().stderr)
            .lines()
            .count(),
        1
    );
}

#[test]
fn overlong_item_is_skipped_but_short_items_still_execute() {
    let probe = Probe::new();
    let root = probe.dir.path().join("items");
    std::fs::create_dir(&root).unwrap();
    std::fs::write(root.join("short"), b"s").unwrap();
    std::fs::write(root.join("a".repeat(100)), b"l").unwrap();
    let padding = "x".repeat(MAX_COMMAND_UNITS - command_units(&probe.prefix("0")) - 3 - 25);
    cargo_bin_cmd!("find")
        .current_dir(root)
        .arg(".")
        .args(["-type", "f", "-exec"])
        .args(probe.prefix("0"))
        .arg(padding)
        .args(["{}", "+"])
        .assert()
        .code(1);
    let records = probe.records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].len(), 2);
    assert_eq!(records[0][1], ".\\short");
}
