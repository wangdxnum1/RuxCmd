use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Ruser;
impl Column for Ruser {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Ruser
    }
    fn header(&self) -> &'static str {
        "RUSER"
    }
    fn width(&self) -> usize {
        12
    }
    fn align_right(&self) -> bool {
        false
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        p.user.name.clone()
    }
}
