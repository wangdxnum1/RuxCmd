use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct Cp;
impl Column for Cp {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Cp
    }
    fn header(&self) -> &'static str {
        "CP"
    }
    fn width(&self) -> usize {
        4
    }
    fn align_right(&self) -> bool {
        true
    }
    fn format(&self, p: &Process, ctx: &FormatCtx) -> String {
        let Some(prev) = ctx.prev else {
            return "?".into();
        };
        if prev.pid != p.pid {
            return "?".into();
        }
        let dt = ctx.now.duration_since(prev.at).as_secs_f64();
        if dt < 0.001 {
            return "0".into();
        }
        let dcpu = (p.kernel_time + p.user_time)
            .saturating_sub(prev.cpu_total)
            .as_secs_f64();
        format!("{}", (100.0 * dcpu / dt).round() as i32)
    }
}
