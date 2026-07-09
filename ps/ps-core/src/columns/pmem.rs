use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Pmem;
impl Column for Pmem {
    fn kind(&self) -> ColumnKind { ColumnKind::Pmem }
    fn header(&self) -> &'static str { "%MEM" }
    fn width(&self) -> usize { 5 }
    fn align_right(&self) -> bool { true }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        format!("{:.1}", (p.working_set as f64 / 8.0e9) * 100.0)
    }
}
