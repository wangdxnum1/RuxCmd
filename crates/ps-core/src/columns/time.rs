use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Time;
impl Column for Time {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Time
    }
    fn header(&self) -> &'static str {
        "TIME"
    }
    fn width(&self) -> usize {
        8
    }
    fn align_right(&self) -> bool {
        false
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        let total = (p.kernel_time + p.user_time).as_secs();
        format!(
            "{:02}:{:02}:{:02}",
            total / 3600,
            (total / 60) % 60,
            total % 60
        )
    }
}
