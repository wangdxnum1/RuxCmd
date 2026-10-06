#![cfg(windows)]

use crate::process::Process;

pub struct Node {
    pub proc: Process,
    pub children: Vec<Node>,
}

pub struct Forest {
    pub roots: Vec<Node>,
}

pub fn build_forest(mut procs: Vec<Process>) -> Forest {
    procs.sort_by_key(|p| p.pid);
    let by_pid: std::collections::HashMap<u32, usize> =
        procs.iter().enumerate().map(|(i, p)| (p.pid, i)).collect();
    let mut children: std::collections::HashMap<u32, Vec<u32>> = std::collections::HashMap::new();
    for p in &procs {
        if p.ppid != 0 && by_pid.contains_key(&p.ppid) {
            children.entry(p.ppid).or_default().push(p.pid);
        }
    }
    let mut roots = Vec::new();
    for p in &procs {
        if p.ppid == 0 || !by_pid.contains_key(&p.ppid) {
            let node = build_node(p, &children, &procs);
            roots.push(node);
        }
    }
    Forest { roots }
}

fn build_node(
    p: &Process,
    children: &std::collections::HashMap<u32, Vec<u32>>,
    procs: &[Process],
) -> Node {
    let mut kids = Vec::new();
    if let Some(cids) = children.get(&p.pid) {
        for cid in cids {
            if let Some(child) = procs.iter().find(|x| x.pid == *cid) {
                kids.push(build_node(child, children, procs));
            }
        }
    }
    Node {
        proc: p.clone(),
        children: kids,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{Process, ProcessFlags};
    use crate::state::ProcessState;
    use crate::user::User;

    fn p(pid: u32, ppid: u32) -> Process {
        Process {
            pid,
            ppid,
            tgid: pid,
            session_id: 1,
            name: format!("p{pid}"),
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
    fn forest_groups_by_ppid() {
        let procs = vec![p(1, 0), p(2, 1), p(3, 1), p(4, 2)];
        let f = build_forest(procs);
        assert_eq!(f.roots.len(), 1);
        assert_eq!(f.roots[0].proc.pid, 1);
        assert_eq!(f.roots[0].children.len(), 2);
    }
}
