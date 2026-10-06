#![cfg(windows)]

use crate::column::BoxCol;

mod args;
mod c;
mod class;
mod cmd;
mod comm;
mod command;
mod cp;
mod etime;
mod etimes;
mod flags;
mod gid;
mod group;
mod lstart;
mod nice;
mod nlwp;
mod pcpu;
mod pgid;
mod pgrp;
mod pid;
mod pmem;
mod ppid;
mod pri;
mod rgroup;
mod rss;
mod ruser;
mod sess;
mod sid;
mod start;
mod stat;
mod stime;
mod suser;
mod thcount;
mod tid;
mod time;
mod times;
mod tt;
mod tty;
mod uid;
mod user;
mod vsz;

pub fn all() -> Vec<(&'static str, fn() -> BoxCol)> {
    vec![
        ("pid", || Box::new(pid::Pid)),
        ("ppid", || Box::new(ppid::Ppid)),
        ("pgid", || Box::new(pgid::Pgid)),
        ("sid", || Box::new(sid::Sid)),
        ("tid", || Box::new(tid::Tid)),
        ("pgrp", || Box::new(pgrp::Pgrp)),
        ("user", || Box::new(user::UserCol)),
        ("uid", || Box::new(uid::Uid)),
        ("gid", || Box::new(gid::Gid)),
        ("group", || Box::new(group::Group)),
        ("ruser", || Box::new(ruser::Ruser)),
        ("rgroup", || Box::new(rgroup::Rgroup)),
        ("suser", || Box::new(suser::Suser)),
        ("vsz", || Box::new(vsz::Vsz)),
        ("rss", || Box::new(rss::Rss)),
        ("pmem", || Box::new(pmem::Pmem)),
        ("pcpu", || Box::new(pcpu::Pcpu)),
        ("pri", || Box::new(pri::Pri)),
        ("nice", || Box::new(nice::Nice)),
        ("ni", || Box::new(nice::Nice)),
        ("class", || Box::new(class::Class)),
        ("etime", || Box::new(etime::Etime)),
        ("etimes", || Box::new(etimes::Etimes)),
        ("times", || Box::new(times::Times)),
        ("time", || Box::new(time::Time)),
        ("stime", || Box::new(stime::Stime)),
        ("start", || Box::new(start::Start)),
        ("lstart", || Box::new(lstart::Lstart)),
        ("stat", || Box::new(stat::Stat)),
        ("flags", || Box::new(flags::Flags)),
        ("f", || Box::new(flags::Flags)),
        ("tty", || Box::new(tty::Tty)),
        ("tt", || Box::new(tt::Tt)),
        ("sess", || Box::new(sess::Sess)),
        ("comm", || Box::new(comm::Comm)),
        ("args", || Box::new(args::Args)),
        ("cmd", || Box::new(cmd::Cmd)),
        ("command", || Box::new(command::Command)),
        ("c", || Box::new(c::C)),
        ("cp", || Box::new(cp::Cp)),
        ("thcount", || Box::new(thcount::Thcount)),
        ("nlwp", || Box::new(nlwp::Nlwp)),
    ]
}

pub fn lookup(name: &str) -> Option<fn() -> BoxCol> {
    all().into_iter().find(|(k, _)| *k == name).map(|(_, f)| f)
}

pub fn parse_list(s: &str) -> Result<Vec<BoxCol>, String> {
    let mut out = Vec::new();
    for raw in s.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let factory = lookup(raw).ok_or_else(|| format!("unknown column: {raw}"))?;
        out.push(factory());
    }
    Ok(out)
}

pub fn col_kind(name: &str) -> Option<crate::column::ColumnKind> {
    lookup(name).map(|f| f().kind())
}
