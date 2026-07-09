use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Sess;
impl Column for Sess {
    fn kind(&self) -> ColumnKind { ColumnKind::Sess }
    fn header(&self) -> &'static str { "SESS" }
    fn width(&self) -> usize { 4 }
    fn align_right(&self) -> bool { true }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String { p.session_id.to_string() }
}
