use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Flags;
impl Column for Flags {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Flags
    }
    fn header(&self) -> &'static str {
        "F"
    }
    fn width(&self) -> usize {
        4
    }
    fn align_right(&self) -> bool {
        true
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        format!("{:08x}", p.flags.bits())
    }
}
