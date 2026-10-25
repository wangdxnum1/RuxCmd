//! Static output columns. Aliases keep their original metadata and ordering.
mod command;
mod identity;
mod resources;
mod time;
use crate::process::Process;

pub(crate) struct Column {
    name: &'static str,
    sort_name: &'static str,
    header: &'static str,
    width: usize,
    align_right: bool,
    formatter: fn(&Process) -> String,
}
impl Column {
    const fn new(
        name: &'static str,
        sort_name: &'static str,
        header: &'static str,
        width: usize,
        align_right: bool,
        formatter: fn(&Process) -> String,
    ) -> Self {
        Self {
            name,
            sort_name,
            header,
            width,
            align_right,
            formatter,
        }
    }
    pub(crate) fn name(&self) -> &'static str {
        self.name
    }
    pub(crate) fn header(&self) -> &'static str {
        self.header
    }
    pub(crate) fn width(&self) -> usize {
        self.width
    }
    pub(crate) fn align_right(&self) -> bool {
        self.align_right
    }
    pub(crate) fn format(&self, p: &Process) -> String {
        (self.formatter)(p)
    }
    pub(crate) fn matches_sort_key(&self, key: &str) -> bool {
        self.header.eq_ignore_ascii_case(key) || self.sort_name.eq_ignore_ascii_case(key)
    }
}

// These columns already render '?' in the one-shot command: no previous
// CPU sample, controlling TTY, or Unix group information is available.
fn unknown(_p: &Process) -> String {
    "?".into()
}

static COLUMNS: &[Column] = &[
    Column::new("pid", "Pid", "PID", 5, true, identity::pid),
    Column::new("ppid", "Ppid", "PPID", 5, true, identity::ppid),
    Column::new("pgid", "Pgid", "PGID", 5, true, identity::pgid),
    Column::new("sid", "Sid", "SID", 4, true, identity::sid),
    Column::new("tid", "Tid", "TID", 5, true, identity::pgid),
    Column::new("pgrp", "Pgrp", "PGRP", 5, true, identity::pgid),
    Column::new("user", "User", "USER", 12, false, identity::user),
    Column::new("uid", "Uid", "UID", 12, false, identity::uid),
    Column::new("gid", "Gid", "GID", 5, true, identity::gid),
    Column::new("group", "Group", "GROUP", 8, false, unknown),
    Column::new("ruser", "Ruser", "RUSER", 12, false, identity::ruser),
    Column::new("rgroup", "Rgroup", "RGROUP", 8, false, unknown),
    Column::new("suser", "Suser", "SUSER", 12, false, identity::ruser),
    Column::new("vsz", "Vsz", "VSZ", 7, true, resources::vsz),
    Column::new("rss", "Rss", "RSS", 7, true, resources::rss),
    Column::new("pmem", "Pmem", "%MEM", 5, true, resources::pmem),
    Column::new("pcpu", "Pcpu", "%CPU", 5, true, resources::pcpu),
    Column::new("pri", "Pri", "PRI", 3, true, resources::pri),
    Column::new("nice", "Nice", "NI", 3, true, resources::nice),
    Column::new("ni", "Nice", "NI", 3, true, resources::nice),
    Column::new("class", "Class", "CLASS", 6, false, resources::class),
    Column::new("etime", "Etime", "ELAPSED", 11, false, time::etime),
    Column::new("etimes", "Etimes", "ELAPSED", 7, true, time::etimes),
    Column::new("times", "Times", "TIMES", 11, false, time::times),
    Column::new("time", "Time", "TIME", 8, false, time::time),
    Column::new("stime", "Stime", "STIME", 5, false, time::stime),
    Column::new("start", "Start", "STARTED", 8, false, time::start),
    Column::new("lstart", "Lstart", "STARTED", 24, false, time::lstart),
    Column::new("stat", "Stat", "STAT", 4, false, resources::stat),
    Column::new("flags", "Flags", "F", 4, true, resources::flags),
    Column::new("f", "Flags", "F", 4, true, resources::flags),
    Column::new("tty", "Tty", "TTY", 3, false, unknown),
    Column::new("tt", "Tt", "TT", 3, false, unknown),
    Column::new("sess", "Sess", "SESS", 4, true, identity::sid),
    Column::new("comm", "Comm", "COMM", 16, false, command::comm),
    Column::new("args", "Args", "ARGS", 32, false, command::args),
    Column::new("cmd", "Cmd", "CMD", 256, false, command::args),
    Column::new("command", "Command", "COMMAND", 80, false, command::args),
    Column::new("c", "C", "C", 3, true, unknown),
    Column::new("cp", "Cp", "CP", 4, true, unknown),
    Column::new("thcount", "Thcount", "THCNT", 5, true, resources::thcount),
    Column::new("nlwp", "Nlwp", "NLWP", 5, true, resources::thcount),
];
pub(crate) fn all() -> &'static [Column] {
    COLUMNS
}
pub(crate) fn lookup(name: &str) -> Option<&'static Column> {
    COLUMNS.iter().find(|column| column.name == name)
}
#[cfg(test)]
mod tests;
