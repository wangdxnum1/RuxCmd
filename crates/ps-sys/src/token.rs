#![cfg(windows)]

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, LocalFree, HANDLE, HLOCAL};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::{
    GetTokenInformation, LookupAccountSidW, TokenUser, SID_NAME_USE, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{
    OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};

use crate::error::{Error, Result};

pub struct UserInfo {
    pub name: String,
    pub domain: String,
    pub sid: String,
}

pub fn user(pid: u32) -> Result<UserInfo> {
    unsafe {
        let ph = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).map_err(Error::from)?;
        if ph.is_invalid() {
            return Err(Error::PermissionDenied(pid));
        }
        let mut th = HANDLE::default();
        OpenProcessToken(ph, TOKEN_QUERY, &mut th).map_err(Error::from)?;
        let _ = CloseHandle(ph);

        let mut needed = 0u32;
        let _ = GetTokenInformation(th, TokenUser, None, 0, &mut needed);
        if needed == 0 {
            let _ = CloseHandle(th);
            return Err(Error::PermissionDenied(pid));
        }
        let mut buf = vec![0u8; needed as usize];
        GetTokenInformation(
            th,
            TokenUser,
            Some(buf.as_mut_ptr() as *mut _),
            needed,
            &mut needed,
        )
        .map_err(Error::from)?;
        let _ = CloseHandle(th);

        let token_user: *const TOKEN_USER = buf.as_ptr() as *const _;
        let sid = (*token_user).User.Sid;
        Ok(sid_to_info(sid))
    }
}

#[repr(C)]
#[allow(non_snake_case)]
struct TOKEN_USER {
    User: SID_AND_ATTRIBUTES,
}

#[repr(C)]
#[allow(non_snake_case)]
struct SID_AND_ATTRIBUTES {
    Sid: windows::Win32::Security::PSID,
    Attributes: u32,
}

unsafe fn sid_to_info(sid: windows::Win32::Security::PSID) -> UserInfo {
    let mut name_len = 0u32;
    let mut domain_len = 0u32;
    let mut name_use = SID_NAME_USE(0);
    let _ = LookupAccountSidW(
        PCWSTR::null(),
        sid,
        PWSTR(std::ptr::null_mut()),
        &mut name_len,
        PWSTR(std::ptr::null_mut()),
        &mut domain_len,
        &mut name_use,
    );
    if name_len == 0 || domain_len == 0 {
        return UserInfo {
            name: "?".into(),
            domain: "?".into(),
            sid: sid_to_string(sid),
        };
    }
    let mut name_buf = vec![0u16; name_len as usize];
    let mut domain_buf = vec![0u16; domain_len as usize];
    let ok = LookupAccountSidW(
        PCWSTR::null(),
        sid,
        PWSTR(name_buf.as_mut_ptr()),
        &mut name_len,
        PWSTR(domain_buf.as_mut_ptr()),
        &mut domain_len,
        &mut name_use,
    );
    if ok.is_err() {
        return UserInfo {
            name: "?".into(),
            domain: "?".into(),
            sid: sid_to_string(sid),
        };
    }
    let name = OsString::from_wide(&name_buf[..name_len as usize])
        .to_string_lossy()
        .into_owned();
    let domain = OsString::from_wide(&domain_buf[..domain_len as usize])
        .to_string_lossy()
        .into_owned();
    let sid_str = sid_to_string(sid);
    UserInfo {
        name,
        domain,
        sid: sid_str,
    }
}

unsafe fn sid_to_string(sid: windows::Win32::Security::PSID) -> String {
    let mut out = PWSTR::null();
    if ConvertSidToStringSidW(sid, &mut out).is_ok() && !out.0.is_null() {
        let mut len = 0;
        let mut q = out.0;
        while !q.is_null() && *q != 0 {
            len += 1;
            q = q.add(1);
        }
        let slice = std::slice::from_raw_parts(out.0, len);
        let s = OsString::from_wide(slice).to_string_lossy().into_owned();
        let _ = LocalFree(HLOCAL(out.0 as *mut _));
        return s;
    }
    "?".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn user_of_current_has_name() {
        let pid = unsafe { GetCurrentProcessId() };
        let u = user(pid).unwrap();
        assert!(!u.name.is_empty());
        assert!(u.sid.starts_with("S-1-"));
    }
}
