#![cfg(windows)]

use std::collections::HashMap;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::ProcessStatus::{EnumProcesses, K32GetModuleBaseNameW};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

use crate::error::{Error, Result};
use crate::util::wstr_to_string;

const INITIAL_BUF: usize = 1024;
const MAX_BUF_BYTES: usize = 1 << 22;

pub struct Snapshot {
    pub pids: Vec<u32>,
    pub ppid_map: HashMap<u32, u32>,
    pub names: HashMap<u32, String>,
}

pub fn snapshot() -> Result<Snapshot> {
    let pids = enum_processes()?;
    let ppid_map = toolhelp32_ppids(&pids)?;
    let names = module_names(&pids)?;
    Ok(Snapshot {
        pids,
        ppid_map,
        names,
    })
}

fn enum_processes() -> Result<Vec<u32>> {
    let mut cap = INITIAL_BUF;
    loop {
        let mut buf = vec![0u32; cap];
        let mut needed = 0u32;
        unsafe {
            EnumProcesses(buf.as_mut_ptr(), (buf.len() * 4) as u32, &mut needed)
                .map_err(Error::from)?;
        }
        let count = (needed as usize) / std::mem::size_of::<u32>();
        if count < buf.len() {
            buf.truncate(count);
            buf.retain(|&p| p != 0);
            return Ok(buf);
        }
        if cap * 4 > MAX_BUF_BYTES {
            return Err(Error::Internal("EnumProcesses overflow".into()));
        }
        cap *= 2;
    }
}

fn toolhelp32_ppids(pids: &[u32]) -> Result<HashMap<u32, u32>> {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map_err(Error::from)?;
        if snap.is_invalid() {
            return Err(Error::Internal("CreateToolhelp32Snapshot failed".into()));
        }
        let mut map = HashMap::new();
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                map.insert(entry.th32ProcessID, entry.th32ParentProcessID);
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
        for &p in pids {
            map.entry(p).or_insert(0);
        }
        Ok(map)
    }
}

fn module_names(pids: &[u32]) -> Result<HashMap<u32, String>> {
    let mut map = HashMap::new();
    for &pid in pids {
        let name = module_name(pid).unwrap_or_default();
        map.insert(pid, name);
    }
    Ok(map)
}

fn module_name(pid: u32) -> Result<String> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).map_err(Error::from)?;
        if h.is_invalid() {
            return Err(Error::PermissionDenied(pid));
        }
        let mut buf = [0u16; 1024];
        let n = K32GetModuleBaseNameW(h, None, &mut buf);
        let _ = CloseHandle(h);
        if n == 0 {
            return Err(Error::PermissionDenied(pid));
        }
        Ok(wstr_to_string(&buf[..n as usize]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn snapshot_contains_current_pid() {
        let s = snapshot().unwrap();
        let me = unsafe { GetCurrentProcessId() };
        assert!(s.pids.contains(&me), "current pid {me} not in snapshot");
    }

    #[test]
    fn snapshot_ppid_map_has_self() {
        let s = snapshot().unwrap();
        let me = unsafe { GetCurrentProcessId() };
        assert!(s.ppid_map.contains_key(&me));
    }
}
