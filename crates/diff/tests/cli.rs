use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

fn write_file(dir: &tempfile::TempDir, name: &str, content: &str) -> String {
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    path.to_string_lossy().into_owned()
}

/// Paths the walk renders use forward-slash display form (ruxcmd-core).
fn shown(path: &std::path::Path) -> String {
    path.display().to_string().replace('\\', "/")
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
        .stderr(predicates::str::contains("No such file or directory"));
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

const NO_NEWLINE: &str = "\\ No newline at end of file";

#[test]
fn ignore_whitespace_folds_missing_final_newline() {
    // GNU -b ignores trailing whitespace, so the EOF-newline difference
    // disappears: 'a\nb' equals 'a\nb\n', and vs 'a\nb\nc\n' it is "2a3".
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb");
    let f2 = write_file(&dir, "f2.txt", "a\nb\n");
    let f3 = write_file(&dir, "f3.txt", "a\nb\nc\n");

    cargo_bin_cmd!("diff")
        .arg("-b")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(0)
        .stdout("");

    cargo_bin_cmd!("diff")
        .arg("-b")
        .arg(&f1)
        .arg(&f3)
        .assert()
        .code(1)
        .stdout("2a3\n> c\n");
}

#[test]
fn no_newline_marker_after_deleted_line() {
    // GNU diff: printf 'a\nb' vs printf 'a\nc\n' — the marker follows the
    // `< b` line that lacks the final newline. Byte-compared against GNU
    // diffutils 3.12.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb");
    let f2 = write_file(&dir, "f2.txt", "a\nc\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout(format!("2c2\n< b\n{}\n---\n> c\n", NO_NEWLINE));
}

#[test]
fn no_newline_marker_on_both_sides() {
    // GNU: printf 'a\nb' vs printf 'a\nc' — one marker per side.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb");
    let f2 = write_file(&dir, "f2.txt", "a\nc");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout(format!(
            "2c2\n< b\n{}\n---\n> c\n{}\n",
            NO_NEWLINE, NO_NEWLINE
        ));
}

#[test]
fn no_newline_marker_after_pure_delete() {
    // GNU: 2d1 with the marker after the deleted `< b` line.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb");
    let f2 = write_file(&dir, "f2.txt", "a\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout(format!("2d1\n< b\n{}\n", NO_NEWLINE));
}

#[test]
fn no_newline_marker_after_added_line() {
    // GNU: 1a2 with the marker after the added `> b` line.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\n");
    let f2 = write_file(&dir, "f2.txt", "a\nb");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout(format!("1a2\n> b\n{}\n", NO_NEWLINE));
}

#[test]
fn missing_newline_makes_same_text_differ() {
    // GNU treats the missing final newline as part of the line: 'a\nb' vs
    // 'a\nb\nc\n' is "2c2,3", not "2a3" — b without \n differs from b with \n.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb");
    let f2 = write_file(&dir, "f2.txt", "a\nb\nc\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout(format!("2c2,3\n< b\n{}\n---\n> b\n> c\n", NO_NEWLINE));
}

#[test]
fn no_newline_marker_in_unified_format() {
    // GNU: -u emits the same marker after the `-` line; a change covering the
    // whole file keeps the exact hunk header, so this is byte-comparable.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "x\nb");
    let f2 = write_file(&dir, "f2.txt", "y\nc\n");

    cargo_bin_cmd!("diff")
        .arg("-u")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout(predicates::str::starts_with(format!("--- {f1}\t")))
        .stdout(predicates::str::contains(format!("\n+++ {f2}\t")))
        .stdout(predicates::str::contains(format!(
            "@@ -1,2 +1,2 @@\n-x\n-b\n{}\n+y\n+c\n",
            NO_NEWLINE
        )));
}

#[test]
fn unified_marker_after_context_line() {
    // GNU: the marker also follows a context line that ends both files
    // without a newline.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "p\nq\nb");
    let f2 = write_file(&dir, "f2.txt", "P\nq\nb");

    cargo_bin_cmd!("diff")
        .arg("-u")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(1)
        .stdout(predicates::str::starts_with(format!("--- {f1}\t")))
        .stdout(predicates::str::contains(format!("\n+++ {f2}\t")))
        .stdout(predicates::str::contains(format!(
            "@@ -1,3 +1,3 @@\n-p\n+P\n q\n b\n{}\n",
            NO_NEWLINE
        )));
}

#[test]
fn unified_identical_inputs_print_nothing() {
    // GNU prints no ---/+++ headers at all when the inputs are identical.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "a\nb\n");
    let f2 = write_file(&dir, "f2.txt", "a\nb\n");

    cargo_bin_cmd!("diff")
        .arg("-u")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .code(0)
        .stdout("");
}

#[test]
fn stdin_as_first_operand() {
    // GNU: `printf 'hello' | diff - file` — `-` reads stdin; the stdin side
    // lacks the final newline so it carries the marker.
    let dir = tempfile::tempdir().unwrap();
    let f2 = write_file(&dir, "f2.txt", "hello\n");

    cargo_bin_cmd!("diff")
        .arg("-")
        .arg(&f2)
        .write_stdin("hello")
        .assert()
        .code(1)
        .stdout(format!("1c1\n< hello\n{}\n---\n> hello\n", NO_NEWLINE));
}

#[test]
fn stdin_as_second_operand() {
    // GNU: `diff file -` with a no-newline stdin side.
    let dir = tempfile::tempdir().unwrap();
    let f1 = write_file(&dir, "f1.txt", "hello\n");

    cargo_bin_cmd!("diff")
        .arg(&f1)
        .arg("-")
        .write_stdin("goodbye")
        .assert()
        .code(1)
        .stdout(format!("1c1\n< hello\n---\n> goodbye\n{}\n", NO_NEWLINE));
}

#[test]
fn unified_stdin_operand_uses_dash_label() {
    // GNU labels the stdin side `--- -` / `+++ -` in unified output.
    let dir = tempfile::tempdir().unwrap();
    let f2 = write_file(&dir, "f2.txt", "hello\n");

    cargo_bin_cmd!("diff")
        .arg("-u")
        .arg("-")
        .arg(&f2)
        .write_stdin("goodbye\n")
        .assert()
        .code(1)
        .stdout(predicates::str::contains("--- -\t"))
        .stdout(predicates::str::contains("\n+++ "))
        .stdout(predicates::str::contains("-goodbye\n+hello"));
}

#[test]
fn recursive_walk_reports_only_in_and_differences() {
    let dir = tempfile::tempdir().unwrap();
    let left = dir.path().join("left");
    let right = dir.path().join("right");
    fs::create_dir(&left).unwrap();
    fs::create_dir(&right).unwrap();
    fs::write(left.join("a.txt"), "1\n").unwrap();
    fs::write(right.join("a.txt"), "2\n").unwrap();
    fs::write(left.join("same.txt"), "same\n").unwrap();
    fs::write(right.join("same.txt"), "same\n").unwrap();
    fs::write(left.join("gone.txt"), "only-left\n").unwrap();
    fs::write(right.join("new.txt"), "only-right\n").unwrap();

    cargo_bin_cmd!("diff")
        .arg("-r")
        .arg(&left)
        .arg(&right)
        .assert()
        .code(1)
        .stdout(predicates::str::contains(format!(
            "diff -r {}/a.txt {}/a.txt\n1c1\n< 1\n---\n> 2\n",
            shown(&left),
            shown(&right)
        )))
        .stdout(predicates::str::contains(format!(
            "Only in {}: gone.txt\nOnly in {}: new.txt\n",
            shown(&left),
            shown(&right)
        )));
}

#[test]
fn recursive_walk_descends_into_subdirectories() {
    let dir = tempfile::tempdir().unwrap();
    let left = dir.path().join("l");
    let right = dir.path().join("r");
    fs::create_dir_all(left.join("sub")).unwrap();
    fs::create_dir_all(right.join("sub")).unwrap();
    fs::write(left.join("sub").join("f.txt"), "l1\n").unwrap();
    fs::write(right.join("sub").join("f.txt"), "l2\n").unwrap();

    cargo_bin_cmd!("diff")
        .arg("-r")
        .arg(&left)
        .arg(&right)
        .assert()
        .code(1)
        .stdout(predicates::str::contains(format!(
            "diff -r {}/sub/f.txt {}/sub/f.txt\n1c1\n< l1\n---\n> l2\n",
            shown(&left),
            shown(&right)
        )));
}

#[test]
fn non_recursive_walk_reports_common_subdirectories() {
    let dir = tempfile::tempdir().unwrap();
    let left = dir.path().join("l");
    let right = dir.path().join("r");
    fs::create_dir_all(left.join("sub")).unwrap();
    fs::create_dir_all(right.join("sub")).unwrap();
    fs::write(left.join("f.txt"), "same\n").unwrap();
    fs::write(right.join("f.txt"), "same\n").unwrap();

    cargo_bin_cmd!("diff")
        .arg(&left)
        .arg(&right)
        .assert()
        .code(0)
        .stdout(predicates::str::contains(format!(
            "Common subdirectories: {}/sub and {}/sub\n",
            shown(&left),
            shown(&right)
        )));
}

#[test]
fn walk_reports_type_mismatch_on_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let left = dir.path().join("t1");
    let right = dir.path().join("t2");
    fs::create_dir(&left).unwrap();
    fs::create_dir(&right).unwrap();
    fs::write(left.join("x"), "file\n").unwrap();
    fs::create_dir(right.join("x")).unwrap();

    cargo_bin_cmd!("diff")
        .arg("-r")
        .arg(&left)
        .arg(&right)
        .assert()
        .code(1)
        .stdout(predicates::str::contains(format!(
            "File {}/x is a regular file while file {}/x is a directory",
            shown(&left),
            shown(&right)
        )));
}

#[test]
fn file_against_directory_compares_same_name() {
    let dir = tempfile::tempdir().unwrap();
    let subdir = dir.path().join("dir");
    fs::create_dir(&subdir).unwrap();
    fs::write(dir.path().join("f.txt"), "outside\n").unwrap();
    fs::write(subdir.join("f.txt"), "inside\n").unwrap();

    cargo_bin_cmd!("diff")
        .arg(dir.path().join("f.txt"))
        .arg(&subdir)
        .assert()
        .code(1)
        .stdout(predicates::str::contains("1c1\n< outside\n---\n> inside\n"));
}
