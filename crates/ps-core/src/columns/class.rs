use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Class;
impl Column for Class {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Class
    }
    fn header(&self) -> &'static str {
        "CLASS"
    }
    fn width(&self) -> usize {
        6
    }
    fn align_right(&self) -> bool {
        false
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        match p.priority_class {
            0x00000040 => "Idle".into(),
            0x00004000 => "Below".into(),
            0x00000020 => "Normal".into(),
            0x00008000 => "Above".into(),
            0x00000080 => "High".into(),
            0x00000100 => "Realtime".into(),
            _ => "?".into(),
        }
    }
}
