use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Pgid;
impl Column for Pgid {
    fn kind(&self) -> ColumnKind { ColumnKind::Pgid }
    fn header(&self) -> &'static str { "PGID" }
    fn width(&self) -> usize { 5 }
    fn align_right(&self) -> bool { true }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String { p.tgid.to_string() }
}
