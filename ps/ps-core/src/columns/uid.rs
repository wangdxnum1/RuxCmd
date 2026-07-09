use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Uid;
impl Column for Uid {
    fn kind(&self) -> ColumnKind { ColumnKind::Uid }
    fn header(&self) -> &'static str { "UID" }
    fn width(&self) -> usize { 12 }
    fn align_right(&self) -> bool { false }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        if p.session_id == 0 {
            "SYSTEM".into()
        } else {
            p.user.sid.clone()
        }
    }
}
