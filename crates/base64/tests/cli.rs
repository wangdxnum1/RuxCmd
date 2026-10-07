use std::io::Write;
use std::process::{Command, Stdio};

fn run(args: &[&str], stdin: &[u8]) -> (Vec<u8>, Vec<u8>, Option<i32>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_base64"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn base64");
    child
        .stdin
        .as_mut()
        .expect("stdin was not piped")
        .write_all(stdin)
        .expect("failed to write stdin");
    let out = child.wait_with_output().expect("failed to wait for base64");
    (out.stdout, out.stderr, out.status.code())
}

/// Small deterministic LCG so the roundtrip test needs no new dependency.
fn pseudo_random_bytes(seed: u64, len: usize) -> Vec<u8> {
    let mut state = seed;
    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as u8
        })
        .collect()
}

#[test]
fn decodes_padded_single_byte() {
    let (out, err, code) = run(&["-d"], b"QQ==");
    assert_eq!(code, Some(0), "stderr: {}", String::from_utf8_lossy(&err));
    assert_eq!(out, b"A");
}

#[test]
fn decodes_padded_hello() {
    let (out, err, code) = run(&["-d"], b"SGVsbG8=");
    assert_eq!(code, Some(0), "stderr: {}", String::from_utf8_lossy(&err));
    assert_eq!(out, b"Hello");
}

#[test]
fn decodes_unpadded_remainder_like_gnu() {
    // GNU coreutils 8.32 emits the decodable bytes, then reports
    // "base64: invalid input" and exits 1 because the input is not a
    // multiple of 4 chars. Only the bytes on stdout are the fix target.
    let (out, err, code) = run(&["-d"], b"QQ");
    assert_eq!(out, b"A");
    assert_eq!(code, Some(1));
    assert!(String::from_utf8_lossy(&err).contains("invalid input"));
}

#[test]
fn encodes_hello_with_default_wrap() {
    let (out, err, code) = run(&[], b"Hello");
    assert_eq!(code, Some(0), "stderr: {}", String::from_utf8_lossy(&err));
    assert_eq!(out, b"SGVsbG8=\n");
}

#[test]
fn roundtrips_random_bytes_unwrapped() {
    for len in [0usize, 1, 2, 3, 4, 5, 76, 255, 256, 257, 1000] {
        let data = pseudo_random_bytes(0x5eed + len as u64, len);
        let (encoded, err, code) = run(&["-w", "0"], &data);
        assert_eq!(
            code,
            Some(0),
            "encode len {len} stderr: {}",
            String::from_utf8_lossy(&err)
        );
        let (decoded, err, code) = run(&["-d"], &encoded);
        assert_eq!(
            code,
            Some(0),
            "decode len {len} stderr: {}",
            String::from_utf8_lossy(&err)
        );
        assert_eq!(decoded, data, "roundtrip failed for len {len}");
    }
}

#[test]
fn roundtrips_random_bytes_with_default_wrapping() {
    let data = pseudo_random_bytes(7, 500);
    let (encoded, err, code) = run(&[], &data);
    assert_eq!(code, Some(0), "stderr: {}", String::from_utf8_lossy(&err));
    let (decoded, err, code) = run(&["-d"], &encoded);
    assert_eq!(code, Some(0), "stderr: {}", String::from_utf8_lossy(&err));
    assert_eq!(decoded, data);
}

#[test]
fn invalid_character_emits_decoded_bytes_then_fails() {
    // GNU coreutils 8.32 emits every decodable byte (including the partial
    // group) before reporting the failure, so stdout must be flushed first.
    let (out, err, code) = run(&["-d"], b"SGVsbG8*");
    assert_eq!(code, Some(1));
    assert_eq!(out, b"Hello");
    assert!(String::from_utf8_lossy(&err).contains("invalid input"));
}
