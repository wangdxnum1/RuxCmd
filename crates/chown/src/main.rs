mod cli;

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use cli::Cli;
use windows_sys::Win32::Foundation::{GetLastError, PSID};
use windows_sys::Win32::Security::Authorization::{SE_FILE_OBJECT, SetNamedSecurityInfoW};
use windows_sys::Win32::Security::{
    GROUP_SECURITY_INFORMATION, LookupAccountNameW, OWNER_SECURITY_INFORMATION, SID_NAME_USE,
};

fn main() {
    use clap::Parser;
    let cli = Cli::parse();

    if cli.user.is_none() && cli.group.is_none() {
        eprintln!("chown: either --user or --group must be specified");
        std::process::exit(1);
    }

    let user_sid = match cli.user.as_deref().map(lookup_account) {
        Some(Ok(sid)) => Some(sid),
        Some(Err(e)) => {
            eprintln!("chown: {}", e);
            std::process::exit(1);
        }
        None => None,
    };

    let group_sid = match cli.group.as_deref().map(lookup_account) {
        Some(Ok(sid)) => Some(sid),
        Some(Err(e)) => {
            eprintln!("chown: {}", e);
            std::process::exit(1);
        }
        None => None,
    };

    let mut had_error = false;
    for p in &cli.path {
        let path = Path::new(p);
        let failed = if cli.recursive && path.is_dir() {
            chown_recursive(path, user_sid, group_sid, cli.verbose)
        } else {
            match chown_file(path, user_sid, group_sid, cli.verbose) {
                Ok(()) => false,
                Err(e) => {
                    eprintln!("chown: {}", e);
                    true
                }
            }
        };
        had_error |= failed;
    }

    unsafe {
        if let Some(sid) = user_sid {
            kernel32::LocalFree(sid);
        }
        if let Some(sid) = group_sid {
            kernel32::LocalFree(sid);
        }
    }

    if had_error {
        std::process::exit(1);
    }
}

mod kernel32 {
    use windows_sys::Win32::Foundation::HLOCAL;
    extern "system" {
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
            "cannot find account '{}': LookupAccountNameW failed with error {}",
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
            "cannot find account '{}': LookupAccountNameW failed with error {}",
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
        return Err(format!("cannot find account '{}': LocalAlloc failed", name));
    }

    unsafe {
        std::ptr::copy_nonoverlapping(sid_buffer.as_ptr(), sid_ptr as *mut u8, sid_size as usize);
    }

    Ok(sid_ptr)
}

fn chown_file(
    path: &Path,
    user_sid: Option<PSID>,
    group_sid: Option<PSID>,
    verbose: bool,
) -> Result<(), String> {
    let path_w: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut info = 0u32;
    if user_sid.is_some() {
        info |= OWNER_SECURITY_INFORMATION;
    }
    if group_sid.is_some() {
        info |= GROUP_SECURITY_INFORMATION;
    }

    let result = unsafe {
        SetNamedSecurityInfoW(
            path_w.as_ptr(),
            SE_FILE_OBJECT,
            info,
            user_sid.unwrap_or(std::ptr::null_mut()),
            group_sid.unwrap_or(std::ptr::null_mut()),
            std::ptr::null(),
            std::ptr::null(),
        )
    };

    if result != 0 {
        return Err(format!(
            "cannot change owner/group of '{}': SetNamedSecurityInfoW failed with error {}",
            path.display(),
            result
        ));
    }

    if verbose {
        let what = match (user_sid.is_some(), group_sid.is_some()) {
            (true, true) => "changed owner and group",
            (true, false) => "changed owner",
            (false, true) => "changed group",
            (false, false) => "changed nothing",
        };
        println!("{}: {}", what, path.display());
    }
    Ok(())
}

/// Recursively chown everything under `path`; returns true if any entry failed.
/// Note: descent follows `is_dir()`, so junction/symlink cycles are not
/// guarded against (known P2 limitation).
fn chown_recursive(
    path: &Path,
    user_sid: Option<PSID>,
    group_sid: Option<PSID>,
    verbose: bool,
) -> bool {
    let mut had_error = false;

    let entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("chown: cannot read directory '{}': {}", path.display(), e);
            return true;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!(
                    "chown: cannot read directory entry under '{}': {}",
                    path.display(),
                    e
                );
                had_error = true;
                continue;
            }
        };

        let entry_path = entry.path();

        if let Err(e) = chown_file(&entry_path, user_sid, group_sid, verbose) {
            eprintln!("chown: {}", e);
            had_error = true;
        }

        if entry_path.is_dir() && chown_recursive(&entry_path, user_sid, group_sid, verbose) {
            had_error = true;
        }
    }

    if let Err(e) = chown_file(path, user_sid, group_sid, verbose) {
        eprintln!("chown: {}", e);
        had_error = true;
    }

    had_error
}
