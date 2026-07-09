use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Thcount;
impl Column for Thcount {
    fn kind(&self) -> ColumnKind { ColumnKind::Thcount }
    fn header(&self) -> &'static str { "THCNT" }
    fn width(&self) -> usize { 5 }
    fn align_right(&self) -> bool { true }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String { p.thread_count.to_string() }
}
