#![cfg_attr(test, allow(dead_code))]

use bitflags::bitflags;
use std::time::SystemTime;

use crate::state::ProcessState;
use crate::user::User;

bitflags! {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct ProcessFlags: u32 {
        const FOREGROUND = 1 << 0;
        const DEBUGGED   = 1 << 1;
        const ELEVATED   = 1 << 2;
        const WOW64      = 1 << 3;
    }
}

#[derive(Debug, Clone)]
pub struct Process {
    pub pid: u32,
    pub ppid: u32,
    pub tgid: u32,
    pub session_id: u32,
    pub name: String,
    pub image_path: String,
    pub cmdline: Option<String>,
    pub user: User,
    pub state: ProcessState,
    pub priority_class: u32,
    pub nice: i32,
    pub start_time: Option<SystemTime>,
    pub kernel_time: std::time::Duration,
    pub user_time: std::time::Duration,
    pub thread_count: u32,
    pub handles: Option<u32>,
    pub working_set: u64,
    pub peak_working_set: u64,
    pub virtual_size: u64,
    pub exit_status: Option<i32>,
    pub flags: ProcessFlags,
}
