use crate::process::Process;

pub(super) fn pid(p: &Process) -> String {
    p.pid.to_string()
}

pub(super) fn ppid(p: &Process) -> String {
    p.ppid.to_string()
}

pub(super) fn pgid(p: &Process) -> String {
    p.tgid.to_string()
}

pub(super) fn sid(p: &Process) -> String {
    p.session_id.to_string()
}

pub(super) fn uid(p: &Process) -> String {
    if p.session_id == 0 {
        "SYSTEM".into()
    } else {
        p.user.sid.clone()
    }
}

pub(super) fn user(p: &Process) -> String {
    if p.session_id == 0 {
        "SYSTEM".into()
    } else if p.user.name.is_empty() {
        "?".into()
    } else {
        p.user.name.clone()
    }
}

pub(super) fn gid(p: &Process) -> String {
    p.user.sid.clone()
}

pub(super) fn ruser(p: &Process) -> String {
    p.user.name.clone()
}
