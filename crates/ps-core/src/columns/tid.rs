use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Tid;
impl Column for Tid {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Tid
    }
    fn header(&self) -> &'static str {
        "TID"
    }
    fn width(&self) -> usize {
        5
    }
    fn align_right(&self) -> bool {
        true
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        p.tgid.to_string()
    }
}
