#![cfg(windows)]

use crate::process::Process;

#[derive(Debug, Clone, Default)]
pub struct Selector {
    pub pids: Vec<u32>,
    pub users: Vec<String>,
    pub session: Vec<u32>,
    pub comm_patterns: Vec<String>,
}

pub fn passes(p: &Process, sel: &Selector) -> bool {
    if !sel.pids.is_empty() && !sel.pids.contains(&p.pid) {
        return false;
    }
    if !sel.users.is_empty()
        && !sel
            .users
            .iter()
            .any(|u| u == &p.user.name || u == &p.user.sid)
    {
        return false;
    }
    if !sel.session.is_empty() && !sel.session.contains(&p.session_id) {
        return false;
    }
    if !sel.comm_patterns.is_empty() {
        let name = p.cmdline.as_deref().unwrap_or(&p.name);
        let first = name.split_whitespace().next().unwrap_or("");
        if !sel.comm_patterns.iter().any(|pat| {
            // procps -C matches the command exactly; Windows image names
            // carry ".exe" and arbitrary casing, so compare stems exactly.
            let stem = |s: &str| s.strip_suffix(".exe").unwrap_or(s).to_lowercase();
            let pat_stem = stem(pat.as_str());
            stem(first) == pat_stem || stem(&p.name) == pat_stem
        }) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::User;
    use crate::process::{Process, ProcessFlags};

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
            state: 'R',
            priority_class: 0,
            nice: 10,
            start_time: None,
            kernel_time: Default::default(),
            user_time: Default::default(),
            thread_count: 1,
            working_set: 0,
            virtual_size: 0,
            flags: ProcessFlags::empty(),
        }
    }

    #[test]
    fn pid_filter() {
        let s = Selector {
            pids: vec![1, 2],
            ..Default::default()
        };
        assert!(passes(&p(1, "x"), &s));
        assert!(!passes(&p(3, "x"), &s));
    }
}
