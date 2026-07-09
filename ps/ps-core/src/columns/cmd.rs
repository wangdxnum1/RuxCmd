use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Cmd;
impl Column for Cmd {
    fn kind(&self) -> ColumnKind { ColumnKind::Cmd }
    fn header(&self) -> &'static str { "CMD" }
    fn width(&self) -> usize { 256 }
    fn align_right(&self) -> bool { false }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        p.cmdline.clone().unwrap_or_else(|| p.image_path.clone())
    }
}
