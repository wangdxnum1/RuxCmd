#![cfg(windows)]

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::core::PCWSTR;
use windows::Win32::Foundation::FILETIME;

pub fn wstr_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    OsString::from_wide(&buf[..len])
        .to_string_lossy()
        .into_owned()
}

pub fn pwstr_to_string(p: PCWSTR) -> String {
    unsafe {
        if p.0.is_null() {
            return String::new();
        }
        let mut len = 0;
        let mut q = p.0;
        while !q.is_null() && *q != 0 {
            len += 1;
            q = q.add(1);
        }
        let slice = std::slice::from_raw_parts(p.0, len);
        wstr_to_string(slice)
    }
}

pub fn filetime_to_systemtime(ft: FILETIME) -> Option<std::time::SystemTime> {
    let ticks = ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64);
    if ticks == 0 {
        return None;
    }
    const OFFSET_100NS: u64 = 116_444_736_000_000_000;
    let epoch_100ns = ticks.checked_sub(OFFSET_100NS)?;
    let nanos = (epoch_100ns % 10_000_000) * 100;
    let secs = epoch_100ns / 10_000_000;
    Some(std::time::UNIX_EPOCH + std::time::Duration::new(secs, nanos as u32))
}

pub fn duration_from_filetimes(kernel: FILETIME, user: FILETIME) -> std::time::Duration {
    fn to_100ns(ft: FILETIME) -> u64 {
        ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
    }
    std::time::Duration::from_nanos((to_100ns(kernel) + to_100ns(user)) * 100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn wstr_to_string_truncates_at_nul() {
        let mut buf = [0u16; 4];
        buf[0] = b'a' as u16;
        buf[1] = b'b' as u16;
        buf[2] = b'c' as u16;
        buf[3] = 0;
        assert_eq!(wstr_to_string(&buf), "abc");
    }

    #[test]
    fn filetime_zero_is_none() {
        let ft = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        assert!(filetime_to_systemtime(ft).is_none());
    }

    #[test]
    fn filetime_unix_epoch() {
        let ft = FILETIME {
            dwLowDateTime: 0xD53E8000,
            dwHighDateTime: 0x019DB1DE,
        };
        let t = filetime_to_systemtime(ft).unwrap();
        let diff = t.duration_since(UNIX_EPOCH).unwrap_or_default();
        assert!(diff < Duration::from_secs(2));
    }
}
