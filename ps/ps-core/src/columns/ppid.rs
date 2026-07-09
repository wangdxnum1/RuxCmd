use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Ppid;
impl Column for Ppid {
    fn kind(&self) -> ColumnKind { ColumnKind::Ppid }
    fn header(&self) -> &'static str { "PPID" }
    fn width(&self) -> usize { 5 }
    fn align_right(&self) -> bool { true }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String { p.ppid.to_string() }
}
