use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Vsz;
impl Column for Vsz {
    fn kind(&self) -> ColumnKind { ColumnKind::Vsz }
    fn header(&self) -> &'static str { "VSZ" }
    fn width(&self) -> usize { 7 }
    fn align_right(&self) -> bool { true }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        (p.virtual_size / 1024).to_string()
    }
}
