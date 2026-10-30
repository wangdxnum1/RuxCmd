//! Change group ownership - a Windows port of Linux chgrp. The GROUP
//! operand is resolved with LookupAccountNameW and applied with
//! SetNamedSecurityInfoW, mirroring the `chown --group` path.

use clap::Parser;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use windows_sys::Win32::Foundation::{GetLastError, PSID};
use windows_sys::Win32::Security::Authorization::{SE_FILE_OBJECT, SetNamedSecurityInfoW};
use windows_sys::Win32::Security::{GROUP_SECURITY_INFORMATION, LookupAccountNameW, SID_NAME_USE};

/// Change the group of each FILE to GROUP.
#[derive(Parser, Debug)]
#[command(name = "chgrp", version, about)]
pub struct Args {
    /// Change files and directories recursively
    #[arg(short = 'R', long = "recursive")]
    pub recursive: bool,

    /// Explain what is being done
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Suppress most error messages
    #[arg(short = 'f', long = "silent")]
    pub silent: bool,

    /// New group (name resolvable on this machine)
    #[arg(value_name = "GROUP")]
    pub group: String,

    /// Files to change
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();

    if args.files.is_empty() {
        eprintln!("chgrp: missing operand after '{}'", args.group);
        eprintln!("Try 'chgrp --help' for more information.");
        std::process::exit(1);
    }

    let group_sid = match lookup_account(&args.group) {
        Ok(sid) => sid,
        Err(e) => {
            eprintln!("chgrp: {}", e);
            std::process::exit(1);
        }
    };

    let mut had_error = false;
    for path in &args.files {
        let failed = if args.recursive && path.is_dir() {
            chgrp_recursive(path, group_sid, args.verbose, args.silent)
        } else {
            match chgrp_file(path, group_sid, args.verbose) {
                Ok(()) => false,
                Err(e) => {
                    if !args.silent {
                        eprintln!("chgrp: {}", e);
                    }
                    true
                }
            }
        };
        had_error |= failed;
    }

    unsafe {
        kernel32::LocalFree(group_sid);
    }

    if had_error {
        std::process::exit(1);
    }
}

mod kernel32 {
    use windows_sys::Win32::Foundation::HLOCAL;
    unsafe extern "system" {
        pub fn LocalFree(hMem: *mut core::ffi::c_void) -> HLOCAL;
    }
}

fn lookup_account(name: &str) -> Result<PSID, String> {
    let name_w: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut sid_size = 0u32;
    let mut domain_size = 0u32;
    let mut sid_name_use: SID_NAME_USE = 0;

    unsafe {
        LookupAccountNameW(
            std::ptr::null(),
            name_w.as_ptr(),
            std::ptr::null_mut(),
            &mut sid_size,
            std::ptr::null_mut(),
            &mut domain_size,
            &mut sid_name_use,
        );
    }

    let err = unsafe { GetLastError() };
    if err != 122 {
        return Err(format!(
            "cannot find group '{}': LookupAccountNameW failed with error {}",
            name, err
        ));
    }

    let mut sid_buffer = vec![0u8; sid_size as usize];
    let mut domain_buffer = vec![0u16; domain_size as usize];

    let result = unsafe {
        LookupAccountNameW(
            std::ptr::null(),
            name_w.as_ptr(),
            sid_buffer.as_mut_ptr() as PSID,
            &mut sid_size,
            domain_buffer.as_mut_ptr(),
            &mut domain_size,
            &mut sid_name_use,
        )
    };

    if result == 0 {
        let err = unsafe { GetLastError() };
        return Err(format!(
            "cannot find group '{}': LookupAccountNameW failed with error {}",
            name, err
        ));
    }

    let sid_ptr = unsafe {
        windows_sys::Win32::System::Memory::LocalAlloc(
            windows_sys::Win32::System::Memory::LMEM_ZEROINIT,
            sid_size as usize,
        )
    };

    if sid_ptr.is_null() {
        return Err(format!("cannot find group '{}': LocalAlloc failed", name));
    }

    unsafe {
        std::ptr::copy_nonoverlapping(sid_buffer.as_ptr(), sid_ptr as *mut u8, sid_size as usize);
    }

    Ok(sid_ptr)
}

fn chgrp_file(path: &Path, group_sid: PSID, verbose: bool) -> Result<(), String> {
    let path_w: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let result = unsafe {
        SetNamedSecurityInfoW(
            path_w.as_ptr(),
            SE_FILE_OBJECT,
            GROUP_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            group_sid,
            std::ptr::null(),
            std::ptr::null(),
        )
    };

    if result != 0 {
        return Err(format!(
            "cannot change group of '{}': SetNamedSecurityInfoW failed with error {}",
            path.display(),
            result
        ));
    }

    if verbose {
        println!("changed group of {} to requested group", path.display());
    }
    Ok(())
}

/// Recursively chgrp everything under `path`; returns true if any entry
/// failed. Descent follows `is_dir()`, so junction/symlink cycles are not
/// guarded against (same limitation as chown).
fn chgrp_recursive(path: &Path, group_sid: PSID, verbose: bool, silent: bool) -> bool {
    let mut had_error = false;

    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => {
            if !silent {
                eprintln!("chgrp: cannot read directory '{}': {}", path.display(), e);
            }
            return true;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                if !silent {
                    eprintln!(
                        "chgrp: cannot read directory entry under '{}': {}",
                        path.display(),
                        e
                    );
                }
                had_error = true;
                continue;
            }
        };

        let entry_path = entry.path();

        if let Err(e) = chgrp_file(&entry_path, group_sid, verbose) {
            if !silent {
                eprintln!("chgrp: {}", e);
            }
            had_error = true;
        }

        if entry_path.is_dir() && chgrp_recursive(&entry_path, group_sid, verbose, silent) {
            had_error = true;
        }
    }

    if let Err(e) = chgrp_file(path, group_sid, verbose) {
        if !silent {
            eprintln!("chgrp: {}", e);
        }
        had_error = true;
    }

    had_error
}
