use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Gid;
impl Column for Gid {
    fn kind(&self) -> ColumnKind { ColumnKind::Gid }
    fn header(&self) -> &'static str { "GID" }
    fn width(&self) -> usize { 5 }
    fn align_right(&self) -> bool { true }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String { p.user.sid.clone() }
}
