use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Suser;
impl Column for Suser {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Suser
    }
    fn header(&self) -> &'static str {
        "SUSER"
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
