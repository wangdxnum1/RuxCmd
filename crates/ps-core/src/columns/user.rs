use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct UserCol;
impl Column for UserCol {
    fn kind(&self) -> ColumnKind {
        ColumnKind::User
    }
    fn header(&self) -> &'static str {
        "USER"
    }
    fn width(&self) -> usize {
        12
    }
    fn align_right(&self) -> bool {
        false
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        if p.session_id == 0 {
            "SYSTEM".into()
        } else if p.user.name.is_empty() {
            "?".into()
        } else {
            p.user.name.clone()
        }
    }
}
