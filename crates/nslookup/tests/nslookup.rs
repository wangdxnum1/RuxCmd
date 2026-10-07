//! Regression tests for P0 #47: MX/TXT/CNAME output used to be fabricated
//! (`<name> mail exchanger = 10 <name>`, constant "(no TXT records)",
//! `<name> canonical name = <name>`), the SERVER positional argument was
//! silently ignored, and record type names were case-sensitive.

use assert_cmd::cargo::cargo_bin_cmd;

fn run(args: &[&str]) -> std::process::Output {
    cargo_bin_cmd!("nslookup")
        .args(args)
        .output()
        .expect("failed to run nslookup")
}

/// Markers that only ever appeared in the fabricated output.
const FORGED_MARKERS: [&str; 4] = [
    "mail exchanger = 10",
    "canonical name =",
    "text =",
    "(no TXT records)",
];

#[test]
fn unsupported_types_fail_without_fabricated_records() {
    for qtype in ["NS", "SOA", "SRV", "ns"] {
        let output = run(&["-t", qtype, "example.com"]);
        assert_eq!(
            output.status.code(),
            Some(1),
            "type {qtype}: expected exit 1, stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        for marker in FORGED_MARKERS {
            assert!(
                !stdout.contains(marker),
                "type {qtype}: stdout contains forged marker {marker:?}: {stdout:?}"
            );
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("unsupported query type"),
            "type {qtype}: stderr: {stderr:?}"
        );
    }
}

/// `-t a` must behave exactly like `-t A`. `localhost` resolves from the
/// hosts file, so this stays network-free. Before the fix `-t a` was rejected
/// with exit 1 while `-t A` performed the lookup.
#[test]
fn type_flag_is_case_insensitive() {
    let lower = run(&["-t", "a", "localhost"]);
    let upper = run(&["-t", "A", "localhost"]);
    assert_eq!(
        lower.status.code(),
        upper.status.code(),
        "exit codes differ: -t a -> {:?} ({}) vs -t A -> {:?} ({})",
        lower.status.code(),
        String::from_utf8_lossy(&lower.stderr),
        upper.status.code(),
        String::from_utf8_lossy(&upper.stderr),
    );
    assert_eq!(
        String::from_utf8_lossy(&lower.stdout),
        String::from_utf8_lossy(&upper.stdout),
        "stdout differs between -t a and -t A"
    );
    assert_eq!(upper.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&upper.stdout).contains("Address:"));
}

/// The SERVER positional argument was declared but never used; after its
/// removal an extra positional must be a clap usage error (exit 2), never a
/// silently ignored argument.
#[test]
fn positional_server_argument_is_rejected() {
    let output = run(&["example.com", "8.8.8.8"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "SERVER positional should be rejected by clap, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unexpected argument"),
        "stderr: {stderr:?}"
    );
}

// ---------------------------------------------------------------------------
// Real-query acceptance tests (require DNS/network; run manually, recorded in
// the P0 remediation ledger).
// ---------------------------------------------------------------------------

#[test]
#[ignore = "requires DNS/network; manual acceptance for P0 #47"]
fn real_mx_query_returns_true_records() {
    let output = run(&["-t", "MX", "example.com"]);
    assert_eq!(output.status.code(), Some(0),
        "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("mail exchanger"), "stdout: {stdout:?}");
    assert_ne!(
        stdout.trim(),
        "example.com mail exchanger = 10 example.com",
        "MX output must not be the old forged literal echo"
    );
}

#[test]
#[ignore = "requires DNS/network; manual acceptance for P0 #47"]
fn real_txt_and_cname_queries_return_true_records() {
    let txt = run(&["-t", "TXT", "example.com"]);
    assert_eq!(txt.status.code(), Some(0),
        "stderr: {}", String::from_utf8_lossy(&txt.stderr));
    let txt_out = String::from_utf8_lossy(&txt.stdout);
    assert!(txt_out.contains("text ="), "stdout: {txt_out:?}");
    assert!(
        !txt_out.contains("(no TXT records)"),
        "stdout still contains the old forged constant: {txt_out:?}"
    );

    let cname = run(&["-t", "CNAME", "www.microsoft.com"]);
    assert_eq!(cname.status.code(), Some(0),
        "stderr: {}", String::from_utf8_lossy(&cname.stderr));
    let cname_out = String::from_utf8_lossy(&cname.stdout);
    assert!(cname_out.contains("canonical name"), "stdout: {cname_out:?}");
    assert_ne!(
        cname_out.trim(),
        "www.microsoft.com canonical name = www.microsoft.com",
        "CNAME output must not be the old forged identity echo"
    );
}

#[test]
#[ignore = "requires DNS/network; manual acceptance for P0 #47"]
fn mx_query_for_missing_name_exits_nonzero() {
    let output = run(&["-t", "MX", "nonexistent-name-ruxcmd-p0-47.invalid"]);
    assert_ne!(
        output.status.code(),
        Some(0),
        "missing name must not exit 0: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}
