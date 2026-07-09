#![cfg(windows)]

use crate::process::Process;

#[derive(Debug, Clone, Default)]
pub struct Selector {
    pub pids: Vec<u32>,
    pub users: Vec<String>,
    pub group: Vec<u32>,
    pub session: Vec<u32>,
    pub comm_patterns: Vec<String>,
    pub include_all: bool,
    pub include_no_ttys: bool,
    pub explicit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOp {
    Eq,
    Ne,
    Lt,
    Gt,
}

#[derive(Debug, Clone)]
pub struct FilterExpr {
    pub col: String,
    pub op: FilterOp,
    pub val: String,
}

pub fn passes(p: &Process, sel: &Selector) -> bool {
    if !sel.pids.is_empty() && !sel.pids.contains(&p.pid) {
        return false;
    }
    if !sel.users.is_empty()
        && !sel.users.iter().any(|u| u == &p.user.name || u == &p.user.sid)
    {
        return false;
    }
    if !sel.session.is_empty() && !sel.session.contains(&p.session_id) {
        return false;
    }
    if !sel.comm_patterns.is_empty() {
        let name = p.cmdline.as_deref().unwrap_or(&p.name);
        let first = name.split_whitespace().next().unwrap_or("");
        if !sel
            .comm_patterns
            .iter()
            .any(|pat| first.contains(pat.as_str()) || p.name.contains(pat.as_str()))
        {
            return false;
        }
    }
    true
}

pub fn passes_filter_expr(
    p: &Process,
    exprs: &[FilterExpr],
    cols: &[Box<dyn crate::column::Column>],
) -> bool {
    use std::str::FromStr;
    for e in exprs {
        let cell = cols
            .iter()
            .find(|c| {
                c.header().eq_ignore_ascii_case(&e.col)
                    || format!("{:?}", c.kind()).eq_ignore_ascii_case(&e.col)
            })
            .map(|c| {
                c.format(
                    p,
                    &crate::column::FormatCtx {
                        now: std::time::Instant::now(),
                        prev: None,
                    },
                )
            });
        let Some(cell) = cell else { return false; };
        let cell_num = u64::from_str(&cell).ok();
        let val_num = u64::from_str(&e.val).ok();
        let cmp = match (cell_num, val_num) {
            (Some(a), Some(b)) => a.cmp(&b),
            _ => cell.cmp(&e.val),
        };
        let ok = match e.op {
            FilterOp::Eq => cmp == std::cmp::Ordering::Equal,
            FilterOp::Ne => cmp != std::cmp::Ordering::Equal,
            FilterOp::Lt => cmp == std::cmp::Ordering::Less,
            FilterOp::Gt => cmp == std::cmp::Ordering::Greater,
        };
        if !ok {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{Process, ProcessFlags};
    use crate::state::ProcessState;
    use crate::user::User;

    fn p(pid: u32, name: &str) -> Process {
        Process {
            pid,
            ppid: 0,
            tgid: pid,
            session_id: 1,
            name: name.into(),
            image_path: String::new(),
            cmdline: None,
            user: User::default(),
            state: ProcessState::Running,
            priority_class: 0,
            nice: 10,
            start_time: None,
            kernel_time: Default::default(),
            user_time: Default::default(),
            thread_count: 1,
            handles: None,
            working_set: 0,
            peak_working_set: 0,
            virtual_size: 0,
            exit_status: None,
            flags: ProcessFlags::empty(),
        }
    }

    #[test]
    fn pid_filter() {
        let s = Selector {
            pids: vec![1, 2],
            explicit: true,
            ..Default::default()
        };
        assert!(passes(&p(1, "x"), &s));
        assert!(!passes(&p(3, "x"), &s));
    }
}
