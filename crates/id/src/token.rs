//! Process token identity: uid/gid/groups derived from the real access token.
//!
//! `OpenProcessToken` + `GetTokenInformation(TokenUser/TokenPrimaryGroup/
//! TokenGroups)` + `LookupAccountSidW`; numeric IDs are the last SID
//! sub-authority (RID).

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{
    GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation, LookupAccountSidW,
    TokenGroups, TokenPrimaryGroup, TokenUser, PSID, SID_NAME_USE, TOKEN_GROUPS,
    TOKEN_INFORMATION_CLASS, TOKEN_PRIMARY_GROUP, TOKEN_QUERY, TOKEN_USER,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

#[derive(Clone)]
pub struct Entry {
    pub rid: u32,
    pub name: Option<String>,
}

impl Entry {
    /// GNU style: `1234` or `1234(Name)`.
    pub fn display(&self) -> String {
        match &self.name {
            Some(n) => format!("{}({})", self.rid, n),
            None => self.rid.to_string(),
        }
    }
}

pub struct Identity {
    pub user: Entry,
    pub primary_group: Entry,
    pub groups: Vec<Entry>,
}

pub fn current_identity() -> Result<Identity, String> {
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)
            .map_err(|e| format!("cannot open process token: {e}"))?;

        let result = collect_identity(token);
        let _ = CloseHandle(token);
        result
    }
}

unsafe fn collect_identity(token: HANDLE) -> Result<Identity, String> {
    unsafe {
        let user_buf = token_info(token, TokenUser)?;
        let primary_buf = token_info(token, TokenPrimaryGroup)?;
        let groups_buf = token_info(token, TokenGroups)?;

        let user_sid = (*(user_buf.as_ptr() as *const TOKEN_USER)).User.Sid;
        let primary_sid = (*(primary_buf.as_ptr() as *const TOKEN_PRIMARY_GROUP)).PrimaryGroup;

        let user = sid_to_entry(user_sid);
        let primary_group = sid_to_entry(primary_sid);

        let tg = groups_buf.as_ptr() as *const TOKEN_GROUPS;
        let count = (*tg).GroupCount as usize;
        // `Groups` is declared as [SID_AND_ATTRIBUTES; 1]; the real length is
        // GroupCount, so read it as a raw slice.
        let group_sids = std::slice::from_raw_parts((*tg).Groups.as_ptr(), count);
        let mut groups: Vec<Entry> = Vec::with_capacity(count);
        for attrs in group_sids {
            let sid = attrs.Sid;
            if sid.0.is_null() {
                continue;
            }
            let entry = sid_to_entry(sid);
            if !groups.iter().any(|e| e.rid == entry.rid) {
                groups.push(entry);
            }
        }
        if !groups.iter().any(|e| e.rid == primary_group.rid) {
            groups.insert(0, primary_group.clone());
        }

        Ok(Identity {
            user,
            primary_group,
            groups,
        })
    }
}

unsafe fn token_info(token: HANDLE, class: TOKEN_INFORMATION_CLASS) -> Result<Vec<u8>, String> {
    unsafe {
        let mut needed = 0u32;
        let _ = GetTokenInformation(token, class, None, 0, &mut needed);
        if needed == 0 {
            return Err(format!("cannot query token information (class {class:?})"));
        }
        let mut buf = vec![0u8; needed as usize];
        GetTokenInformation(
            token,
            class,
            Some(buf.as_mut_ptr() as *mut _),
            needed,
            &mut needed,
        )
        .map_err(|e| format!("cannot query token information: {e}"))?;
        Ok(buf)
    }
}

unsafe fn sid_to_entry(sid: PSID) -> Entry {
    unsafe {
        Entry {
            rid: sid_rid(sid),
            name: lookup_account(sid).map(|(name, _)| name),
        }
    }
}

/// Last SID sub-authority (RID) as the numeric ID.
unsafe fn sid_rid(sid: PSID) -> u32 {
    unsafe {
        let count = GetSidSubAuthorityCount(sid);
        if count.is_null() || *count == 0 {
            return 0;
        }
        *GetSidSubAuthority(sid, (*count as u32) - 1)
    }
}

unsafe fn lookup_account(sid: PSID) -> Option<(String, String)> {
    unsafe {
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
        if name_len == 0 {
            return None;
        }
        let mut name_buf = vec![0u16; name_len as usize];
        let mut domain_buf: Vec<u16> = vec![0u16; domain_len as usize];
        LookupAccountSidW(
            PCWSTR::null(),
            sid,
            PWSTR(name_buf.as_mut_ptr()),
            &mut name_len,
            if domain_len > 0 {
                PWSTR(domain_buf.as_mut_ptr())
            } else {
                PWSTR::null()
            },
            &mut domain_len,
            &mut name_use,
        )
        .ok()?;

        let name = wide_to_string(&name_buf);
        let domain = wide_to_string(&domain_buf);
        Some((name, domain))
    }
}

fn wide_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    OsString::from_wide(&buf[..len]).to_string_lossy().into_owned()
}
