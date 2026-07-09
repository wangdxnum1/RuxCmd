use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Args;
impl Column for Args {
    fn kind(&self) -> ColumnKind { ColumnKind::Args }
    fn header(&self) -> &'static str { "ARGS" }
    fn width(&self) -> usize { 32 }
    fn align_right(&self) -> bool { false }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        p.cmdline.clone().unwrap_or_else(|| p.image_path.clone())
    }
}
