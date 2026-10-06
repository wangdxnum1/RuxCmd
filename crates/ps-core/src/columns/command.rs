use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Command;
impl Column for Command {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Command
    }
    fn header(&self) -> &'static str {
        "COMMAND"
    }
    fn width(&self) -> usize {
        80
    }
    fn align_right(&self) -> bool {
        false
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        p.cmdline.clone().unwrap_or_else(|| p.image_path.clone())
    }
}
