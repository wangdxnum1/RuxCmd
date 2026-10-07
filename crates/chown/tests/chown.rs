//! Regression tests for P0 #48: `chown --group` must change the file's group
//! (GROUP_SECURITY_INFORMATION) and never touch its owner, `--user` and
//! `--group` may be combined, and any per-file failure must exit 1.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::Path;

use windows_sys::Win32::Foundation::{GetLastError, BOOL, PSID};
use windows_sys::Win32::Security::{
    GetFileSecurityW, GetLengthSid, GetSecurityDescriptorGroup, GetSecurityDescriptorOwner,
    LookupAccountNameW, PSECURITY_DESCRIPTOR, SID_NAME_USE,
};

/// OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION
const OWNER_AND_GROUP: u32 = 1 | 2;
const ERROR_INSUFFICIENT_BUFFER: u32 = 122;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Resolve `name` to a SID and return its bytes.
fn lookup_sid(name: &str) -> Vec<u8> {
    let name_w = wide(name);
    let mut sid_size = 0u32;
    let mut domain_size = 0u32;
    let mut use_kind: SID_NAME_USE = 0;
    unsafe {
        LookupAccountNameW(
            std::ptr::null(),
            name_w.as_ptr(),
            std::ptr::null_mut(),
            &mut sid_size,
            std::ptr::null_mut(),
            &mut domain_size,
            &mut use_kind,
        );
    }
    assert_eq!(
        unsafe { GetLastError() },
        ERROR_INSUFFICIENT_BUFFER,
        "first LookupAccountNameW call must return the required buffer sizes"
    );

    let mut sid = vec![0u8; sid_size as usize];
    let mut domain = vec![0u16; domain_size as usize];
    let ok = unsafe {
        LookupAccountNameW(
            std::ptr::null(),
            name_w.as_ptr(),
            sid.as_mut_ptr() as PSID,
            &mut sid_size,
            domain.as_mut_ptr(),
            &mut domain_size,
            &mut use_kind,
        )
    };
    assert_ne!(ok, 0, "LookupAccountNameW must resolve {name:?}");
    sid
}

/// Read a file's owner and group SID bytes.
fn get_owner_and_group(path: &Path) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
    let path_w = wide(&path.as_os_str().to_string_lossy());
    let mut needed = 0u32;
    unsafe {
        GetFileSecurityW(
            path_w.as_ptr(),
            OWNER_AND_GROUP,
            std::ptr::null_mut(),
            0,
            &mut needed,
        );
    }
    assert_eq!(
        unsafe { GetLastError() },
        ERROR_INSUFFICIENT_BUFFER,
        "first GetFileSecurityW call must return the required buffer size"
    );

    let mut sd = vec![0u8; needed as usize];
    let ok = unsafe {
        GetFileSecurityW(
            path_w.as_ptr(),
            OWNER_AND_GROUP,
            sd.as_mut_ptr() as PSECURITY_DESCRIPTOR,
            needed,
            &mut needed,
        )
    };
    assert_ne!(ok, 0, "GetFileSecurityW must succeed");

    fn descriptor_sid(sd: PSECURITY_DESCRIPTOR, owner: bool) -> Option<Vec<u8>> {
        let mut sid: PSID = std::ptr::null_mut();
        let mut defaulted: BOOL = 0;
        let ok = unsafe {
            if owner {
                GetSecurityDescriptorOwner(sd, &mut sid, &mut defaulted)
            } else {
                GetSecurityDescriptorGroup(sd, &mut sid, &mut defaulted)
            }
        };
        if ok == 0 || sid.is_null() {
            return None;
        }
        let len = unsafe { GetLengthSid(sid) } as usize;
        Some(unsafe { std::slice::from_raw_parts(sid as *const u8, len) }.to_vec())
    }

    let sd = sd.as_mut_ptr() as PSECURITY_DESCRIPTOR;
    (descriptor_sid(sd, true), descriptor_sid(sd, false))
}

fn current_user_name() -> String {
    std::env::var("USERNAME").expect("USERNAME environment variable")
}

// --- P0 #48: --group must change the group, not the owner ---

#[test]
fn group_only_changes_group_and_leaves_owner_alone() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("group-only.txt");
    fs::write(&file, b"payload").unwrap();

    let (owner_before, _) = get_owner_and_group(&file);
    let admins = lookup_sid("Administrators");

    cargo_bin_cmd!("chown")
        .arg("--group")
        .arg("Administrators")
        .arg(&file)
        .assert()
        .success();

    let (owner_after, group_after) = get_owner_and_group(&file);
    assert_eq!(
        owner_after, owner_before,
        "--group must not change the file owner (P0 #48)"
    );
    assert_eq!(
        group_after.as_deref(),
        Some(admins.as_slice()),
        "--group must set the file group to Administrators"
    );
}

// --- P2 fix: --user and --group may be combined ---

#[test]
fn user_and_group_can_be_set_together() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("user-and-group.txt");
    fs::write(&file, b"payload").unwrap();

    let user = lookup_sid(&current_user_name());
    let admins = lookup_sid("Administrators");

    cargo_bin_cmd!("chown")
        .arg("--user")
        .arg(current_user_name())
        .arg("--group")
        .arg("Administrators")
        .arg(&file)
        .assert()
        .success();

    let (owner_after, group_after) = get_owner_and_group(&file);
    assert_eq!(
        owner_after.as_deref(),
        Some(user.as_slice()),
        "--user must set the file owner"
    );
    assert_eq!(
        group_after.as_deref(),
        Some(admins.as_slice()),
        "--group must not be silently ignored when combined with --user"
    );
}

// --- --user path keeps working after the SetNamedSecurityInfoW migration ---

#[test]
fn user_changes_owner() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("user-only.txt");
    fs::write(&file, b"payload").unwrap();

    let user = lookup_sid(&current_user_name());

    cargo_bin_cmd!("chown")
        .arg("--user")
        .arg(current_user_name())
        .arg(&file)
        .assert()
        .success();

    let (owner_after, _) = get_owner_and_group(&file);
    assert_eq!(
        owner_after.as_deref(),
        Some(user.as_slice()),
        "--user must set the file owner"
    );
}

// --- error paths must exit 1 ---

#[test]
fn unknown_account_fails_with_exit_1() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("target.txt");
    fs::write(&file, b"payload").unwrap();

    cargo_bin_cmd!("chown")
        .arg("--user")
        .arg("definitely-not-an-account-31337")
        .arg(&file)
        .assert()
        .code(1)
        .stderr(predicates::str::contains("definitely-not-an-account-31337"));
}

#[test]
fn failing_file_makes_the_process_exit_1() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.txt");

    cargo_bin_cmd!("chown")
        .arg("--user")
        .arg(current_user_name())
        .arg(&missing)
        .assert()
        .code(1)
        .stderr(predicates::str::contains("missing.txt"));
}
