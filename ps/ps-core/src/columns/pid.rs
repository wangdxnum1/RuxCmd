use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Pid;
impl Column for Pid {
    fn kind(&self) -> ColumnKind { ColumnKind::Pid }
    fn header(&self) -> &'static str { "PID" }
    fn width(&self) -> usize { 5 }
    fn align_right(&self) -> bool { true }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String { p.pid.to_string() }
}
