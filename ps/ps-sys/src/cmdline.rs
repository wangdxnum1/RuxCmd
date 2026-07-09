#![cfg(windows)]

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::Win32::Foundation::{CloseHandle, HANDLE, NTSTATUS};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ};

use crate::error::{Error, Result};

#[repr(C)]
struct PbiLocal {
    _reserved1: usize,
    peb_base: usize,
    _reserved2: [usize; 2],
    _pid: usize,
    _reserved3: usize,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct CmdUnicode {
    length: u16,
    _max_length: u16,
    _pad: u32,
    buffer: usize,
}

#[link(name = "ntdll")]
extern "system" {
    fn NtQueryInformationProcess(
        process_handle: HANDLE,
        process_information_class: u32,
        process_information: *mut core::ffi::c_void,
        process_information_length: u32,
        return_length: *mut u32,
    ) -> NTSTATUS;
}

const PROCESS_BASIC_INFORMATION_CLASS: u32 = 0;

pub fn cmdline(pid: u32) -> Result<Option<String>> {
    unsafe {
        let access = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ;
        let h = OpenProcess(access, false, pid).map_err(Error::from)?;
        if h.is_invalid() {
            return Err(Error::PermissionDenied(pid));
        }
        let mut pbi = std::mem::zeroed::<PbiLocal>();
        let mut ret_len = 0u32;
        let status = NtQueryInformationProcess(
            h,
            PROCESS_BASIC_INFORMATION_CLASS,
            &mut pbi as *mut _ as *mut _,
            std::mem::size_of::<PbiLocal>() as u32,
            &mut ret_len,
        );
        if status.0 != 0 {
            let _ = CloseHandle(h);
            return Err(Error::PebRead(pid));
        }
        let peb_base = pbi.peb_base;
        if peb_base == 0 {
            let _ = CloseHandle(h);
            return Ok(None);
        }

        let mut params_ptr: usize = 0;
        if !read_usize(h, peb_base + 0x20, &mut params_ptr) {
            let _ = CloseHandle(h);
            return Err(Error::PebRead(pid));
        }
        if params_ptr == 0 {
            let _ = CloseHandle(h);
            return Ok(None);
        }

        let mut cmd = CmdUnicode::default();
        if !read_struct::<CmdUnicode>(h, params_ptr + 0x70, &mut cmd) {
            let _ = CloseHandle(h);
            return Err(Error::PebRead(pid));
        }
        if cmd.length == 0 || cmd.buffer == 0 {
            let _ = CloseHandle(h);
            return Ok(None);
        }
        let mut buf = vec![0u16; (cmd.length / 2) as usize];
        if !read_bytes(h, cmd.buffer, buf.as_mut_ptr() as *mut u8, buf.len() * 2) {
            let _ = CloseHandle(h);
            return Err(Error::PebRead(pid));
        }
        let _ = CloseHandle(h);
        let s = OsString::from_wide(&buf).to_string_lossy().into_owned();
        Ok(Some(s))
    }
}

unsafe fn read_usize(h: HANDLE, addr: usize, dst: &mut usize) -> bool {
    let mut read = 0usize;
    ReadProcessMemory(
        h,
        addr as *const _,
        dst as *mut _ as *mut _,
        std::mem::size_of::<usize>(),
        Some(&mut read),
    )
    .is_ok()
        && read == std::mem::size_of::<usize>()
}

unsafe fn read_struct<T>(h: HANDLE, addr: usize, dst: *mut T) -> bool {
    let mut read = 0usize;
    ReadProcessMemory(
        h,
        addr as *const _,
        dst as *mut _ as *mut _,
        std::mem::size_of::<T>(),
        Some(&mut read),
    )
    .is_ok()
        && read == std::mem::size_of::<T>()
}

unsafe fn read_bytes(h: HANDLE, addr: usize, dst: *mut u8, len: usize) -> bool {
    let mut read = 0usize;
    ReadProcessMemory(
        h,
        addr as *const _,
        dst as *mut _ as *mut _,
        len,
        Some(&mut read),
    )
    .is_ok()
        && read == len
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn cmdline_of_current_present() {
        let pid = unsafe { GetCurrentProcessId() };
        let s = cmdline(pid).unwrap();
        assert!(s.is_some(), "expected cmdline for self, got None");
    }
}
