use assert_cmd::cargo::cargo_bin_cmd;

// Regression for P0 #27: in table mode without an explicit -s, every separator
// character produced its own split, so `a  b` (double space) yielded three
// columns with an empty middle one. GNU column treats a run of whitespace as a
// single separator.

#[test]
fn table_mode_collapses_whitespace_runs_by_default() {
    cargo_bin_cmd!("column")
        .args(["-t"])
        .write_stdin("a  b\n1  2\n")
        .assert()
        .success()
        .stdout("a  b\n1  2\n");
}

#[test]
fn table_mode_collapses_tab_runs_and_trims_edges() {
    cargo_bin_cmd!("column")
        .args(["-t"])
        .write_stdin(" \ta\t\tb \t\n1\t2\n")
        .assert()
        .success()
        .stdout("a  b\n1  2\n");
}

#[test]
fn explicit_separator_keeps_empty_columns() {
    cargo_bin_cmd!("column")
        .args(["-t", "-s,"])
        .write_stdin("a,,b\n1,2,3\n")
        .assert()
        .success()
        .stdout("a     b\n1  2  3\n");
}

// Declared deviation: an explicit -s splits per character without collapsing,
// even when it names the default whitespace set.
#[test]
fn explicit_whitespace_separator_does_not_collapse() {
    cargo_bin_cmd!("column")
        .args(["-t", "-s", " \t"])
        .write_stdin("a  b\n")
        .assert()
        .success()
        .stdout("a    b\n");
}
