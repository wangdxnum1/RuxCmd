#![cfg(windows)]

pub mod cmdline;
pub mod error;
pub mod procinfo;
pub mod snapshot;
pub mod threads;
pub mod times;
pub mod token;
pub mod util;

pub use cmdline::cmdline;
pub use error::{Error, Result};
pub use procinfo::{procinfo, ProcInfo};
pub use snapshot::{snapshot as raw_snapshot, Snapshot};
pub use threads::thread_counts;
pub use times::{times, Times};
pub use token::{user, UserInfo};

pub struct ProcessRaw {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub info: Result<ProcInfo>,
    pub times: Result<Times>,
    pub user: Result<UserInfo>,
    pub cmdline: Option<String>,
    pub thread_count: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CollectOptions {
    /// Skip the PEB-based command line lookup. Useful when the caller only needs
    /// PID / module name / owner and wants to avoid touching system processes
    /// that error out while reading PEB.
    pub skip_cmdline: bool,
}

impl CollectOptions {
    pub const FULL: Self = Self { skip_cmdline: false };
    pub const FAST: Self = Self { skip_cmdline: true };
}

pub fn collect() -> Result<Vec<ProcessRaw>> {
    collect_with(CollectOptions::FULL)
}

pub fn collect_with(opts: CollectOptions) -> Result<Vec<ProcessRaw>> {
    let snap = raw_snapshot()?;
    let thread_map = thread_counts().unwrap_or_default();
    let mut out = Vec::with_capacity(snap.pids.len());
    for &pid in &snap.pids {
        let ppid = *snap.ppid_map.get(&pid).unwrap_or(&0);
        let name = snap.names.get(&pid).cloned().unwrap_or_default();
        let info = procinfo(pid);
        let times = times(pid);
        let user = user(pid);
        let cmdline = if opts.skip_cmdline {
            None
        } else {
            cmdline(pid).ok().flatten()
        };
        let thread_count = *thread_map.get(&pid).unwrap_or(&0);
        out.push(ProcessRaw {
            pid,
            ppid,
            name,
            info,
            times,
            user,
            cmdline,
            thread_count,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_with_full_includes_cmdline() {
        let raws = collect_with(CollectOptions::FULL).unwrap();
        let me = raws.iter().find(|r| r.pid == std::process::id()).expect("self");
        assert!(me.cmdline.is_some(), "current process should have a cmdline");
    }

    #[test]
    fn collect_with_fast_skips_cmdline() {
        let raws = collect_with(CollectOptions::FAST).unwrap();
        let me = raws.iter().find(|r| r.pid == std::process::id()).expect("self");
        assert!(me.cmdline.is_none(), "FAST mode must skip cmdline lookup");
    }
}
