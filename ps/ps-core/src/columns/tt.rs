use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Tt;
impl Column for Tt {
    fn kind(&self) -> ColumnKind { ColumnKind::Tt }
    fn header(&self) -> &'static str { "TT" }
    fn width(&self) -> usize { 3 }
    fn align_right(&self) -> bool { false }
    fn format(&self, _p: &Process, _ctx: &FormatCtx) -> String { "?".into() }
    fn supported(&self) -> bool { false }
}
