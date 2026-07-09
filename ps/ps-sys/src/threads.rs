#![cfg(windows)]

use std::collections::HashMap;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, THREADENTRY32, TH32CS_SNAPTHREAD,
};

use crate::error::{Error, Result};

pub fn thread_counts() -> Result<HashMap<u32, u32>> {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0).map_err(Error::from)?;
        if snap.is_invalid() {
            return Err(Error::Internal(
                "CreateToolhelp32Snapshot THREAD failed".into(),
            ));
        }
        let mut map: HashMap<u32, u32> = HashMap::new();
        let mut entry = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };
        if Thread32First(snap, &mut entry).is_ok() {
            loop {
                *map.entry(entry.th32OwnerProcessID).or_insert(0) += 1;
                if Thread32Next(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn thread_counts_includes_current() {
        let m = thread_counts().unwrap();
        let me = unsafe { GetCurrentProcessId() };
        assert!(m.contains_key(&me));
    }
}
