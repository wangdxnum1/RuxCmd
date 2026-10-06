use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Group;
impl Column for Group {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Group
    }
    fn header(&self) -> &'static str {
        "GROUP"
    }
    fn width(&self) -> usize {
        8
    }
    fn align_right(&self) -> bool {
        false
    }
    fn format(&self, _p: &Process, _ctx: &FormatCtx) -> String {
        "?".into()
    }
    fn supported(&self) -> bool {
        false
    }
}
