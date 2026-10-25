#![cfg(windows)]

use std::sync::OnceLock;

static TOTAL_PHYS: OnceLock<Option<u64>> = OnceLock::new();

/// Total physical memory in bytes, sampled once from `GlobalMemoryStatusEx`
/// (via `ps-sys`) and cached for the lifetime of the process.
///
/// Returns `None` when the underlying Win32 call failed.
pub fn total_physical_memory() -> Option<u64> {
    *TOTAL_PHYS.get_or_init(ps_sys::total_physical_memory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_physical_memory_is_sane() {
        let total = total_physical_memory().expect("total physical memory must be available");
        assert!(total > 0);
    }
}
