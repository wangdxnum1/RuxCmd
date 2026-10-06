use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Start;
impl Column for Start {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Start
    }
    fn header(&self) -> &'static str {
        "STARTED"
    }
    fn width(&self) -> usize {
        8
    }
    fn align_right(&self) -> bool {
        false
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        use std::time::UNIX_EPOCH;
        match p.start_time {
            Some(s) => s.duration_since(UNIX_EPOCH).map_or("?".into(), |d| {
                let total_min = d.as_secs() / 60;
                let hh = (total_min / 60) % 24;
                let mm = total_min % 60;
                format!("{hh:02}:{mm:02}")
            }),
            None => "?".into(),
        }
    }
}
