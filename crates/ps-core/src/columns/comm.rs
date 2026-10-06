use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Comm;
impl Column for Comm {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Comm
    }
    fn header(&self) -> &'static str {
        "COMM"
    }
    fn width(&self) -> usize {
        16
    }
    fn align_right(&self) -> bool {
        false
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        p.name.clone()
    }
}
