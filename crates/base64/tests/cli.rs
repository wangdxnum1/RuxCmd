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

#[test]
fn multi_file_operands_behave_like_one_concatenated_input() {
    // Operands whose lengths are not multiples of 3 force the encoder's
    // 0-2 byte carry across chunk AND file boundaries; the output must
    // equal encoding the concatenation in one go (Group carry regression).
    let dir = std::env::temp_dir().join(format!(
        "ruxcmd-base64-carry-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    std::fs::write(dir.join("f1"), b"a").expect("write f1");
    std::fs::write(dir.join("f2"), b"bb").expect("write f2");
    std::fs::write(dir.join("f3"), b"cd").expect("write f3");

    let out = Command::new(env!("CARGO_BIN_EXE_base64"))
        .arg(dir.join("f1"))
        .arg(dir.join("f2"))
        .arg(dir.join("f3"))
        .output()
        .expect("failed to run base64");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // "abbcd": groups span f1|f2 (abb) and the tail of f2|f3 (cd).
    assert_eq!(out.stdout, b"YWJiY2Q=\n");

    let _ = std::fs::remove_dir_all(&dir);
}

/// Like `run`, but safe for inputs far larger than the OS pipe buffer: the
/// child's stdout is only drained by `wait_with_output`, so stdin must be
/// fed from a helper thread or both sides deadlock on full pipes.
fn run_streamed(args: &[&str], stdin_data: Vec<u8>) -> (Vec<u8>, Vec<u8>, Option<i32>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_base64"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn base64");
    let mut stdin = child.stdin.take().expect("stdin was not piped");
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(&stdin_data);
        // Dropping stdin here closes the pipe so the child sees EOF.
    });
    let out = child.wait_with_output().expect("failed to wait for base64");
    writer.join().expect("stdin writer panicked");
    (out.stdout, out.stderr, out.status.code())
}

/// xorshift64 stream for the large-input streaming regression tests.
fn xorshift_bytes(seed: u64, len: usize) -> Vec<u8> {
    let mut state = seed;
    (0..len)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8
        })
        .collect()
}

#[test]
fn five_megabyte_roundtrip_is_byte_identical() {
    let data = xorshift_bytes(0x9E3779B97F4A7C15, 5 * 1024 * 1024);
    // 5 MiB is not a multiple of 3, so the run also exercises the padded
    // final group on encode and its removal on decode.

    // Unwrapped: chunk boundaries fall inside the base64 stream.
    let (encoded, err, code) = run_streamed(&["-w", "0"], data.clone());
    assert_eq!(code, Some(0), "stderr: {}", String::from_utf8_lossy(&err));
    assert_eq!(encoded.len(), data.len().div_ceil(3) * 4);
    let (decoded, err, code) = run_streamed(&["-d"], encoded);
    assert_eq!(code, Some(0), "stderr: {}", String::from_utf8_lossy(&err));
    assert!(
        decoded == data,
        "decode(encode(data)) must be byte-identical, got {} of {} bytes",
        decoded.len(),
        data.len()
    );

    // Default 76-column wrapping interleaves newlines across chunk
    // boundaries, which the streaming decoder must skip.
    let (encoded, err, code) = run_streamed(&[], data.clone());
    assert_eq!(code, Some(0), "stderr: {}", String::from_utf8_lossy(&err));
    let (decoded, err, code) = run_streamed(&["-d"], encoded);
    assert_eq!(code, Some(0), "stderr: {}", String::from_utf8_lossy(&err));
    assert!(
        decoded == data,
        "wrapped roundtrip must be byte-identical, got {} of {} bytes",
        decoded.len(),
        data.len()
    );
}

#[test]
fn file_argument_roundtrips_large_input() {
    let data = xorshift_bytes(0xABCDEF0123456789, 5 * 1024 * 1024);
    let dir = std::env::temp_dir().join(format!(
        "ruxcmd-base64-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");

    let input = dir.join("big.bin");
    std::fs::write(&input, &data).expect("write input file");
    let out = Command::new(env!("CARGO_BIN_EXE_base64"))
        .arg("-w")
        .arg("0")
        .arg(&input)
        .output()
        .expect("failed to run base64 encode");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout.len(), data.len().div_ceil(3) * 4);

    let encoded = dir.join("big.b64");
    std::fs::write(&encoded, &out.stdout).expect("write encoded file");
    let out = Command::new(env!("CARGO_BIN_EXE_base64"))
        .arg("-d")
        .arg(&encoded)
        .output()
        .expect("failed to run base64 decode");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.stdout == data,
        "file roundtrip must be byte-identical, got {} of {} bytes",
        out.stdout.len(),
        data.len()
    );

    let _ = std::fs::remove_dir_all(&dir);
}
