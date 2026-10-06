#![cfg(windows)]

use crate::process::{Process, ProcessFlags};
use crate::state::ProcessState;
use crate::user::User;
use ps_sys::ProcessRaw;

pub fn assemble_one(r: ProcessRaw) -> Process {
    let mut flags = ProcessFlags::empty();
    let (
        image_path,
        session_id,
        working_set,
        peak_working_set,
        pagefile_usage,
        priority_class,
        wow64,
    ) = match r.info {
        Ok(i) => (
            i.image_path,
            i.session_id,
            i.working_set,
            i.peak_working_set,
            i.pagefile_usage,
            i.priority_class,
            i.wow64,
        ),
        Err(_) => (String::new(), 0, 0, 0, 0, 0, false),
    };
    if wow64 {
        flags |= ProcessFlags::WOW64;
    }
    let (start, kernel, user) = match r.times {
        Ok(t) => (t.start, t.kernel, t.user),
        Err(_) => (None, Default::default(), Default::default()),
    };
    let user_info = match r.user {
        Ok(u) => User {
            name: u.name,
            domain: u.domain,
            sid: u.sid,
        },
        Err(_) => User::default(),
    };
    let name = if r.name.is_empty() {
        std::path::Path::new(&image_path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    } else {
        r.name
    };
    let nice = map_nice(priority_class);
    let virtual_size = working_set.saturating_add(pagefile_usage);
    Process {
        pid: r.pid,
        ppid: r.ppid,
        tgid: r.pid,
        session_id,
        name,
        image_path,
        cmdline: r.cmdline,
        user: user_info,
        state: ProcessState::Running,
        priority_class,
        nice,
        start_time: start,
        kernel_time: kernel,
        user_time: user,
        thread_count: r.thread_count,
        handles: None,
        working_set,
        peak_working_set,
        virtual_size,
        exit_status: None,
        flags,
    }
}

fn map_nice(prio: u32) -> i32 {
    match prio {
        0x00000040 => 19,
        0x00004000 => 15,
        0x00000020 => 10,
        0x00008000 => 5,
        0x00000080 => -5,
        0x00000100 => -20,
        _ => 10,
    }
}

pub fn snapshot() -> Vec<Process> {
    snapshot_with(ps_sys::CollectOptions::FULL)
}

pub fn snapshot_with(opts: ps_sys::CollectOptions) -> Vec<Process> {
    ps_sys::collect_with(opts)
        .unwrap_or_default()
        .into_iter()
        .map(assemble_one)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ps_sys::Error;

    #[test]
    fn assemble_handles_missing_info() {
        let r = ProcessRaw {
            pid: 7,
            ppid: 1,
            name: "x.exe".into(),
            info: Err(Error::PermissionDenied(7)),
            times: Err(Error::PermissionDenied(7)),
            user: Err(Error::PermissionDenied(7)),
            cmdline: None,
            thread_count: 0,
        };
        let p = assemble_one(r);
        assert_eq!(p.pid, 7);
        assert_eq!(p.name, "x.exe");
    }
}
