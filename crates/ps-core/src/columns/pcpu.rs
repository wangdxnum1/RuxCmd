use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;
use std::time::SystemTime;

#[derive(Default)]
pub struct Pcpu;
impl Column for Pcpu {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Pcpu
    }
    fn header(&self) -> &'static str {
        "%CPU"
    }
    fn width(&self) -> usize {
        5
    }
    fn align_right(&self) -> bool {
        true
    }
    fn format(&self, p: &Process, ctx: &FormatCtx) -> String {
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
}
