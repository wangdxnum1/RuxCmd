use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Rss;
impl Column for Rss {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Rss
    }
    fn header(&self) -> &'static str {
        "RSS"
    }
    fn width(&self) -> usize {
        7
    }
    fn align_right(&self) -> bool {
        true
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        (p.working_set / 1024).to_string()
    }
}
