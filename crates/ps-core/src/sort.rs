#![cfg(windows)]

use crate::column::FormatCtx;
use crate::process::Process;

#[derive(Debug, Clone, Default)]
pub struct SortSpec {
    pub keys: Vec<SortKey>,
}

#[derive(Debug, Clone)]
pub struct SortKey {
    pub name: String,
    pub reverse: bool,
}

pub fn parse_sort(spec: &[String]) -> SortSpec {
    let mut keys = Vec::new();
    for s in spec {
        for k in s.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let (name, reverse) = if let Some(rest) = k.strip_prefix('-') {
                (rest.to_string(), true)
            } else {
                (k.to_string(), false)
            };
            keys.push(SortKey { name, reverse });
        }
    }
    SortSpec { keys }
}

pub fn apply_sort(
    mut procs: Vec<Process>,
    spec: &SortSpec,
    cols: &[Box<dyn crate::column::Column>],
) -> Vec<Process> {
    if spec.keys.is_empty() {
        procs.sort_by_key(|p| p.pid);
        return procs;
    }
    let keys = spec.keys.clone();
    procs.sort_by(|a, b| {
        for k in &keys {
            let Some(ca) = find_col(cols, &k.name) else {
                continue;
            };
            let Some(cb) = find_col(cols, &k.name) else {
                continue;
            };
            let ctx = FormatCtx {
                now: std::time::Instant::now(),
                prev: None,
            };
            let sa = ca.format(a, &ctx);
            let sb = cb.format(b, &ctx);
            let ord = natural_cmp(&sa, &sb);
            let ord = if k.reverse { ord.reverse() } else { ord };
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        a.pid.cmp(&b.pid)
    });
    procs
}

fn find_col<'a>(
    cols: &'a [Box<dyn crate::column::Column>],
    name: &str,
) -> Option<&'a Box<dyn crate::column::Column>> {
    cols.iter().find(|c| {
        c.header().eq_ignore_ascii_case(name)
            || format!("{:?}", c.kind()).eq_ignore_ascii_case(name)
    })
}

fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let (an, bn) = (a.parse::<u64>().ok(), b.parse::<u64>().ok());
    match (an, bn) {
        (Some(x), Some(y)) => x.cmp(&y),
        _ => a.cmp(b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{Process, ProcessFlags};
    use crate::state::ProcessState;
    use crate::user::User;

    fn p(pid: u32) -> Process {
        Process {
            pid,
            ppid: 0,
            tgid: pid,
            session_id: 1,
            name: "x".into(),
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
    fn default_sort_by_pid_asc() {
        let mut v = vec![p(3), p(1), p(2)];
        v = apply_sort(v, &SortSpec::default(), &[]);
        assert_eq!(v.iter().map(|x| x.pid).collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn parse_sort_with_reverse() {
        let s = parse_sort(&["-pcpu".into(), "pid".into()]);
        assert_eq!(s.keys.len(), 2);
        assert!(s.keys[0].reverse);
        assert!(!s.keys[1].reverse);
    }
}
