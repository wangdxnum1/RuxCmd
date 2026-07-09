use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Rgroup;
impl Column for Rgroup {
    fn kind(&self) -> ColumnKind { ColumnKind::Rgroup }
    fn header(&self) -> &'static str { "RGROUP" }
    fn width(&self) -> usize { 8 }
    fn align_right(&self) -> bool { false }
    fn format(&self, _p: &Process, _ctx: &FormatCtx) -> String { "?".into() }
    fn supported(&self) -> bool { false }
}
