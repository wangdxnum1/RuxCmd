use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn trailing_dot_and_dotdot_components() {
    cargo_bin_cmd!("basename")
        .arg("./")
        .assert()
        .success()
        .stdout(".\n");
    cargo_bin_cmd!("basename")
        .arg(".")
        .assert()
        .success()
        .stdout(".\n");
    cargo_bin_cmd!("basename")
        .arg("a/..")
        .assert()
        .success()
        .stdout("..\n");
    cargo_bin_cmd!("basename")
        .arg("..")
        .assert()
        .success()
        .stdout("..\n");
}

#[test]
fn root_paths() {
    cargo_bin_cmd!("basename")
        .arg("/")
        .assert()
        .success()
        .stdout("/\n");
    cargo_bin_cmd!("basename")
        .arg("//")
        .assert()
        .success()
        .stdout("/\n");
    // GNU strips trailing separators, so the drive root's basename is "C:"
    cargo_bin_cmd!("basename")
        .arg("C:/")
        .assert()
        .success()
        .stdout("C:\n");
}

#[test]
fn regular_names_and_suffix_unchanged() {
    cargo_bin_cmd!("basename")
        .arg("a/b")
        .assert()
        .success()
        .stdout("b\n");
    cargo_bin_cmd!("basename")
        .args(["foo.c", "-s", ".c"])
        .assert()
        .success()
        .stdout("foo\n");
}

/// POSIX two-operand form: `basename NAME SUFFIX` removes the trailing
/// suffix from the first operand's basename.
#[test]
fn two_operand_suffix_form() {
    cargo_bin_cmd!("basename")
        .args(["foo.c", ".c"])
        .assert()
        .success()
        .stdout("foo\n");
    cargo_bin_cmd!("basename")
        .args(["a/b.c", ".c"])
        .assert()
        .success()
        .stdout("b\n");
    cargo_bin_cmd!("basename")
        .args(["foo.c", ".txt"])
        .assert()
        .success()
        .stdout("foo.c\n");
    // A suffix that would consume the whole name is not removed (GNU).
    cargo_bin_cmd!("basename")
        .args(["foo.c", "foo.c"])
        .assert()
        .success()
        .stdout("foo.c\n");
}

/// Without -a more than two operands is an error (GNU: extra operand).
#[test]
fn extra_operand_without_a_is_an_error() {
    let output = cargo_bin_cmd!("basename")
        .args(["a", "b", "c"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8(output).unwrap();
    assert!(stderr.contains("extra operand"), "stderr was: {}", stderr);
}

/// -a treats every operand as a name, with an optional -s suffix.
#[test]
fn multiple_flag_treats_all_operands_as_names() {
    cargo_bin_cmd!("basename")
        .args(["-a", "foo.c", "bar.c"])
        .assert()
        .success()
        .stdout("foo.c\nbar.c\n");
    cargo_bin_cmd!("basename")
        .args(["-a", "-s", ".c", "foo.c", "bar.txt"])
        .assert()
        .success()
        .stdout("foo\nbar.txt\n");
}
