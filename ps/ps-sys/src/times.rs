#![cfg(windows)]

use std::time::{Duration, SystemTime};
use windows::Win32::Foundation::{CloseHandle, FILETIME};
use windows::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

use crate::error::{Error, Result};
use crate::util::{duration_from_filetimes, filetime_to_systemtime};

pub struct Times {
    pub start: Option<SystemTime>,
    pub cpu: Duration,
    pub kernel: Duration,
    pub user: Duration,
}

pub fn times(pid: u32) -> Result<Times> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).map_err(Error::from)?;
        if h.is_invalid() {
            return Err(Error::PermissionDenied(pid));
        }
        let (mut ct, mut et, mut kt, mut ut) = (
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
            FILETIME::default(),
        );
        GetProcessTimes(h, &mut ct, &mut et, &mut kt, &mut ut).map_err(Error::from)?;
        let _ = CloseHandle(h);
        Ok(Times {
            start: filetime_to_systemtime(ct),
            cpu: duration_from_filetimes(kt, ut),
            kernel: duration_from_filetimes(kt, FILETIME::default()),
            user: duration_from_filetimes(FILETIME::default(), ut),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn times_of_current_process_has_start() {
        let pid = unsafe { GetCurrentProcessId() };
        let t = times(pid).unwrap();
        assert!(t.start.is_some());
    }
}
