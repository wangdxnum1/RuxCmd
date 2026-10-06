use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Tty;
impl Column for Tty {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Tty
    }
    fn header(&self) -> &'static str {
        "TTY"
    }
    fn width(&self) -> usize {
        3
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
