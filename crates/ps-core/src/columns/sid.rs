use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Sid;
impl Column for Sid {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Sid
    }
    fn header(&self) -> &'static str {
        "SID"
    }
    fn width(&self) -> usize {
        4
    }
    fn align_right(&self) -> bool {
        true
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        p.session_id.to_string()
    }
}
