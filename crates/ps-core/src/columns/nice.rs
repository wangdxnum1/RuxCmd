use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Nice;
impl Column for Nice {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Nice
    }
    fn header(&self) -> &'static str {
        "NI"
    }
    fn width(&self) -> usize {
        3
    }
    fn align_right(&self) -> bool {
        true
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        p.nice.to_string()
    }
}
