use crate::column::{Column, ColumnKind, FormatCtx};
use crate::meminfo::total_physical_memory;
use crate::process::Process;

#[derive(Default)]
pub struct Pmem;
impl Column for Pmem {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Pmem
    }
    fn header(&self) -> &'static str {
        "%MEM"
    }
    fn width(&self) -> usize {
        5
    }
    fn align_right(&self) -> bool {
        true
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        match total_physical_memory() {
            Some(total) if total > 0 => {
                format!("{:.1}", (p.working_set as f64 / total as f64) * 100.0)
            }
            _ => "?".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn proc_with_working_set(working_set: u64) -> Process {
        Process {
            pid: 1,
            ppid: 0,
            tgid: 1,
            session_id: 0,
            name: "test.exe".into(),
            image_path: String::new(),
            cmdline: None,
            user: crate::user::User::default(),
            state: crate::state::ProcessState::Running,
            priority_class: 0,
            nice: 0,
            start_time: None,
            kernel_time: Default::default(),
            user_time: Default::default(),
            thread_count: 0,
            handles: None,
            working_set,
            peak_working_set: working_set,
            virtual_size: working_set,
            exit_status: None,
            flags: Default::default(),
        }
    }

    #[test]
    fn pmem_uses_real_total_physical_memory() {
        // Regression for the hardcoded 8.0e9 denominator: %MEM must equal
        // working_set / real total physical memory * 100. Real machines never
        // report ullTotalPhys == 8_000_000_000 (installed RAM is a power of
        // two in bytes), so the old hardcode failed this exact comparison.
        let total = total_physical_memory().expect("total physical memory must be available");
        assert!(total > 0);

        let working_set: u64 = 1_000_000_000;
        let p = proc_with_working_set(working_set);
        let ctx = FormatCtx {
            now: Instant::now(),
            prev: None,
        };
        let out = Pmem.format(&p, &ctx);
        let expected = format!("{:.1}", (working_set as f64 / total as f64) * 100.0);
        assert_eq!(out, expected, "%MEM must use real total physical memory");

        let value: f64 = out.parse().expect("%MEM must parse as a number");
        assert!(value > 0.0 && value <= 100.0);
    }
}
