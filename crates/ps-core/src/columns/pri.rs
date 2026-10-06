use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Pri;
impl Column for Pri {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Pri
    }
    fn header(&self) -> &'static str {
        "PRI"
    }
    fn width(&self) -> usize {
        3
    }
    fn align_right(&self) -> bool {
        true
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        (20 - p.nice).to_string()
    }
}
