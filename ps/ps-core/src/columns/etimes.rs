use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Etimes;
impl Column for Etimes {
    fn kind(&self) -> ColumnKind { ColumnKind::Etimes }
    fn header(&self) -> &'static str { "ELAPSED" }
    fn width(&self) -> usize { 7 }
    fn align_right(&self) -> bool { true }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        p.start_time
            .map_or("?".into(), |s| {
                std::time::SystemTime::now()
                    .duration_since(s)
                    .map_or("?".into(), |d| d.as_secs().to_string())
            })
    }
}
