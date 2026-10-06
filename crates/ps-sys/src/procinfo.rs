#![cfg(windows)]

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::ProcessStatus::{
    K32GetModuleBaseNameW, K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
};
use windows::Win32::System::RemoteDesktop::ProcessIdToSessionId;
use windows::Win32::System::SystemInformation::IMAGE_FILE_MACHINE;
use windows::Win32::System::Threading::{
    GetPriorityClass, IsWow64Process2, OpenProcess, QueryFullProcessImageNameW,
    PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};

use crate::error::{Error, Result};
use crate::util::wstr_to_string;

pub struct ProcInfo {
    pub image_path: String,
    pub session_id: u32,
    pub working_set: u64,
    pub peak_working_set: u64,
    pub pagefile_usage: u64,
    pub priority_class: u32,
    pub wow64: bool,
}

pub fn procinfo(pid: u32) -> Result<ProcInfo> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).map_err(Error::from)?;
        if h.is_invalid() {
            return Err(Error::PermissionDenied(pid));
        }
        let image_path = image_path(h);
        let mut session: u32 = 0;
        let session_id = ProcessIdToSessionId(pid, &mut session)
            .map(|_| session)
            .unwrap_or(0);
        let mem = memory(h).unwrap_or((0, 0, 0));
        let priority_class = GetPriorityClass(h);
        let wow64 = wow64_status(h).unwrap_or(false);
        let _ = CloseHandle(h);
        Ok(ProcInfo {
            image_path: image_path.unwrap_or_default(),
            session_id,
            working_set: mem.0,
            peak_working_set: mem.1,
            pagefile_usage: mem.2,
            priority_class,
            wow64,
        })
    }
}

unsafe fn image_path(h: HANDLE) -> Result<String> {
    let mut buf = [0u16; 1024];
    let mut len = buf.len() as u32;
    QueryFullProcessImageNameW(h, PROCESS_NAME_FORMAT(0), PWSTR(buf.as_mut_ptr()), &mut len)
        .map_err(Error::from)?;
    Ok(wstr_to_string(&buf[..len as usize]))
}

unsafe fn memory(h: HANDLE) -> Result<(u64, u64, u64)> {
    let mut mc = PROCESS_MEMORY_COUNTERS::default();
    let cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    let ok = K32GetProcessMemoryInfo(h, &mut mc, cb);
    if !ok.as_bool() {
        return Ok((0, 0, 0));
    }
    Ok((
        mc.WorkingSetSize as u64,
        mc.PeakWorkingSetSize as u64,
        mc.PagefileUsage as u64,
    ))
}

unsafe fn wow64_status(h: HANDLE) -> Result<bool> {
    let mut process_machine = IMAGE_FILE_MACHINE(0);
    let mut native_machine = IMAGE_FILE_MACHINE(0);
    IsWow64Process2(
        h,
        &mut process_machine as *mut _,
        Some(&mut native_machine as *mut _),
    )
    .map_err(Error::from)?;
    Ok(process_machine.0 != 0)
}

#[allow(dead_code)]
pub fn module_base_name(h: HANDLE) -> Option<String> {
    let mut buf = [0u16; 1024];
    let n = unsafe { K32GetModuleBaseNameW(h, None, &mut buf) };
    if n == 0 {
        return None;
    }
    Some(wstr_to_string(&buf[..n as usize]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn procinfo_of_current_has_image() {
        let pid = unsafe { GetCurrentProcessId() };
        let p = procinfo(pid).unwrap();
        assert!(!p.image_path.is_empty());
    }
}
