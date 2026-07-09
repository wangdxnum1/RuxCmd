use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Times;
impl Column for Times {
    fn kind(&self) -> ColumnKind { ColumnKind::Times }
    fn header(&self) -> &'static str { "TIMES" }
    fn width(&self) -> usize { 11 }
    fn align_right(&self) -> bool { false }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        format!("{}:{}", p.user_time.as_secs(), p.kernel_time.as_secs())
    }
}
