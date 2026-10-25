use crate::{
    meminfo::total_physical_memory,
    process::{Process, ProcessFlags},
};
use std::time::SystemTime;

pub(super) fn vsz(p: &Process) -> String {
    (p.virtual_size / 1024).to_string()
}

pub(super) fn rss(p: &Process) -> String {
    (p.working_set / 1024).to_string()
}

pub(super) fn pmem(p: &Process) -> String {
    match total_physical_memory() {
        Some(total) if total > 0 => {
            format!("{:.1}", (p.working_set as f64 / total as f64) * 100.0)
        }
        _ => "?".into(),
    }
}

pub(super) fn pcpu(p: &Process) -> String {
    // Lifetime average (procps %CPU semantics): CPU time over elapsed
    // time since process start. No second sample is needed.
    let Some(start) = p.start_time else {
        return "?".into();
    };
    let alive = SystemTime::now()
        .duration_since(start)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    if alive < 0.001 {
        return "0.0".into();
    }
    let cpu = (p.kernel_time + p.user_time).as_secs_f64();
    format!("{:.1}", 100.0 * cpu / alive)
}

pub(super) fn pri(p: &Process) -> String {
    (20 - p.nice).to_string()
}

pub(super) fn nice(p: &Process) -> String {
    p.nice.to_string()
}

pub(super) fn class(p: &Process) -> String {
    match p.priority_class {
        0x00000040 => "Idle".into(),
        0x00004000 => "Below".into(),
        0x00000020 => "Normal".into(),
        0x00008000 => "Above".into(),
        0x00000080 => "High".into(),
        0x00000100 => "Realtime".into(),
        _ => "?".into(),
    }
}

pub(super) fn thcount(p: &Process) -> String {
    p.thread_count.to_string()
}

pub(super) fn flags(p: &Process) -> String {
    format!("{:08x}", p.flags.bits())
}

pub(super) fn stat(p: &Process) -> String {
    let mut s = String::new();
    s.push(p.state);
    if p.flags.contains(ProcessFlags::ELEVATED) {
        s.push('*');
    }
    s
}
