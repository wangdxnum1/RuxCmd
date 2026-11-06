use assert_cmd::cargo::cargo_bin_cmd;

fn write_file(dir: &tempfile::TempDir, name: &str, content: &str) -> String {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path.to_string_lossy().into_owned()
}

#[test]
fn identical_files_exit_zero() {
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "a.txt", "a\nb\nc\n");
    let f2 = write_file(&dir, "b.txt", "a\nb\nc\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(0)
        .stdout("");
}

#[test]
fn single_line_change_merges_into_c_command() {
    // Behavior fix (0.4.6): GNU merges the adjacent delete+add into a single
    // change command ("3c3" with the "---" separator). The old expectations
    // locked the unmerged "3d2"/"2a3" split, which GNU never emits.
    // Byte-compared against GNU diffutils 3.12.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb\nx\n");
    let f2 = write_file(&dir, "f2.txt", "a\nb\ny\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout("3c3\n< x\n---\n> y\n");
}

#[test]
fn multi_line_change_folds_both_ranges() {
    // GNU: 1,3c1,3 — same-side consecutive lines fold into "n1,n2".
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb\nc\n");
    let f2 = write_file(&dir, "f2.txt", "x\ny\nz\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout("1,3c1,3\n< a\n< b\n< c\n---\n> x\n> y\n> z\n");
}

#[test]
fn single_delete_keeps_d_command() {
    // GNU: 3d2 — pure delete with the right anchor at the previous common line.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb\nc\nd\n");
    let f2 = write_file(&dir, "f2.txt", "a\nb\nd\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout("3d2\n< c\n");
}

#[test]
fn delete_at_file_start_anchors_to_zero() {
    // GNU: 1,2d0 — right anchor 0 when the deletion precedes every common line.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "x\ny\na\n");
    let f2 = write_file(&dir, "f2.txt", "a\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout("1,2d0\n< x\n< y\n");
}

#[test]
fn add_at_file_start_anchors_to_zero() {
    // GNU: 0a1,3 — left anchor 0 when the addition precedes every common line.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\n");
    let f2 = write_file(&dir, "f2.txt", "x\ny\nz\na\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout("0a1,3\n> x\n> y\n> z\n");
}

#[test]
fn add_after_common_lines_folds_range() {
    // GNU: 2a3,4 — left anchor is the previous common line number.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb\nc\n");
    let f2 = write_file(&dir, "f2.txt", "a\nb\nx\ny\nc\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout("2a3,4\n> x\n> y\n");
}

#[test]
fn segments_separated_by_common_lines_stay_separate() {
    // GNU: one command per diff segment; a single common line splits hunks
    // (normal diff never merges across common lines), and each anchor counts
    // the preceding common lines on both sides.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "p\nq\nr\ns\nt\n");
    let f2 = write_file(&dir, "f2.txt", "p\n1\nq\nr\n2\ns\nt\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout("1a2\n> 1\n3a5\n> 2\n");
}

#[test]
fn change_at_file_end_covers_both_tails() {
    // GNU: 3,4c3,4 — trailing replacement folds both sides.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb\nc\nd\n");
    let f2 = write_file(&dir, "f2.txt", "a\nb\nx\ny\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout("3,4c3,4\n< c\n< d\n---\n> x\n> y\n");
}

#[test]
fn delete_and_add_around_one_common_line() {
    // GNU: "1d0" then "2a2" — the common line b separates the two commands,
    // and the add anchor is the common line number after the delete.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb\n");
    let f2 = write_file(&dir, "f2.txt", "b\na\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout("1d0\n< a\n2a2\n> a\n");
}

#[test]
fn interleaved_delete_add_add_segment_split() {
    // GNU: "1c1" merges the leading delete+add pair, then "2a3" adds the tail.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "x\nb\n");
    let f2 = write_file(&dir, "f2.txt", "y\nb\nz\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout("1c1\n< x\n---\n> y\n2a3\n> z\n");
}

#[test]
fn delete_entire_file_anchors_to_zero() {
    // GNU: deleting every line of file1 (file2 empty) is a pure delete whose
    // right anchor is 0.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "x\ny\n");
    let f2 = write_file(&dir, "f2.txt", "");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout("1,2d0\n< x\n< y\n");
}

#[test]
fn differing_files_exit_one() {
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "one\n");
    let f2 = write_file(&dir, "f2.txt", "two\n");

    cargo_bin_cmd!("diff").arg(&f1).arg(&f2).assert().code(1);
}

#[test]
fn missing_file_exits_two_with_error_on_stderr() {
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "one\n");
    let missing = dir.path().join("missing.txt");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&missing)
        .assert()
        .code(2)
        .stderr(predicates::str::contains("Error reading"));
}

#[test]
fn unified_format_exits_one_on_difference() {
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb\nx\n");
    let f2 = write_file(&dir, "f2.txt", "a\nb\ny\n");

    cargo_bin_cmd!("diff")
        .arg("-u")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout(predicates::str::contains("@@"));
}
