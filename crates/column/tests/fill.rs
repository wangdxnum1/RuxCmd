//! Fill mode (no -t) parity with GNU column from util-linux 2.40.2
//! ("C:\Program Files\Git\usr\bin\column.exe"). Every expected output below
//! was recorded from that binary's byte output on the same fixed input, run
//! with COLUMNS=80 (GNU reads the terminal size on stdout, then the COLUMNS
//! environment variable, then defaults to 80; cargo tests run with pipes so
//! COLUMNS decides).
//!
//! Measured semantics: every line is one entry (verbatim, leading and
//! trailing whitespace kept); empty and whitespace-only lines are skipped;
//! entries fill columns top-to-bottom before starting a new row; column
//! widths are the longest entry rounded up to a multiple of 8 and columns are
//! aligned with TABs (one tab per tab stop, last column unpadded); an entry
//! at least as wide as the terminal switches to "simple" mode (one entry per
//! line, verbatim).

use assert_cmd::cargo::cargo_bin_cmd;

fn fill(input: impl Into<Vec<u8>>) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("column")
        .env("COLUMNS", "80")
        .write_stdin(input)
        .assert()
}

// ① Entries (one per line) fill multiple tab-aligned columns. GNU fills
//    columns top-to-bottom: entries 0,2,4,... form row 1, 1,3,5,... row 2.
#[test]
fn fill_arranges_lines_into_tab_columns() {
    fill("a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\nl\n")
        .success()
        .stdout("a\tc\te\tg\ti\tk\nb\td\tf\th\tj\tl\n");
}

// Column cell width is the longest entry rounded up to the next multiple of
// 8; gaps are TABs, the last column is not padded.
#[test]
fn fill_aligns_columns_on_tab_stops() {
    fill("aaaa\nbb\ncccccccccc\ndd\n")
        .success()
        .stdout("aaaa\t\tbb\t\tcccccccccc\tdd\n");
}

#[test]
fn fill_wraps_into_multiple_rows() {
    let row = "aaaaa\taaaaa\taaaaa\taaaaa\taaaaa\taaaaa\n";
    fill("aaaaa\n".repeat(12)).success().stdout(row.repeat(2));
}

#[test]
fn fill_respects_narrow_columns_env() {
    cargo_bin_cmd!("column")
        .env("COLUMNS", "20")
        .write_stdin("a\nb\nc\n")
        .assert()
        .success()
        .stdout("a\tc\nb\n");
}

// ② An entry at least as wide as the terminal is printed verbatim on its own
//    line (GNU switches to "simple" mode).
#[test]
fn fill_prints_oversized_entry_verbatim() {
    let long = "x".repeat(100);
    fill(format!("short\n{long}\nafter\n"))
        .success()
        .stdout(format!("short\n{long}\nafter\n"));
}

#[test]
fn fill_prints_entry_at_exact_width_verbatim() {
    let long = "y".repeat(80);
    fill(format!("{long}\nz\n"))
        .success()
        .stdout(format!("{long}\nz\n"));
}

// ③ Empty lines: GNU ignores them (and whitespace-only lines) rather than
//    starting a new row or column. Entries from before and after an empty
//    line share the same row.
#[test]
fn fill_ignores_empty_lines() {
    fill("aaa bbb\nccc ddd\n\neee fff\nggg\n")
        .success()
        .stdout("aaa bbb\tccc ddd\teee fff\tggg\n");
}

#[test]
fn fill_ignores_whitespace_only_lines() {
    fill("a\n   \n\t\t\nb\n").success().stdout("a\tb\n");
}

#[test]
fn fill_of_only_empty_lines_is_empty() {
    fill("\n\n   \n").success().stdout("");
}

// Entry text is kept verbatim, including leading whitespace and internal
// whitespace (GNU width() gives '\t' zero cells).
#[test]
fn fill_keeps_leading_whitespace() {
    fill("  abc\nxy\n").success().stdout("  abc\txy\n");
}

#[test]
fn fill_keeps_internal_whitespace() {
    fill("x\ty\nz\n").success().stdout("x\ty\tz\n");
}

// The '\r' of CRLF input is part of the entry (GNU getline strips only '\n')
// and counts as zero cells.
#[test]
fn fill_keeps_carriage_return_of_crlf_input() {
    fill("abc\r\ndef\n").success().stdout("abc\r\tdef\n");
}

// Non-ASCII cells: CJK counts as 2 cells, matching GNU wcwidth.
#[test]
fn fill_counts_cjk_as_two_cells() {
    fill("中文测试\nab\n中\n")
        .success()
        .stdout("中文测试\tab\t\t中\n");
}

#[test]
fn fill_of_empty_input_is_empty() {
    fill("").success().stdout("");
}

// Entries accumulate across all file arguments.
#[test]
fn fill_concatenates_files_in_order() {
    let dir = tempfile::tempdir().unwrap();
    let f1 = dir.path().join("one.txt");
    let f2 = dir.path().join("two.txt");
    std::fs::write(&f1, b"a\nb\nc\n").unwrap();
    std::fs::write(&f2, b"d\ne\n").unwrap();
    cargo_bin_cmd!("column")
        .env("COLUMNS", "80")
        .arg(&f1)
        .arg(&f2)
        .assert()
        .success()
        .stdout("a\tb\tc\td\te\n");
}

// A file that cannot be opened is reported on stderr, the remaining input is
// still columnated, and the exit code is 1.
#[test]
fn fill_reports_missing_file_and_still_prints() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.txt");
    let f1 = dir.path().join("one.txt");
    std::fs::write(&f1, b"a\nb\nc\n").unwrap();
    cargo_bin_cmd!("column")
        .env("COLUMNS", "80")
        .arg(&missing)
        .arg(&f1)
        .assert()
        .failure()
        .stdout("a\tb\tc\n");
}

// ④ Table mode is unaffected: same input, different output separator and
//    layout.
#[test]
fn table_mode_still_formats_columns() {
    cargo_bin_cmd!("column")
        .env("COLUMNS", "80")
        .args(["-t"])
        .write_stdin("aaa bbb\nccc ddd\n")
        .assert()
        .success()
        .stdout("aaa  bbb\nccc  ddd\n");
}

#[test]
fn fill_mode_uses_tabs_not_two_spaces() {
    fill("aaa bbb\nccc ddd\n")
        .success()
        .stdout("aaa bbb\tccc ddd\n");
}
