use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::{Process, ProcessFlags};
use crate::state::ProcessState;

#[derive(Default)]
pub struct Stat;
impl Column for Stat {
    fn kind(&self) -> ColumnKind {
        ColumnKind::Stat
    }
    fn header(&self) -> &'static str {
        "STAT"
    }
    fn width(&self) -> usize {
        4
    }
    fn align_right(&self) -> bool {
        false
    }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String {
        let mut s = String::new();
        s.push(match p.state {
            ProcessState::Running => 'R',
            ProcessState::Sleeping => 'S',
            ProcessState::DiskSleep => 'D',
            ProcessState::Zombie => 'Z',
            ProcessState::Stopped => 'T',
            ProcessState::Tracing => 't',
            ProcessState::Dead => 'X',
            _ => '?',
        });
        if p.flags.contains(ProcessFlags::ELEVATED) {
            s.push('*');
        }
        s
    }
}
