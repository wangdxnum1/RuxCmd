mod cli;

use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use cli::Cli;
use windows_sys::Win32::Foundation::{GetLastError, PSID};
use windows_sys::Win32::Security::{
    GetFileSecurityW, LookupAccountNameW, SetFileSecurityW, SetSecurityDescriptorOwner,
    OWNER_SECURITY_INFORMATION, SID_NAME_USE,
};

fn main() {
    use clap::Parser;
    let cli = Cli::parse();

    if cli.user.is_none() && cli.group.is_none() {
        eprintln!("错误: 必须指定 --user 或 --group");
        std::process::exit(1);
    }

    let name = cli.user.as_deref().or(cli.group.as_deref()).unwrap();
    let sid = match lookup_account(name) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("错误: 无法查找 '{}': {}", name, e);
            std::process::exit(1);
        }
    };

    for p in &cli.path {
        let path = Path::new(p);
        if cli.recursive && path.is_dir() {
            if let Err(e) = chown_recursive(path, sid, cli.verbose) {
                eprintln!("错误: {}", e);
            }
        } else {
            if let Err(e) = chown_file(path, sid, cli.verbose) {
                eprintln!("错误: {}", e);
            }
        }
    }

    unsafe {
        kernel32::LocalFree(sid);
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
        return Err(format!("LookupAccountNameW 失败: {}", err));
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
        return Err(format!("LookupAccountNameW 失败: {}", err));
    }

    let sid_ptr = unsafe {
        windows_sys::Win32::System::Memory::LocalAlloc(
            windows_sys::Win32::System::Memory::LMEM_ZEROINIT,
            sid_size as usize,
        )
    };

    if sid_ptr == std::ptr::null_mut() {
        return Err("LocalAlloc 失败".to_string());
    }

    unsafe {
        std::ptr::copy_nonoverlapping(sid_buffer.as_ptr(), sid_ptr as *mut u8, sid_size as usize);
    }

    Ok(sid_ptr)
}

fn chown_file(path: &Path, sid: PSID, verbose: bool) -> Result<(), String> {
    let path_w: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut security_desc_size = 0u32;

    unsafe {
        GetFileSecurityW(
            path_w.as_ptr(),
            OWNER_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            0,
            &mut security_desc_size,
        );
    }

    let err = unsafe { GetLastError() };
    if err != 122 {
        return Err(format!("GetFileSecurityW 失败: {}", err));
    }

    let mut security_desc = vec![0u8; security_desc_size as usize];

    let result = unsafe {
        GetFileSecurityW(
            path_w.as_ptr(),
            OWNER_SECURITY_INFORMATION,
            security_desc.as_mut_ptr() as *mut _,
            security_desc_size,
            &mut security_desc_size,
        )
    };

    if result == 0 {
        let err = unsafe { GetLastError() };
        return Err(format!("GetFileSecurityW 失败: {}", err));
    }

    let result =
        unsafe { SetSecurityDescriptorOwner(security_desc.as_mut_ptr() as *mut _, sid, 0) };

    if result == 0 {
        let err = unsafe { GetLastError() };
        return Err(format!("SetSecurityDescriptorOwner 失败: {}", err));
    }

    let result = unsafe {
        SetFileSecurityW(
            path_w.as_ptr(),
            OWNER_SECURITY_INFORMATION,
            security_desc.as_mut_ptr() as *mut _,
        )
    };

    if result == 0 {
        let err = unsafe { GetLastError() };
        Err(format!(
            "无法更改文件 '{}' 的所有权: {}",
            path.display(),
            err
        ))
    } else {
        if verbose {
            println!("已更改所有权: {}", path.display());
        }
        Ok(())
    }
}

fn chown_recursive(path: &Path, sid: PSID, verbose: bool) -> Result<(), String> {
    let mut entries = match std::fs::read_dir(path) {
        Ok(e) => e,
        Err(e) => return Err(format!("无法读取目录 '{}': {}", path.display(), e)),
    };

    while let Some(entry) = entries.next() {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("警告: 无法读取目录项: {}", e);
                continue;
            }
        };

        let entry_path = entry.path();

        if let Err(e) = chown_file(&entry_path, sid, verbose) {
            eprintln!("警告: {}", e);
        }

        if entry_path.is_dir() {
            if let Err(e) = chown_recursive(&entry_path, sid, verbose) {
                eprintln!("警告: {}", e);
            }
        }
    }

    if let Err(e) = chown_file(path, sid, verbose) {
        eprintln!("警告: {}", e);
    }

    Ok(())
}
