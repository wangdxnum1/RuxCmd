//! Behavior tests for the Round 12 G1 wget P1 fixes.
//!
//! Covers the filename derivation fixes (trailing "/" used to produce an
//! empty file name and an fs::write failure), query-string stripping and
//! percent decoding, the 10s connect timeout (constructor only), GNU exit
//! code 8 on HTTP errors, and -O - writing to stdout.
//!
//! All requests go to a loopback TCP listener (127.0.0.1), so the suite
//! stays network-free.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn wget_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_wget"))
        .args(args)
        .current_dir(dir)
        // reqwest honors the Windows registry system proxy; force loopback
        // traffic direct so requests reach the local test servers. (A "*"
        // entry is not enough: hyper-util matches IP hosts only against
        // explicit IP entries.)
        .env("NO_PROXY", "127.0.0.1")
        .env("no_proxy", "127.0.0.1")
        .output()
        .expect("failed to run wget")
}

/// Serve exactly one HTTP response built from `status_line` + `body`.
fn serve_once(status_line: &str, body: &str) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let addr = listener.local_addr().expect("local addr");
    let response = format!(
        "{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 8192];
            let _ = stream.read(&mut buf);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    addr
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ruxcmd-wget-test-{}-{}-{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn saves_file_named_after_url_path() {
    let addr = serve_once("HTTP/1.1 200 OK", "<html>ok</html>");
    let dir = temp_dir("named");
    let out = wget_in(&dir, &[&format!("http://{addr}/page.html")]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    let saved = dir.join("page.html");
    let content = std::fs::read(&saved).expect("page.html must exist");
    assert_eq!(content, b"<html>ok</html>");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn trailing_slash_saves_index_html() {
    let addr = serve_once("HTTP/1.1 200 OK", "<html>root</html>");
    let dir = temp_dir("index");
    let out = wget_in(&dir, &[&format!("http://{addr}/")]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    let content = std::fs::read(dir.join("index.html")).expect("index.html must exist");
    assert_eq!(content, b"<html>root</html>");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn query_string_is_stripped_from_filename() {
    let addr = serve_once("HTTP/1.1 200 OK", "data");
    let dir = temp_dir("query");
    let out = wget_in(&dir, &[&format!("http://{addr}/file.txt?a=1&b=2")]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    let content = std::fs::read(dir.join("file.txt")).expect("file.txt must exist");
    assert_eq!(content, b"data");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn percent_escaped_filename_is_decoded() {
    let addr = serve_once("HTTP/1.1 200 OK", "spaces");
    let dir = temp_dir("percent");
    let out = wget_in(&dir, &[&format!("http://{addr}/file%20name.txt")]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    let content = std::fs::read(dir.join("file name.txt")).expect("decoded file must exist");
    assert_eq!(content, b"spaces");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn output_document_dash_writes_to_stdout() {
    let addr = serve_once("HTTP/1.1 200 OK", "streamed-body");
    let dir = temp_dir("stdout");
    // With -q the body is the only stdout output.
    let out = wget_in(&dir, &["-q", "-O", "-", &format!("http://{addr}/anything")]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    assert_eq!(String::from_utf8_lossy(&out.stdout), "streamed-body");
    assert!(
        dir.read_dir().expect("readdir").next().is_none(),
        "no file must be created for -O -"
    );

    // Without -q progress messages share stdout, but the body is still there.
    let addr2 = serve_once("HTTP/1.1 200 OK", "streamed-body");
    let out = wget_in(&dir, &["-O", "-", &format!("http://{addr2}/anything")]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("streamed-body"), "stdout: {stdout:?}");
    assert!(stdout.contains("Saving to: 'STDOUT'"), "stdout: {stdout:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn http_error_exits_8() {
    let addr = serve_once("HTTP/1.1 404 Not Found", "gone");
    let dir = temp_dir("err8");
    let out = wget_in(&dir, &[&format!("http://{addr}/missing.html")]);
    assert_eq!(out.status.code(), Some(8), "stderr: {:?}", out.stderr);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("404"), "stderr: {stderr:?}");
    assert!(
        dir.read_dir().expect("readdir").next().is_none(),
        "no file must be created on HTTP error"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// Real-request acceptance test (requires network; run manually, recorded in
// the Round 12 ledger).
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires network; manual acceptance for Round 12 G1"]
fn real_download_exits_zero_and_writes_file() {
    let dir = temp_dir("real");
    let out = wget_in(&dir, &["-q", "https://example.com/"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    let content = std::fs::read(dir.join("index.html")).expect("index.html must exist");
    assert!(!content.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
