//! Behavior tests for the Round 12 G1 curl P1 fixes.
//!
//! Default output is body-only (headers only with -i), -d implies POST,
//! redirects are NOT followed without -L (reqwest used to follow by
//! default), -L follows up to 10 hops manually, -f exits 22 without body
//! output on >= 400, and -I issues HEAD.
//!
//! All networked behavior runs against a loopback TCP listener (127.0.0.1),
//! so the non-ignored suite stays network-free.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::process::{Command, Output};

fn curl(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_curl"))
        .args(args)
        // reqwest honors the Windows registry system proxy; force loopback
        // traffic direct so requests reach the local test servers. (A "*"
        // entry is not enough: hyper-util matches IP hosts only against
        // explicit IP entries.)
        .env("NO_PROXY", "127.0.0.1")
        .env("no_proxy", "127.0.0.1")
        .output()
        .expect("failed to run curl")
}

/// Accept `hops` sequential connections on 127.0.0.1 and answer each with
/// `responder(hop_index, request_bytes)`. Connection reuse is avoided by
/// always sending `Connection: close`.
fn serve(hops: usize, responder: impl Fn(usize, &[u8]) -> String + Send + 'static) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let addr = listener.local_addr().expect("local addr");
    std::thread::spawn(move || {
        for hop in 0..hops {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            let mut buf = [0u8; 8192];
            let read = stream.read(&mut buf).unwrap_or(0);
            let response = responder(hop, &buf[..read]);
            let _ = stream.write_all(response.as_bytes());
        }
    });
    addr
}

fn body_response(status_line: &str, body: &str) -> String {
    format!(
        "{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

#[test]
fn default_output_is_body_only() {
    let addr = serve(1, |_, _| {
        body_response("HTTP/1.1 200 OK", "hello")
            .replace("Content-Length", "X-Test: 1\r\nContent-Length")
    });
    let out = curl(&[&format!("http://{addr}/")]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout, "hello", "body must be the only output");
    assert!(!stdout.contains("HTTP/1.1"), "no status line by default");
    assert!(!stdout.contains("x-test"), "no headers by default");
}

#[test]
fn include_flag_prints_status_line_and_headers() {
    let addr = serve(1, |_, _| {
        body_response("HTTP/1.1 200 OK", "hello")
            .replace("Content-Length", "X-Test: 1\r\nContent-Length")
    });
    let out = curl(&["-i", &format!("http://{addr}/")]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("HTTP/1.1 200 OK"), "stdout: {stdout:?}");
    assert!(stdout.contains("x-test: 1"), "stdout: {stdout:?}");
    assert!(stdout.contains("hello"), "stdout: {stdout:?}");
}

/// First line of the request echoed back as the body.
fn method_echo_server() -> SocketAddr {
    serve(1, |_, request| {
        let first = String::from_utf8_lossy(request);
        let method = first
            .lines()
            .next()
            .unwrap_or("")
            .split(' ')
            .next()
            .unwrap_or("");
        let method = method.trim().to_string();
        body_response("HTTP/1.1 200 OK", &method)
    })
}

#[test]
fn data_implies_post_without_explicit_method() {
    let addr = method_echo_server();
    let out = curl(&["-d", "k=v", &format!("http://{addr}/")]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    assert_eq!(String::from_utf8_lossy(&out.stdout), "POST");
}

#[test]
fn explicit_request_overrides_data_implied_post() {
    let addr = method_echo_server();
    let out = curl(&["-X", "PUT", "-d", "k=v", &format!("http://{addr}/")]);
    assert_eq!(String::from_utf8_lossy(&out.stdout), "PUT");
}

#[test]
fn head_flag_sends_head_request() {
    // The method is echoed in a header because a HEAD response has no body.
    let addr = serve(1, |_, request| {
        let first = String::from_utf8_lossy(request);
        let method = first
            .lines()
            .next()
            .unwrap_or("")
            .split(' ')
            .next()
            .unwrap_or("");
        let method = method.trim().to_string();
        format!(
            "HTTP/1.1 200 OK\r\nX-Method: {method}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )
    });
    let out = curl(&["-I", "-i", &format!("http://{addr}/")]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("x-method: HEAD"), "stdout: {stdout:?}");
}

#[test]
fn fail_flag_exits_22_without_body_on_http_error() {
    let addr = serve(2, |_, _| body_response("HTTP/1.1 404 Not Found", "nope"));
    let url = format!("http://{addr}/");

    let out = curl(&["-f", &url]);
    assert_eq!(out.status.code(), Some(22), "stderr: {:?}", out.stderr);
    assert!(out.stdout.is_empty(), "-f must not output the error body");

    // Without -f the body is printed and the exit code stays 0.
    let out = curl(&[&url]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "nope");
}

#[test]
fn redirects_are_not_followed_by_default() {
    let addr = serve(1, |_, _| {
        body_response("HTTP/1.1 302 Found", "moved")
            .replace("Content-Length", "Location: /next\r\nContent-Length")
    });
    let out = curl(&[&format!("http://{addr}/")]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "following the (dead) redirect target would fail; stderr: {:?}",
        out.stderr
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "moved");
}

#[test]
fn location_flag_follows_redirect_chain() {
    let addr = serve(2, |hop, _| {
        if hop == 0 {
            body_response("HTTP/1.1 302 Found", "")
                .replace("Content-Length", "Location: /next\r\nContent-Length")
        } else {
            body_response("HTTP/1.1 200 OK", "final")
        }
    });
    let out = curl(&["-L", &format!("http://{addr}/")]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    assert_eq!(String::from_utf8_lossy(&out.stdout), "final");
}

#[test]
fn location_flag_stops_after_ten_redirects() {
    let addr = serve(11, |_, _| {
        body_response("HTTP/1.1 302 Found", "")
            .replace("Content-Length", "Location: /next\r\nContent-Length")
    });
    let out = curl(&["-L", &format!("http://{addr}/")]);
    assert_eq!(out.status.code(), Some(1), "stderr: {:?}", out.stderr);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("redirect"), "stderr: {stderr:?}");
}

#[test]
fn silent_mode_suppresses_error_messages() {
    // Bind a loopback port and release it again: nothing is listening there,
    // so the connection must be refused (port 1 cannot be used because some
    // systems have services on low ports).
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let addr = listener.local_addr().expect("local addr");
    drop(listener);

    let url = format!("http://{addr}/");

    let out = curl(&["-s", &url]);
    assert_ne!(out.status.code(), Some(0));
    assert!(
        out.stderr.is_empty(),
        "-s must suppress error messages: {:?}",
        out.stderr
    );

    let out = curl(&[&url]);
    assert_ne!(out.status.code(), Some(0));
    assert!(
        !out.stderr.is_empty(),
        "without -s the error must be reported"
    );
}

// ---------------------------------------------------------------------------
// Real-request acceptance tests (require network; run manually, recorded in
// the Round 12 ledger).
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires network; manual acceptance for Round 12 G1"]
fn real_default_body_only() {
    let out = curl(&["https://example.com/"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.to_ascii_lowercase().contains("doctype html"),
        "stdout: {stdout:?}"
    );
    assert!(!stdout.contains("HTTP/1.1"), "no header dump: {stdout:?}");
}

#[test]
#[ignore = "requires network; manual acceptance for Round 12 G1"]
fn real_include_prints_headers() {
    let out = curl(&["-i", "https://example.com/"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("HTTP/"), "stdout: {stdout:?}");
    assert!(
        stdout.to_ascii_lowercase().contains("doctype html"),
        "stdout: {stdout:?}"
    );
}

#[test]
#[ignore = "requires network; manual acceptance for Round 12 G1"]
fn real_fail_exits_22_on_http_error() {
    let out = curl(&["-f", "https://httpbin.org/status/418"]);
    assert_eq!(out.status.code(), Some(22), "stderr: {:?}", out.stderr);
    assert!(out.stdout.is_empty(), "stdout: {:?}", out.stdout);
}

#[test]
#[ignore = "requires network; manual acceptance for Round 12 G1"]
fn real_location_follows_redirects() {
    let out = curl(&["-L", "https://httpbin.org/redirect/3"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    assert!(!out.stdout.is_empty());

    let out = curl(&["https://httpbin.org/redirect/3"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
}

#[test]
#[ignore = "requires network; manual acceptance for Round 12 G1"]
fn real_data_posts() {
    let out = curl(&["-d", "k=v", "https://httpbin.org/post"]);
    assert_eq!(out.status.code(), Some(0), "stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"k=v\""), "stdout: {stdout:?}");
}
