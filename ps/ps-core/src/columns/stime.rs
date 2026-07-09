use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

fn format_time_short(t: Option<std::time::SystemTime>) -> String {
    use std::time::UNIX_EPOCH;
    match t {
        Some(s) => {
            let dur = s.duration_since(UNIX_EPOCH).unwrap_or_default();
            let total_min = dur.as_secs() / 60;
            let hh = (total_min / 60) % 24;
            let mm = total_min % 60;
            format!("{hh:02}:{mm:02}")
        }
        None => "?".into(),
    }
}

#[derive(Default)]
pub struct Stime;
impl Column for Stime {
    fn kind(&self) -> ColumnKind { ColumnKind::Stime }
    fn header(&self) -> &'static str { "STIME" }
    fn width(&self) -> usize { 5 }
    fn align_right(&self) -> bool { false }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String { format_time_short(p.start_time) }
}
