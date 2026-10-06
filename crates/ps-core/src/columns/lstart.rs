use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Lstart;
impl Column for Lstart {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Lstart
    }
    fn header(&self) -> &'static str {
        "STARTED"
    }
    fn width(&self) -> usize {
        24
    }
    fn align_right(&self) -> bool {
        false
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        p.start_time.map_or("?".into(), |s| {
            s.duration_since(std::time::UNIX_EPOCH)
                .map_or("?".into(), |d| format!("{}", d.as_secs()))
        })
    }
}
