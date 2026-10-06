use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Etime;
impl Column for Etime {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Etime
    }
    fn header(&self) -> &'static str {
        "ELAPSED"
    }
    fn width(&self) -> usize {
        11
    }
    fn align_right(&self) -> bool {
        false
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        let Some(start) = p.start_time else {
            return "?".into();
        };
        let secs = std::time::SystemTime::now()
            .duration_since(start)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let dd = secs / 86400;
        let hh = (secs / 3600) % 24;
        let mm = (secs / 60) % 60;
        let ss = secs % 60;
        if dd > 0 {
            format!("{dd:02}-{hh:02}:{mm:02}:{ss:02}")
        } else {
            format!("{hh:02}:{mm:02}:{ss:02}")
        }
    }
}
