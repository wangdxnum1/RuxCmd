use assert_cmd::cargo::cargo_bin_cmd;
use std::process::Output;

// P0 #41 regression. The old implementation fabricated the groups list from
// USERNAME/COMPUTERNAME env vars plus hardcoded "Everyone"/"Authenticated
// Users", and printed the account *name* (or DOMAIN\user) instead of a numeric
// RID for uid/gid. The assertions below require the GNU format with numeric
// RIDs, which the old output could not satisfy.

fn run_id(args: &[&str]) -> Output {
    cargo_bin_cmd!("id").args(args).output().expect("id runs")
}

fn stdout_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// One `id` list entry: `123` or `123(Name)`.
fn is_rid_with_optional_name(entry: &str) -> bool {
    let entry = entry.trim();
    match entry.find('(') {
        Some(idx) => {
            let (rid, rest) = entry.split_at(idx);
            !rid.is_empty()
                && rid.chars().all(|c| c.is_ascii_digit())
                && rest.len() >= 2
                && rest.starts_with('(')
                && rest.ends_with(')')
        }
        None => !entry.is_empty() && entry.chars().all(|c| c.is_ascii_digit()),
    }
}

#[test]
fn default_output_is_gnu_format_with_numeric_ids() {
    let out = run_id(&[]);
    assert!(out.status.success());
    let stdout = stdout_str(&out);
    assert!(stdout.ends_with('\n'), "output must end with a newline");
    let line = stdout.trim_end();
    let (uid_part, rest) = line
        .split_once(" gid=")
        .unwrap_or_else(|| panic!("missing ' gid=' separator in {line:?}"));
    let (gid_part, groups_part) = rest
        .split_once(" groups=")
        .unwrap_or_else(|| panic!("missing ' groups=' separator in {line:?}"));
    let uid = uid_part
        .strip_prefix("uid=")
        .unwrap_or_else(|| panic!("missing uid= prefix in {uid_part:?}"));
    assert!(
        is_rid_with_optional_name(uid),
        "uid must be a numeric RID with optional (name): {uid:?}"
    );
    assert!(
        is_rid_with_optional_name(gid_part),
        "gid must be a numeric RID with optional (name): {gid_part:?}"
    );
    for entry in groups_part.split(',') {
        assert!(
            is_rid_with_optional_name(entry),
            "groups entries must be numeric RIDs with optional (name): {entry:?}"
        );
    }
}

#[test]
fn groups_field_rids_equal_capital_g_output() {
    let full = run_id(&[]);
    let cap_g = run_id(&["-G"]);
    assert!(full.status.success());
    assert!(cap_g.status.success());

    let stdout = stdout_str(&full);
    let groups_field = stdout
        .rsplit_once(" groups=")
        .map(|(_, tail)| tail.trim_end())
        .unwrap_or_else(|| panic!("missing ' groups=' field in {stdout:?}"));

    let field_rids: Vec<String> = groups_field
        .split(',')
        .map(|e| e.split('(').next().unwrap().trim().to_string())
        .collect();

    let cap_g_line = stdout_str(&cap_g);
    let cap_g_rids: Vec<String> = cap_g_line.trim_end().split(' ').map(String::from).collect();
    assert!(
        cap_g_rids.iter().all(|r| !r.is_empty() && r.chars().all(|c| c.is_ascii_digit())),
        "id -G must print space separated numeric RIDs, got {cap_g_line:?}"
    );

    let mut a = field_rids;
    a.sort();
    a.dedup();
    let mut b = cap_g_rids;
    b.sort();
    b.dedup();
    assert_eq!(
        a, b,
        "the groups field of `id` must list the same RIDs as `id -G`"
    );
}

#[test]
fn u_and_g_flags_print_numeric_rids() {
    for flag in ["-u", "-g"] {
        let out = run_id(&[flag]);
        assert!(out.status.success(), "id {flag} must exit 0");
        let s = stdout_str(&out).trim_end().to_string();
        assert!(
            !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()),
            "id {flag} must print a bare numeric RID, got {s:?}"
        );
    }
}

#[test]
fn n_flag_prints_names_for_u_g_and_capital_g() {
    for flag in ["-u", "-g", "-G"] {
        let out = run_id(&[flag, "-n"]);
        assert!(out.status.success(), "id {flag} -n must exit 0");
        let s = stdout_str(&out).trim_end().to_string();
        assert!(
            !s.is_empty(),
            "id {flag} -n must print at least one name"
        );
        assert!(
            s.chars().any(|c| !c.is_ascii_digit() && c != ' '),
            "id {flag} -n must print names, not bare numbers, got {s:?}"
        );
    }
}
