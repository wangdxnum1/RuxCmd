#![cfg(windows)]

use std::time::{Duration, Instant};

use crate::process::Process;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnKind {
    Pid,
    Ppid,
    Pgid,
    Sid,
    Tid,
    Pgrp,
    Uid,
    User,
    Gid,
    Group,
    Ruser,
    Rgroup,
    Suser,
    Vsz,
    Rss,
    Pmem,
    Pcpu,
    Pri,
    Nice,
    Class,
    Etime,
    Etimes,
    Times,
    Time,
    Stime,
    Start,
    Lstart,
    Stat,
    Flags,
    Tty,
    Tt,
    Sess,
    Comm,
    Args,
    Cmd,
    Command,
    C,
    Cp,
    Thcount,
    Nlwp,
}

pub struct FormatCtx<'a> {
    pub now: Instant,
    pub prev: Option<&'a ProcessSample>,
}

#[derive(Debug, Clone, Copy)]
pub enum WidthMode {
    Wide,
    Truncate(usize),
}

#[derive(Debug, Clone)]
pub struct ProcessSample {
    pub pid: u32,
    pub cpu_total: Duration,
    pub at: Instant,
}

pub trait Column: Send + Sync {
    fn kind(&self) -> ColumnKind;
    fn header(&self) -> &'static str;
    fn width(&self) -> usize;
    fn format(&self, p: &Process, ctx: &FormatCtx) -> String;
    fn supported(&self) -> bool {
        true
    }
    fn align_right(&self) -> bool {
        false
    }
}

pub type BoxCol = Box<dyn Column>;

pub fn registry() -> Vec<(&'static str, fn() -> BoxCol)> {
    crate::columns::all()
}
