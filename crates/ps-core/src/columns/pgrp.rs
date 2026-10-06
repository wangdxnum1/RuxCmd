use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Pgrp;
impl Column for Pgrp {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Pgrp
    }
    fn header(&self) -> &'static str {
        "PGRP"
    }
    fn width(&self) -> usize {
        5
    }
    fn align_right(&self) -> bool {
        true
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        p.tgid.to_string()
    }
}
