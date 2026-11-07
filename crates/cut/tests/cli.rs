//! P1 regression tests (Round 12 D1): cut must be byte-faithful —
//! non-UTF-8 data passes through, -b emits raw bytes, and trailing
//! spaces survive (only \r is stripped from CRLF lines).

use std::path::PathBuf;
use std::process::Command;

fn cut(args: &[&str], file: &std::path::Path) -> (Vec<u8>, Option<i32>) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cut"));
    for arg in args {
        cmd.arg(arg);
    }
    cmd.arg(file);
    let output = cmd.output().expect("cut runs");
    (output.stdout, output.status.code())
}

fn write_file(name: &str, data: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ruxcmd-cut-test-{}-{}", std::process::id(), name));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(name);
    std::fs::write(&path, data).unwrap();
    path
}

// --- P1: -b selects raw bytes (no as-char re-encoding) ---

#[test]
fn multibyte_bytes_selection_outputs_raw_bytes() {
    // "你好世界\n" — 12 bytes of UTF-8 plus newline; -b 1-3 must yield the
    // first three raw bytes (exactly the encoding of 你).
    let path = write_file("utf8.txt", "你好世界\n".as_bytes());
    let (stdout, code) = cut(&["-b", "1-3"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, "你\n".as_bytes());
}

#[test]
fn non_utf8_bytes_pass_through() {
    // A line that is not valid UTF-8 must survive selection untouched.
    let path = write_file("bin.txt", b"abc\xFFdef\nplain\n");
    let (stdout, code) = cut(&["-b", "2-4"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"bc\xFF\nlai\n");
}

// --- P1: non-UTF-8 field selection is not silently truncated ---

#[test]
fn non_utf8_field_selection_still_sees_later_lines() {
    // Before the fix, read_line errors swallowed the rest of the input;
    // the second line must still be processed.
    let path = write_file("bin2.txt", b"caf\xFF,x\nplain,z\n");
    let (stdout, code) = cut(&["-d", ",", "-f", "2"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"x\nz\n");
}

// --- P1: trailing spaces are preserved (only \r is stripped) ---

#[test]
fn trailing_spaces_are_preserved() {
    let path = write_file("spaces.txt", b"hello   \nworld \n");
    let (stdout, code) = cut(&[], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"hello   \nworld \n");
}

#[test]
fn crlf_is_stripped_but_trailing_tabs_kept() {
    let path = write_file("crlf.txt", b"a,b\r\nc,d\r\n");
    let (stdout, code) = cut(&["-d", ",", "-f", "1"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"a\nc\n");
}

// --- --complement: invert the selection set ---

#[test]
fn complement_fields_drops_the_selected_field() {
    // echo "hello world" | cut --complement -d ' ' -f 1  =>  "world"
    let path = write_file("comp-fields.txt", b"hello world\n");
    let (stdout, code) = cut(&["--complement", "-d", " ", "-f", "1"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"world\n");
}

#[test]
fn complement_fields_keep_position_order() {
    // Complement output follows line order, not the order in the list.
    let path = write_file("comp-order.txt", b"a,b,c,d\n");
    let (stdout, code) = cut(&["--complement", "-d", ",", "-f", "2,3"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"a,d\n");
}

#[test]
fn complement_fields_with_ranges_and_lists() {
    let path = write_file("comp-ranges.txt", b"a,b,c,d\n");
    let (stdout, code) = cut(&["--complement", "-d", ",", "-f", "1,3-"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"b\n");
}

#[test]
fn complement_bytes_selection() {
    let path = write_file("comp-bytes.txt", b"abcdef\n");
    let (stdout, code) = cut(&["--complement", "-b", "1-3"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"def\n");
}

#[test]
fn complement_characters_on_multibyte_text() {
    let path = write_file("comp-chars.txt", "你好世界\n".as_bytes());
    let (stdout, code) = cut(&["--complement", "-c", "1"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, "好世界\n".as_bytes());
}

#[test]
fn complement_field_mode_passes_delimiterless_lines_through() {
    let path = write_file("comp-plain.txt", b"plain\n");
    let (stdout, code) = cut(&["--complement", "-d", ",", "-f", "1"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"plain\n");
}

// --- 0.5.1: -n with -b never splits multibyte characters ---------------------
//
// "你好世界" is four 3-byte UTF-8 characters: 你 at bytes 1-3, 好 at 4-6,
// 世 at 7-9, 界 at 10-12. GNU coreutils 8.32 (the local reference) documents
// -n as "(ignored)" — its -b 1-4 -n output still carries a partial 好 — so
// these tests pin the POSIX-documented semantics instead: a byte selection
// point that falls inside a character extends to the whole character.

#[test]
fn n_with_byte_range_aligned_to_character_outputs_whole_char() {
    // echo 你好世界 | cut -b 1-3 -n  =>  你 (3 bytes = 1 complete character).
    let path = write_file("n-aligned.txt", "你好世界\n".as_bytes());
    let (stdout, code) = cut(&["-b", "1-3", "-n"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, "你\n".as_bytes());
}

#[test]
fn n_without_flag_keeps_raw_byte_selection() {
    // Without -n the selection stays raw bytes: -b 1-4 emits the first four
    // bytes (你 + 好's first byte 0xE5), cutting 好 in half (pre-existing
    // behavior).
    let path = write_file("n-absent.txt", "你好世界\n".as_bytes());
    let (stdout, code) = cut(&["-b", "1-4"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, &[0xE4, 0xBD, 0xA0, 0xE5, b'\n']);
}

#[test]
fn n_extends_end_point_forward_to_character_end() {
    // -b 1-4 -n: byte 4 is 好's first byte, so the range extends to the
    // whole character => 你好.
    let path = write_file("n-end.txt", "你好世界\n".as_bytes());
    let (stdout, code) = cut(&["-b", "1-4", "-n"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, "你好\n".as_bytes());
}

#[test]
fn n_extends_start_point_back_to_character_start() {
    // -b 2-3 -n: both bytes sit inside 你, so the whole character is output.
    // -b 2 -n behaves the same for the same reason.
    let path = write_file("n-start.txt", "你好世界\n".as_bytes());
    let (stdout, code) = cut(&["-b", "2-3", "-n"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, "你\n".as_bytes());
    let (stdout, code) = cut(&["-b", "2", "-n"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, "你\n".as_bytes());
}

#[test]
fn n_does_not_extend_beyond_an_aligned_end_point() {
    // -b 4-6 -n selects exactly 好; the extension must not swallow 世.
    let path = write_file("n-exact.txt", "你好世界\n".as_bytes());
    let (stdout, code) = cut(&["-b", "4-6", "-n"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, "好\n".as_bytes());
}

#[test]
fn n_mixed_ascii_and_multibyte_expands_only_the_multibyte_char() {
    // "a你b": -b 1-3 covers `a` plus two bytes of 你; only 你 is expanded.
    let path = write_file("n-mixed.txt", "a你b\n".as_bytes());
    let (stdout, code) = cut(&["-b", "1-3", "-n"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, "a你\n".as_bytes());
}

#[test]
fn n_ascii_selection_is_unchanged() {
    // echo hello | cut -b 1-3 -n  =>  hel (ASCII has no characters to split).
    let path = write_file("n-ascii.txt", b"hello\n");
    let (stdout, code) = cut(&["-b", "1-3", "-n"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"hel\n");
}

#[test]
fn n_falls_back_to_bytes_on_non_utf8_input() {
    // Input that is not valid UTF-8 keeps the raw byte selection (same
    // output as without -n).
    let path = write_file("n-bin.txt", b"ab\xFFcd\n");
    let (stdout, code) = cut(&["-b", "1-3", "-n"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"ab\xFF\n");
}

#[test]
fn n_complement_selects_whole_characters() {
    // The -n mapping applies before --complement, so the complement of a
    // partial character is the remaining whole characters: -b 2 -n
    // --complement on 你好世界 => 好世界.
    let path = write_file("n-comp.txt", "你好世界\n".as_bytes());
    let (stdout, code) = cut(&["-b", "2", "-n", "--complement"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, "好世界\n".as_bytes());
}

#[test]
fn n_is_ignored_for_character_and_field_selection() {
    // -c already selects whole characters and -f does not split them, so
    // -n must not change either.
    let path = write_file("n-other-modes.txt", "你好世界\n".as_bytes());
    let (stdout, code) = cut(&["-c", "1-2", "-n"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, "你好\n".as_bytes());
    let path = write_file("n-fields.txt", b"a,b\n");
    let (stdout, code) = cut(&["-d", ",", "-f", "1", "-n"], &path);
    assert_eq!(code, Some(0));
    assert_eq!(stdout, b"a\n");
}
