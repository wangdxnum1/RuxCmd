#![cfg(windows)]

use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

/// Total physical memory in bytes, as reported by `GlobalMemoryStatusEx`.
///
/// Returns `None` when the Win32 call fails.
pub fn total_physical_memory() -> Option<u64> {
    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    unsafe {
        // Fails only on malformed input / exotic OOM conditions; treat any
        // failure as "total memory unknown".
        GlobalMemoryStatusEx(&mut status).ok()?;
    }
    Some(status.ullTotalPhys)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_physical_memory_is_sane() {
        let total = total_physical_memory().expect("GlobalMemoryStatusEx should succeed");
        assert!(total > 0, "total physical memory must be positive");
    }
}
