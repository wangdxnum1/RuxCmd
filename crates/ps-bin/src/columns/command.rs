use crate::process::Process;

pub(super) fn comm(p: &Process) -> String {
    p.name.clone()
}

pub(super) fn args(p: &Process) -> String {
    p.cmdline.clone().unwrap_or_else(|| p.image_path.clone())
}
