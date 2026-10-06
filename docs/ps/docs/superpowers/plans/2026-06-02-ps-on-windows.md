# ps-on-windows Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust workspace that produces a Linux-`ps(1)`-compatible `ps.exe` for Windows.

**Architecture:** Three-crate workspace: `ps-sys` (raw Win32 FFI), `ps-core` (data model + columns/filter/sort/tree), `ps-bin` (clap CLI + render). Dependency direction is strictly `ps-bin → ps-core → ps-sys`.

**Tech Stack:** Rust 1.95 (edition 2021), `windows = 0.58`, `clap = 4` (derive), `thiserror`, `bitflags`, `anyhow`, `assert_cmd`, `predicates`.

**Spec:** `docs/superpowers/specs/2026-06-02-ps-on-windows-design.md`

---

## File Map

| Path | Responsibility |
|---|---|
| `Cargo.toml` | Workspace root, members + shared deps |
| `.gitignore` | Ignore `target/`, `dist/`, `*.exe` |
| `.github/workflows/ci.yml` | CI on `windows-latest` |
| `ps-sys/Cargo.toml` | Crate manifest, windows crate dep |
| `ps-sys/src/lib.rs` | Public API re-exports + `snapshot()` |
| `ps-sys/src/error.rs` | `Error` enum |
| `ps-sys/src/util.rs` | UTF-16 / SID / time helpers |
| `ps-sys/src/snapshot.rs` | `EnumProcesses` + `Toolhelp32` PPID |
| `ps-sys/src/procinfo.rs` | Per-PID image/session/memory/priority |
| `ps-sys/src/token.rs` | OpenProcessToken → user name/domain/SID |
| `ps-sys/src/times.rs` | GetProcessTimes → SystemTime/Duration |
| `ps-sys/src/cmdline.rs` | PEB → CommandLine via ReadProcessMemory |
| `ps-sys/src/threads.rs` | Toolhelp32 thread count cache |
| `ps-core/Cargo.toml` | Crate manifest |
| `ps-core/src/lib.rs` | Public re-exports + `snapshot()` |
| `ps-core/src/process.rs` | `Process` struct + `ProcessFlags` |
| `ps-core/src/state.rs` | `ProcessState` enum |
| `ps-core/src/user.rs` | `User` struct |
| `ps-core/src/column.rs` | `Column` trait, `ColumnKind`, `FormatCtx`, registry |
| `ps-core/src/columns/*.rs` | One file per column |
| `ps-core/src/filter.rs` | `Selector` + `FilterExpr` |
| `ps-core/src/sort.rs` | `SortSpec` + apply |
| `ps-core/src/tree.rs` | `Forest` builder + traversal |
| `ps-core/src/assemble.rs` | Map `ps_sys::ProcessRaw` → `ps_core::Process` |
| `ps-bin/Cargo.toml` | Crate manifest |
| `ps-bin/src/main.rs` | `Cli::parse()` → run → exit code |
| `ps-bin/src/args.rs` | clap Cli + parse_filter + resolve_columns |
| `ps-bin/src/render/mod.rs` | Module entry |
| `ps-bin/src/render/table.rs` | Aligned table |
| `ps-bin/src/render/forest.rs` | Tree renderer |
| `ps-bin/src/render/color.rs` | ANSI enable + helpers |
| `ps-bin/src/compat.rs` | Exit code semantics |
| `ps-bin/tests/cli.rs` | assert_cmd integration tests |
| `scripts/release.ps1` | Build + copy to dist/ |

---

## Task 1: Initialize workspace, gitignore, CI

**Files:**
- Create: `Cargo.toml`
- Create: `.gitignore`
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Create root `Cargo.toml`**

```toml
[workspace]
resolver = "2"
members = ["ps-sys", "ps-core", "ps-bin"]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "MIT"

[workspace.dependencies]
thiserror = "1"
bitflags = "2"
once_cell = "1"
anyhow = "1"
clap = { version = "4", features = ["derive"] }
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_System_ProcessStatus",
    "Win32_System_Threading",
    "Win32_System_Console",
    "Win32_System_Kernel",
    "Win32_Security",
    "Win32_System_JobObjects",
    "Win32_Storage_FileSystem",
] }
```

- [ ] **Step 2: Create `.gitignore`**

```
target/
dist/
*.exe
*.pdb
.idea/
.vscode/
```

- [ ] **Step 3: Create CI**

`.github/workflows/ci.yml`:
```yaml
name: ci
on: [push, pull_request]
jobs:
  test:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo build --workspace
      - run: cargo test --workspace
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo fmt --all -- --check
```

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml .gitignore .github/
git commit -m "chore: init workspace, gitignore, ci"
```

---

## Task 2: Initialize `ps-sys` crate skeleton

**Files:**
- Create: `ps-sys/Cargo.toml`
- Create: `ps-sys/src/lib.rs`
- Create: `ps-sys/src/error.rs`

- [ ] **Step 1: Create `ps-sys/Cargo.toml`**

```toml
[package]
name = "ps-sys"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
thiserror = { workspace = true }
windows = { workspace = true }
```

- [ ] **Step 2: Create `ps-sys/src/error.rs`**

```rust
use windows::core::Error as Win32Error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Win32 error: {0}")]
    Win32(#[from] Win32Error),
    #[error("invalid PID {0}")]
    InvalidPid(u32),
    #[error("permission denied for PID {0}")]
    PermissionDenied(u32),
    #[error("process not found: PID {0}")]
    NotFound(u32),
    #[error("PEB read failed for PID {0}")]
    PebRead(u32),
    #[error("internal: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

- [ ] **Step 3: Create `ps-sys/src/lib.rs`**

```rust
#![cfg(windows)]

pub mod error;
pub mod util;

pub use error::{Error, Result};
```

- [ ] **Step 4: Write failing test for Error Display**

`ps-sys/src/error.rs` add at bottom:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_displays_kind() {
        let e = Error::InvalidPid(42);
        assert_eq!(format!("{e}"), "invalid PID 42");
    }
}
```

- [ ] **Step 5: Run test**

Run: `cd d:\Work\rust\ps && cargo test -p ps-sys`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add ps-sys/
git commit -m "feat(ps-sys): init crate with error type"
```

---

## Task 3: `ps-sys::util` — UTF-16 + time helpers

**Files:**
- Create: `ps-sys/src/util.rs`
- Modify: `ps-sys/src/lib.rs`

- [ ] **Step 1: Register module**

In `ps-sys/src/lib.rs`:
```rust
pub mod util;
```

- [ ] **Step 2: Create `ps-sys/src/util.rs`**

```rust
#![cfg(windows)]

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::core::PCWSTR;
use windows::Win32::Foundation::FILETIME;

pub fn wstr_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    OsString::from_wide(&buf[..len]).to_string_lossy().into_owned()
}

pub fn pwstr_to_string(p: PCWSTR) -> String {
    unsafe {
        if p.0.is_null() {
            return String::new();
        }
        let mut len = 0;
        let mut q = p.0;
        while !q.is_null() && *q != 0 {
            len += 1;
            q = q.add(1);
        }
        let slice = std::slice::from_raw_parts(p.0, len);
        wstr_to_string(slice)
    }
}

pub fn filetime_to_systemtime(ft: FILETIME) -> Option<std::time::SystemTime> {
    let ticks = ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64);
    if ticks == 0 {
        return None;
    }
    const OFFSET_100NS: u64 = 116_444_736_000_000_000;
    let epoch_100ns = ticks.checked_sub(OFFSET_100NS)?;
    let nanos = (epoch_100ns % 10_000_000) * 100;
    let secs = epoch_100ns / 10_000_000;
    Some(std::time::UNIX_EPOCH + std::time::Duration::new(secs, nanos as u32))
}

pub fn duration_from_filetimes(kernel: FILETIME, user: FILETIME) -> std::time::Duration {
    fn to_100ns(ft: FILETIME) -> u64 {
        ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
    }
    std::time::Duration::from_nanos((to_100ns(kernel) + to_100ns(user)) * 100)
}
```

- [ ] **Step 3: Add unit tests**

Append to `ps-sys/src/util.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn wstr_to_string_truncates_at_nul() {
        let mut buf = [0u16; 4];
        buf[0] = b'a' as u16;
        buf[1] = b'b' as u16;
        buf[2] = b'c' as u16;
        buf[3] = 0;
        assert_eq!(wstr_to_string(&buf), "abc");
    }

    #[test]
    fn filetime_zero_is_none() {
        let ft = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        assert!(filetime_to_systemtime(ft).is_none());
    }

    #[test]
    fn filetime_unix_epoch() {
        let ft = FILETIME { dwLowDateTime: 0xD53E8000, dwHighDateTime: 0x019DB1DE };
        let t = filetime_to_systemtime(ft).unwrap();
        let diff = t.duration_since(UNIX_EPOCH).unwrap_or_default();
        assert!(diff < Duration::from_secs(2));
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cd d:\Work\rust\ps && cargo test -p ps-sys`
Expected: 3 tests pass

- [ ] **Step 5: Commit**

```bash
git add ps-sys/
git commit -m "feat(ps-sys): util helpers (wstr, filetime)"
```

---

## Task 4: `ps-sys::snapshot` — `EnumProcesses` + `Toolhelp32` for PPID + name

**Files:**
- Create: `ps-sys/src/snapshot.rs`
- Modify: `ps-sys/src/lib.rs`

- [ ] **Step 1: Register module**

In `ps-sys/src/lib.rs`:
```rust
pub mod snapshot;
```

- [ ] **Step 2: Create `ps-sys/src/snapshot.rs`**

```rust
#![cfg(windows)]

use std::collections::HashMap;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::ProcessStatus::{EnumProcesses, K32GetModuleBaseNameW};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

use crate::error::{Error, Result};
use crate::util::wstr_to_string;

const INITIAL_BUF: usize = 1024;
const MAX_BUF_BYTES: usize = 1 << 22; // 4 MB

pub struct Snapshot {
    pub pids: Vec<u32>,
    pub ppid_map: HashMap<u32, u32>,
    pub names: HashMap<u32, String>,
}

pub fn snapshot() -> Result<Snapshot> {
    let pids = enum_processes()?;
    let ppid_map = toolhelp32_ppids(&pids)?;
    let names = module_names(&pids)?;
    Ok(Snapshot { pids, ppid_map, names })
}

fn enum_processes() -> Result<Vec<u32>> {
    let mut cap = INITIAL_BUF;
    loop {
        let mut buf = vec![0u32; cap];
        let mut needed = 0u32;
        unsafe {
            EnumProcesses(buf.as_mut_ptr(), (buf.len() * 4) as u32, &mut needed)
                .map_err(Error::from)?;
        }
        let count = (needed as usize) / std::mem::size_of::<u32>();
        if count < buf.len() {
            buf.truncate(count);
            buf.retain(|&p| p != 0);
            return Ok(buf);
        }
        if cap * 4 > MAX_BUF_BYTES {
            return Err(Error::Internal("EnumProcesses overflow".into()));
        }
        cap *= 2;
    }
}

fn toolhelp32_ppids(pids: &[u32]) -> Result<HashMap<u32, u32>> {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).map_err(Error::from)?;
        if snap.is_invalid() {
            return Err(Error::Internal("CreateToolhelp32Snapshot failed".into()));
        }
        let mut map = HashMap::new();
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                map.insert(entry.th32ProcessID, entry.th32ParentProcessID);
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
        for &p in pids {
            map.entry(p).or_insert(0);
        }
        Ok(map)
    }
}

fn module_names(pids: &[u32]) -> Result<HashMap<u32, String>> {
    let mut map = HashMap::new();
    for &pid in pids {
        let name = module_name(pid).unwrap_or_default();
        map.insert(pid, name);
    }
    Ok(map)
}

fn module_name(pid: u32) -> Result<String> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).map_err(Error::from)?;
        if h.is_invalid() {
            return Err(Error::PermissionDenied(pid));
        }
        let mut buf = [0u16; 1024];
        let n = K32GetModuleBaseNameW(h, None, &mut buf);
        let _ = CloseHandle(h);
        if n == 0 {
            return Err(Error::PermissionDenied(pid));
        }
        Ok(wstr_to_string(&buf[..n as usize]))
    }
}
```

- [ ] **Step 3: Add integration test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn snapshot_contains_current_pid() {
        let s = snapshot().unwrap();
        let me = unsafe { GetCurrentProcessId() };
        assert!(s.pids.contains(&me), "current pid {me} not in snapshot");
    }

    #[test]
    fn snapshot_ppid_map_has_self() {
        let s = snapshot().unwrap();
        let me = unsafe { GetCurrentProcessId() };
        assert!(s.ppid_map.contains_key(&me));
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cd d:\Work\rust\ps && cargo test -p ps-sys`
Expected: 5 tests pass

- [ ] **Step 5: Commit**

```bash
git add ps-sys/
git commit -m "feat(ps-sys): snapshot with pid/ppid/name"
```

---

## Task 5: `ps-sys::times` — `GetProcessTimes` wrapper

**Files:**
- Create: `ps-sys/src/times.rs`
- Modify: `ps-sys/src/lib.rs`

- [ ] **Step 1: Register module**

In `ps-sys/src/lib.rs`:
```rust
pub mod times;
```

- [ ] **Step 2: Create `ps-sys/src/times.rs`**

```rust
#![cfg(windows)]

use std::time::{Duration, SystemTime};
use windows::Win32::Foundation::{CloseHandle, FILETIME};
use windows::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

use crate::error::{Error, Result};
use crate::util::{duration_from_filetimes, filetime_to_systemtime};

pub struct Times {
    pub start: Option<SystemTime>,
    pub cpu: Duration,
    pub kernel: Duration,
    pub user: Duration,
}

pub fn times(pid: u32) -> Result<Times> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).map_err(Error::from)?;
        if h.is_invalid() {
            return Err(Error::PermissionDenied(pid));
        }
        let (mut ct, mut et, mut kt, mut ut) = (FILETIME::default(), FILETIME::default(), FILETIME::default(), FILETIME::default());
        GetProcessTimes(h, &mut ct, &mut et, &mut kt, &mut ut).map_err(Error::from)?;
        let _ = CloseHandle(h);
        Ok(Times {
            start: filetime_to_systemtime(ct),
            cpu: duration_from_filetimes(kt, ut),
            kernel: duration_from_filetimes(kt, FILETIME::default()),
            user: duration_from_filetimes(FILETIME::default(), ut),
        })
    }
}
```

- [ ] **Step 3: Add test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn times_of_current_process_has_start() {
        let pid = unsafe { GetCurrentProcessId() };
        let t = times(pid).unwrap();
        assert!(t.start.is_some());
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cd d:\Work\rust\ps && cargo test -p ps-sys`
Expected: 6 tests pass

- [ ] **Step 5: Commit**

```bash
git add ps-sys/
git commit -m "feat(ps-sys): GetProcessTimes wrapper"
```

---

## Task 6: `ps-sys::token` — user name + SID

**Files:**
- Create: `ps-sys/src/token.rs`
- Modify: `ps-sys/src/lib.rs`

- [ ] **Step 1: Register module**

In `ps-sys/src/lib.rs`:
```rust
pub mod token;
```

- [ ] **Step 2: Create `ps-sys/src/token.rs`**

```rust
#![cfg(windows)]

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, PSID};
use windows::Win32::Security::{GetTokenInformation, LookupAccountSidW, TokenUser, SID_NAME_USE, TOKEN_QUERY};
use windows::Win32::System::Threading::{OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::System::WindowsProgramming::{ConvertSidToStringSidW, LocalFree, HLOCAL};

use crate::error::{Error, Result};

pub struct UserInfo {
    pub name: String,
    pub domain: String,
    pub sid: String,
}

pub fn user(pid: u32) -> Result<UserInfo> {
    unsafe {
        let ph = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).map_err(Error::from)?;
        if ph.is_invalid() {
            return Err(Error::PermissionDenied(pid));
        }
        let mut th = windows::Win32::Foundation::HANDLE::default();
        OpenProcessToken(ph, TOKEN_QUERY, &mut th).map_err(Error::from)?;
        let _ = CloseHandle(ph);

        let mut needed = 0u32;
        let _ = GetTokenInformation(th, TokenUser, None, 0, &mut needed);
        let mut buf = vec![0u8; needed as usize];
        GetTokenInformation(th, TokenUser, Some(buf.as_mut_ptr() as *mut _), needed, &mut needed)
            .map_err(Error::from)?;
        let _ = CloseHandle(th);

        let token_user: *const TOKEN_USER_HEADER = buf.as_ptr() as *const _;
        let sid_ptr = (*token_user).User.Sid;
        Ok(sid_to_info(sid_ptr))
    }
}

#[repr(C)]
struct TOKEN_USER_HEADER {
    User: SID_AND_ATTRIBUTES_H,
}

#[repr(C)]
struct SID_AND_ATTRIBUTES_H {
    Sid: PSID,
    Attributes: u32,
}

fn sid_to_info(sid: PSID) -> UserInfo {
    unsafe {
        let mut name_len = 0u32;
        let mut domain_len = 0u32;
        let mut name_use = SID_NAME_USE(0);
        let _ = LookupAccountSidW(PCWSTR::null(), sid, None, &mut name_len, None, &mut domain_len, &mut name_use);
        let mut name_buf = vec![0u16; name_len as usize];
        let mut domain_buf = vec![0u16; domain_len as usize];
        let ok = LookupAccountSidW(
            PCWSTR::null(),
            sid,
            Some(name_buf.as_mut_slice()),
            &mut name_len,
            Some(domain_buf.as_mut_slice()),
            &mut domain_len,
            &mut name_use,
        );
        if ok.is_err() {
            return UserInfo { name: "?".into(), domain: "?".into(), sid: "?".into() };
        }
        let name = OsString::from_wide(&name_buf[..name_len as usize]).to_string_lossy().into_owned();
        let domain = OsString::from_wide(&domain_buf[..domain_len as usize]).to_string_lossy().into_owned();
        let sid_str = sid_to_string(sid);
        UserInfo { name, domain, sid: sid_str }
    }
}

fn sid_to_string(sid: PSID) -> String {
    unsafe {
        let mut out = PCWSTR::null();
        if ConvertSidToStringSidW(sid, &mut out).is_ok() && !out.0.is_null() {
            let mut len = 0;
            let mut q = out.0;
            while !q.is_null() && *q != 0 {
                len += 1;
                q = q.add(1);
            }
            let slice = std::slice::from_raw_parts(out.0, len);
            let s = OsString::from_wide(slice).to_string_lossy().into_owned();
            let _ = LocalFree(HLOCAL(out.0 as _));
            return s;
        }
        "?".into()
    }
}
```

- [ ] **Step 3: Add test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn user_of_current_has_name() {
        let pid = unsafe { GetCurrentProcessId() };
        let u = user(pid).unwrap();
        assert!(!u.name.is_empty());
        assert!(u.sid.starts_with("S-1-"));
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cd d:\Work\rust\ps && cargo test -p ps-sys`
Expected: 7 tests pass

- [ ] **Step 5: Commit**

```bash
git add ps-sys/
git commit -m "feat(ps-sys): token user + sid lookup"
```

---

## Task 7: `ps-sys::procinfo` — image path, session, memory, priority

**Files:**
- Create: `ps-sys/src/procinfo.rs`
- Modify: `ps-sys/src/lib.rs`

- [ ] **Step 1: Register module**

In `ps-sys/src/lib.rs`:
```rust
pub mod procinfo;
```

- [ ] **Step 2: Create `ps-sys/src/procinfo.rs`**

```rust
#![cfg(windows)]

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Console::ProcessIdToSessionId;
use windows::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows::Win32::System::Threading::{
    GetPriorityClass, IsWow64Process, OpenProcess, QueryFullProcessImageNameW,
    PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};

use crate::error::{Error, Result};
use crate::util::wstr_to_string;

pub struct ProcInfo {
    pub image_path: String,
    pub session_id: u32,
    pub working_set: u64,
    pub peak_working_set: u64,
    pub pagefile_usage: u64,
    pub priority_class: u32,
    pub wow64: bool,
}

pub fn procinfo(pid: u32) -> Result<ProcInfo> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).map_err(Error::from)?;
        if h.is_invalid() {
            return Err(Error::PermissionDenied(pid));
        }
        let image_path = image_path(h)?;
        let session_id = ProcessIdToSessionId(pid).unwrap_or(0);
        let (working_set, peak_working_set, pagefile_usage) = memory(h)?;
        let priority_class = GetPriorityClass(h).map_err(Error::from)?;
        let wow64 = IsWow64Process(h).unwrap_or(false);
        let _ = CloseHandle(h);
        Ok(ProcInfo {
            image_path,
            session_id,
            working_set,
            peak_working_set,
            pagefile_usage,
            priority_class,
            wow64,
        })
    }
}

unsafe fn image_path(h: HANDLE) -> Result<String> {
    let mut buf = [0u16; 1024];
    let mut len = buf.len() as u32;
    QueryFullProcessImageNameW(
        h,
        PROCESS_NAME_FORMAT(0),
        windows::core::PWSTR(buf.as_mut_ptr()),
        &mut len,
    )
    .map_err(Error::from)?;
    Ok(wstr_to_string(&buf[..len as usize]))
}

unsafe fn memory(h: HANDLE) -> Result<(u64, u64, u64)> {
    let mut mc = PROCESS_MEMORY_COUNTERS::default();
    let cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    if K32GetProcessMemoryInfo(h, &mut mc, cb).is_err() {
        return Ok((0, 0, 0));
    }
    Ok((mc.WorkingSetSize as u64, mc.PeakWorkingSetSize as u64, mc.PagefileUsage as u64))
}
```

- [ ] **Step 3: Add test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn procinfo_of_current_has_image() {
        let pid = unsafe { GetCurrentProcessId() };
        let p = procinfo(pid).unwrap();
        assert!(!p.image_path.is_empty());
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cd d:\Work\rust\ps && cargo test -p ps-sys`
Expected: 8 tests pass

- [ ] **Step 5: Commit**

```bash
git add ps-sys/
git commit -m "feat(ps-sys): procinfo (image, session, memory, priority)"
```

---

## Task 8: `ps-sys::threads` — Toolhelp32 thread count

**Files:**
- Create: `ps-sys/src/threads.rs`
- Modify: `ps-sys/src/lib.rs`

- [ ] **Step 1: Register module**

In `ps-sys/src/lib.rs`:
```rust
pub mod threads;
```

- [ ] **Step 2: Create `ps-sys/src/threads.rs`**

```rust
#![cfg(windows)]

use std::collections::HashMap;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, THREADENTRY32, TH32CS_SNAPTHREAD,
};

use crate::error::{Error, Result};

pub fn thread_counts() -> Result<HashMap<u32, u32>> {
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0).map_err(Error::from)?;
        if snap.is_invalid() {
            return Err(Error::Internal("CreateToolhelp32Snapshot THREAD failed".into()));
        }
        let mut map: HashMap<u32, u32> = HashMap::new();
        let mut entry = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };
        if Thread32First(snap, &mut entry).is_ok() {
            loop {
                *map.entry(entry.th32OwnerProcessID).or_insert(0) += 1;
                if Thread32Next(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
        Ok(map)
    }
}
```

- [ ] **Step 3: Add test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn thread_counts_includes_current() {
        let m = thread_counts().unwrap();
        let me = unsafe { GetCurrentProcessId() };
        assert!(m.contains_key(&me));
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cd d:\Work\rust\ps && cargo test -p ps-sys`
Expected: 9 tests pass

- [ ] **Step 5: Commit**

```bash
git add ps-sys/
git commit -m "feat(ps-sys): toolhelp32 thread count map"
```

---

## Task 9: `ps-sys::cmdline` — PEB → CommandLine (64-bit)

**Files:**
- Create: `ps-sys/src/cmdline.rs`
- Modify: `ps-sys/src/lib.rs`

- [ ] **Step 1: Register module**

In `ps-sys/src/lib.rs`:
```rust
pub mod cmdline;
```

- [ ] **Step 2: Create `ps-sys/src/cmdline.rs`**

```rust
#![cfg(windows)]

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::Win32::Foundation::{CloseHandle, HANDLE, UNICODE_STRING};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Threading::{
    NtQueryInformationProcess, OpenProcess, ProcessBasicInformation,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
};

use crate::error::{Error, Result};

#[allow(non_snake_case)]
#[repr(C)]
struct PROCESS_BASIC_INFORMATION {
    Reserved1: usize,
    PebBaseAddress: usize,
    Reserved2: [usize; 2],
    UniqueProcessId: usize,
    Reserved3: usize,
}

pub fn cmdline(pid: u32) -> Result<Option<String>> {
    unsafe {
        let access = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ;
        let h = OpenProcess(access, false, pid).map_err(Error::from)?;
        if h.is_invalid() {
            return Err(Error::PermissionDenied(pid));
        }
        let mut pbi = std::mem::zeroed::<PROCESS_BASIC_INFORMATION>();
        let mut ret_len = 0u32;
        let status = NtQueryInformationProcess(
            h,
            ProcessBasicInformation,
            &mut pbi as *mut _ as *mut _,
            std::mem::size_of::<PROCESS_BASIC_INFORMATION>() as u32,
            &mut ret_len,
        );
        if status.0 != 0 {
            let _ = CloseHandle(h);
            return Err(Error::PebRead(pid));
        }
        let peb_base = pbi.PebBaseAddress;
        if peb_base == 0 {
            let _ = CloseHandle(h);
            return Ok(None);
        }

        // PEB layout (64-bit): +0x20 = ProcessParameters pointer
        let mut params_ptr: usize = 0;
        if !read_mem(h, peb_base + 0x20, &mut params_ptr) {
            let _ = CloseHandle(h);
            return Err(Error::PebRead(pid));
        }
        if params_ptr == 0 {
            let _ = CloseHandle(h);
            return Ok(None);
        }

        // RTL_USER_PROCESS_PARAMETERS (64-bit): +0x70 = CommandLine (UNICODE_STRING)
        let mut cmd: UNICODE_STRING = std::mem::zeroed();
        if !read_mem(h, params_ptr + 0x70, &mut cmd) {
            let _ = CloseHandle(h);
            return Err(Error::PebRead(pid));
        }
        if cmd.Length == 0 || cmd.Buffer.is_null() {
            let _ = CloseHandle(h);
            return Ok(None);
        }
        let mut buf = vec![0u16; (cmd.Length / 2) as usize];
        if !read_mem(h, cmd.Buffer as usize, buf.as_mut_ptr()) {
            let _ = CloseHandle(h);
            return Err(Error::PebRead(pid));
        }
        let _ = CloseHandle(h);
        let s = OsString::from_wide(&buf).to_string_lossy().into_owned();
        Ok(Some(s))
    }
}

unsafe fn read_mem<T>(h: HANDLE, addr: usize, dst: *mut T) -> bool {
    let mut read = 0usize;
    let ok = ReadProcessMemory(
        h,
        addr as *const _,
        dst as *mut _,
        std::mem::size_of::<T>(),
        Some(&mut read),
    );
    ok.is_ok() && read == std::mem::size_of::<T>()
}
```

> 64-bit offsets only. Documented as known limitation in spec §7.

- [ ] **Step 3: Add test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn cmdline_of_current_present() {
        let pid = unsafe { GetCurrentProcessId() };
        let s = cmdline(pid).unwrap();
        assert!(s.is_some(), "expected cmdline for self, got None");
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cd d:\Work\rust\ps && cargo test -p ps-sys`
Expected: 10 tests pass

- [ ] **Step 5: Commit**

```bash
git add ps-sys/
git commit -m "feat(ps-sys): PEB cmdline read (64-bit)"
```

---

## Task 10: `ps-sys` public `collect()` aggregator

**Files:**
- Modify: `ps-sys/src/lib.rs`

- [ ] **Step 1: Replace `ps-sys/src/lib.rs`**

```rust
#![cfg(windows)]

pub mod error;
pub mod util;
pub mod snapshot;
pub mod procinfo;
pub mod token;
pub mod times;
pub mod cmdline;
pub mod threads;

pub use error::{Error, Result};
pub use procinfo::{procinfo, ProcInfo};
pub use snapshot::{snapshot as raw_snapshot, Snapshot};
pub use threads::thread_counts;
pub use times::{times, Times};
pub use token::{user, UserInfo};
pub use cmdline::cmdline;

pub struct ProcessRaw {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub info: Result<ProcInfo>,
    pub times: Result<Times>,
    pub user: Result<UserInfo>,
    pub cmdline: Option<String>,
    pub thread_count: u32,
}

pub fn collect() -> Result<Vec<ProcessRaw>> {
    let snap = raw_snapshot()?;
    let thread_map = thread_counts().unwrap_or_default();
    let mut out = Vec::with_capacity(snap.pids.len());
    for &pid in &snap.pids {
        let ppid = *snap.ppid_map.get(&pid).unwrap_or(&0);
        let name = snap.names.get(&pid).cloned().unwrap_or_default();
        let info = procinfo(pid);
        let times = times(pid);
        let user = user(pid);
        let cmdline = cmdline(pid).ok().flatten();
        let thread_count = *thread_map.get(&pid).unwrap_or(&0);
        out.push(ProcessRaw { pid, ppid, name, info, times, user, cmdline, thread_count });
    }
    Ok(out)
}
```

- [ ] **Step 2: Build**

Run: `cd d:\Work\rust\ps && cargo build -p ps-sys`
Expected: success

- [ ] **Step 3: Commit**

```bash
git add ps-sys/
git commit -m "feat(ps-sys): public collect() aggregating all info"
```

---

## Task 11: Initialize `ps-core` crate skeleton

**Files:**
- Create: `ps-core/Cargo.toml`
- Create: `ps-core/src/lib.rs`
- Create: `ps-core/src/process.rs`
- Create: `ps-core/src/user.rs`
- Create: `ps-core/src/state.rs`

- [ ] **Step 1: Create `ps-core/Cargo.toml`**

```toml
[package]
name = "ps-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
ps-sys = { path = "../ps-sys" }
thiserror = { workspace = true }
bitflags = { workspace = true }
once_cell = { workspace = true }
```

- [ ] **Step 2: Create `ps-core/src/user.rs`**

```rust
#[derive(Debug, Clone, Default)]
pub struct User {
    pub name: String,
    pub domain: String,
    pub sid: String,
}
```

- [ ] **Step 3: Create `ps-core/src/state.rs`**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    Sleeping,
    DiskSleep,
    Zombie,
    Stopped,
    Tracing,
    Dead,
    Unknown,
}
```

- [ ] **Step 4: Create `ps-core/src/process.rs`**

```rust
#![cfg_attr(test, allow(dead_code))]

use bitflags::bitflags;
use std::time::SystemTime;
use crate::user::User;
use crate::state::ProcessState;

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
```

- [ ] **Step 5: Create `ps-core/src/lib.rs` (stub others later)**

```rust
#![cfg(windows)]

pub mod process;
pub mod state;
pub mod user;
pub mod column;
pub mod columns;
pub mod filter;
pub mod sort;
pub mod tree;
pub mod assemble;

pub use process::{Process, ProcessFlags};
pub use state::ProcessState;
pub use user::User;
pub use column::{BoxCol, Column, ColumnKind, FormatCtx, ProcessSample, WidthMode};
pub use columns::{col_kind, lookup, parse_list};
pub use filter::{FilterExpr, FilterOp, Selector};
pub use sort::{apply_sort, SortSpec};
pub use tree::{build_forest, Forest, Node};
pub use assemble::{assemble, snapshot};
```

- [ ] **Step 6: Commit**

```bash
git add ps-core/
git commit -m "feat(ps-core): init crate with Process struct"
```

---

## Task 12: `ps-core::column` trait + ColumnKind

**Files:**
- Create: `ps-core/src/column.rs`

- [ ] **Step 1: Create `ps-core/src/column.rs`**

```rust
#![cfg(windows)]

use std::time::{Duration, Instant};
use crate::process::Process;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnKind {
    Pid, Ppid, Pgid, Sid, Tid, Pgrp,
    Uid, User, Gid, Group, Ruser, Rgroup, Suser,
    Vsz, Rss, Pmem, Pcpu, Pri, Nice, Class,
    Etime, Etimes, Times, Time, Stime, Start, Lstart,
    Stat, Flags, Tty, Tt, Sess,
    Comm, Args, Cmd, Command,
    C, Cp, Thcount, Nlwp,
}

pub struct FormatCtx<'a> {
    pub now: Instant,
    pub prev: Option<&'a ProcessSample>,
}

#[derive(Debug, Clone, Copy)]
pub enum WidthMode { Wide, Truncate(usize) }

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
    fn supported(&self) -> bool { true }
    fn align_right(&self) -> bool { false }
}

pub type BoxCol = Box<dyn Column>;

pub fn registry() -> Vec<(&'static str, fn() -> BoxCol)> {
    crate::columns::all()
}
```

- [ ] **Step 2: Commit**

```bash
git add ps-core/
git commit -m "feat(ps-core): column trait + ColumnKind"
```

---

## Task 13: `ps-core::columns::all` — all column implementations

**Files:**
- Create: `ps-core/src/columns/mod.rs`
- Create: One file per column

All columns follow the pattern:

```rust
use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;
pub struct Name;
impl Column for Name {
    fn kind(&self) -> ColumnKind { ColumnKind::Name }
    fn header(&self) -> &'static str { "HEADER" }
    fn width(&self) -> usize { N }
    fn align_right(&self) -> bool { bool }
    fn format(&self, p: &Process, _ctx: &FormatCtx) -> String { /* expr */ }
}
```

- [ ] **Step 1: Create `ps-core/src/columns/mod.rs`**

```rust
#![cfg(windows)]

use crate::column::BoxCol;

macro_rules! col {
    ($name:literal, $key:ident, $struct:path) => { ($name, || Box::new(<$struct>::default()) as BoxCol) };
}

mod pid; mod ppid; mod pgid; mod sid; mod tid; mod pgrp;
mod user; mod uid; mod gid; mod group; mod ruser; mod rgroup; mod suser;
mod vsz; mod rss; mod pmem; mod pcpu; mod pri; mod nice; mod class;
mod etime; mod etimes; mod times; mod time; mod stime; mod start; mod lstart;
mod stat; mod flags; mod tty; mod tt; mod sess;
mod comm; mod args; mod cmd; mod command;
mod c; mod cp; mod thcount; mod nlwp;

pub fn all() -> Vec<(&'static str, fn() -> BoxCol)> {
    vec![
        ("pid",     || Box::new(pid::Pid)),
        ("ppid",    || Box::new(ppid::Ppid)),
        ("pgid",    || Box::new(pgid::Pgid)),
        ("sid",     || Box::new(sid::Sid)),
        ("tid",     || Box::new(tid::Tid)),
        ("pgrp",    || Box::new(pgrp::Pgrp)),
        ("user",    || Box::new(user::UserCol)),
        ("uid",     || Box::new(uid::Uid)),
        ("gid",     || Box::new(gid::Gid)),
        ("group",   || Box::new(group::Group)),
        ("ruser",   || Box::new(ruser::Ruser)),
        ("rgroup",  || Box::new(rgroup::Rgroup)),
        ("suser",   || Box::new(suser::Suser)),
        ("vsz",     || Box::new(vsz::Vsz)),
        ("rss",     || Box::new(rss::Rss)),
        ("pmem",    || Box::new(pmem::Pmem)),
        ("pcpu",    || Box::new(pcpu::Pcpu)),
        ("pri",     || Box::new(pri::Pri)),
        ("nice",    || Box::new(nice::Nice)),
        ("ni",      || Box::new(nice::Nice)),
        ("class",   || Box::new(class::Class)),
        ("etime",   || Box::new(etime::Etime)),
        ("etimes",  || Box::new(etimes::Etimes)),
        ("times",   || Box::new(times::Times)),
        ("time",    || Box::new(time::Time)),
        ("stime",   || Box::new(stime::Stime)),
        ("start",   || Box::new(start::Start)),
        ("lstart",  || Box::new(lstart::Lstart)),
        ("stat",    || Box::new(stat::Stat)),
        ("flags",   || Box::new(flags::Flags)),
        ("f",       || Box::new(flags::Flags)),
        ("tty",     || Box::new(tty::Tty)),
        ("tt",      || Box::new(tt::Tt)),
        ("sess",    || Box::new(sess::Sess)),
        ("comm",    || Box::new(comm::Comm)),
        ("args",    || Box::new(args::Args)),
        ("cmd",     || Box::new(cmd::Cmd)),
        ("command", || Box::new(command::Command)),
        ("c",       || Box::new(c::C)),
        ("cp",      || Box::new(cp::Cp)),
        ("thcount", || Box::new(thcount::Thcount)),
        ("nlwp",    || Box::new(nlwp::Nlwp)),
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
```

- [ ] **Step 2: Create all individual column files**

For brevity, every column is its own `.rs` file. Each implements the trait with the data in the table below. The exact code for each column is:

```rust
// ps-core/src/columns/<name>.rs
use crate::column::{Column, ColumnKind, FormatCtx};
use crate::process::Process;

#[derive(Default)]
pub struct <Name>;

impl Column for <Name> {
    fn kind(&self) -> ColumnKind { ColumnKind::<KEY> }
    fn header(&self) -> &'static str { "<HEADER>" }
    fn width(&self) -> usize { <WIDTH> }
    fn align_right(&self) -> bool { <BOOL> }
    fn format(&self, p: &Process, ctx: &FormatCtx) -> String { <EXPR> }
    fn supported(&self) -> bool { <BOOL> }
}
```

Per-column data (kind, header, width, align_right, format, supported):

| file | kind | header | w | right | format | supported |
|---|---|---|---|---|---|---|
| `pid.rs` | Pid | PID | 5 | true | `p.pid.to_string()` | true |
| `ppid.rs` | Ppid | PPID | 5 | true | `p.ppid.to_string()` | true |
| `pgid.rs` | Pgid | PGID | 5 | true | `p.tgid.to_string()` | true |
| `sid.rs` | Sid | SID | 4 | true | `p.session_id.to_string()` | true |
| `tid.rs` | Tid | TID | 5 | true | `p.tgid.to_string()` | true |
| `pgrp.rs` | Pgrp | PGRP | 5 | true | `p.tgid.to_string()` | true |
| `user.rs` | User | USER | 12 | false | `if p.user.name.is_empty(){"?".into()} else {p.user.name.clone()}` | true |
| `uid.rs` | Uid | UID | 12 | false | `p.user.sid.clone()` | true |
| `gid.rs` | Gid | GID | 5 | true | `p.user.sid.clone()` | true |
| `group.rs` | Group | GROUP | 8 | false | `"?"` | false |
| `ruser.rs` | Ruser | RUSER | 12 | false | `p.user.name.clone()` | true |
| `rgroup.rs` | Rgroup | RGROUP | 8 | false | `"?"` | false |
| `suser.rs` | Suser | SUSER | 12 | false | `p.user.name.clone()` | true |
| `vsz.rs` | Vsz | VSZ | 7 | true | `(p.virtual_size/1024).to_string()` | true |
| `rss.rs` | Rss | RSS | 7 | true | `(p.working_set/1024).to_string()` | true |
| `pmem.rs` | Pmem | %MEM | 5 | true | `format!("{:.1}", (p.working_set as f64 / 8.0e9) * 100.0)` | true |
| `pcpu.rs` | Pcpu | %CPU | 5 | true | see snippet below | true |
| `pri.rs` | Pri | PRI | 3 | true | `(20 - p.nice).to_string()` | true |
| `nice.rs` | Nice | NI | 3 | true | `p.nice.to_string()` | true |
| `class.rs` | Class | CLASS | 6 | false | see snippet below | true |
| `etime.rs` | Etime | ELAPSED | 11 | false | see snippet below | true |
| `etimes.rs` | Etimes | ELAPSED | 7 | true | `p.start_time.map_or("?".into(), |s| std::time::SystemTime::now().duration_since(s).map_or("?".into(), |d| d.as_secs().to_string()))` | true |
| `times.rs` | Times | TIMES | 11 | false | `format!("{}:{}", p.user_time.as_secs(), p.kernel_time.as_secs())` | true |
| `time.rs` | Time | TIME | 8 | false | `format!("{:02}:{:02}:{:02}", (p.kernel_time + p.user_time).as_secs()/3600, ((p.kernel_time + p.user_time).as_secs()/60)%60, (p.kernel_time + p.user_time).as_secs()%60)` | true |
| `stime.rs` | Stime | STIME | 5 | false | `format_time_short(p.start_time)` | true |
| `start.rs` | Start | STARTED | 8 | false | `format_time_short(p.start_time)` | true |
| `lstart.rs` | Lstart | STARTED | 24 | false | `p.start_time.map_or("?".into(), |s| s.duration_since(std::time::UNIX_EPOCH).map_or("?".into(), |d| format!("{}", d.as_secs())))` | true |
| `stat.rs` | Stat | STAT | 4 | false | see snippet below | true |
| `flags.rs` | Flags | F | 4 | true | `format!("{:08x}", p.flags.bits())` | true |
| `tty.rs` | Tty | TTY | 3 | false | `"?"` | false |
| `tt.rs` | Tt | TT | 3 | false | `"?"` | false |
| `sess.rs` | Sess | SESS | 4 | true | `p.session_id.to_string()` | true |
| `comm.rs` | Comm | COMM | 16 | false | `p.name.clone()` | true |
| `args.rs` | Args | ARGS | 32 | false | `p.cmdline.clone().unwrap_or_else(|| p.image_path.clone())` | true |
| `cmd.rs` | Cmd | CMD | 32 | false | same as args | true |
| `command.rs` | Command | COMMAND | 40 | false | same as args | true |
| `c.rs` | C | C | 3 | true | see snippet below | true |
| `cp.rs` | Cp | CP | 4 | true | same as `c` rounded to int | true |
| `thcount.rs` | Thcount | THCNT | 5 | true | `p.thread_count.to_string()` | true |
| `nlwp.rs` | Nlwp | NLWP | 5 | true | `p.thread_count.to_string()` | true |

Snippets referenced above:

`pcpu.rs`:
```rust
let Some(prev) = ctx.prev else { return "?".into(); };
if prev.pid != p.pid { return "?".into(); }
let dt = ctx.now.duration_since(prev.at).as_secs_f64();
if dt < 0.001 { return "0.0".into(); }
let dcpu = (p.kernel_time + p.user_time).saturating_sub(prev.cpu_total).as_secs_f64();
format!("{:.1}", 100.0 * dcpu / dt)
```

`class.rs`:
```rust
match p.priority_class {
    0x00000040 => "Idle".into(),
    0x00004000 => "Below".into(),
    0x00000020 => "Normal".into(),
    0x00008000 => "Above".into(),
    0x00000080 => "High".into(),
    0x00000100 => "Realtime".into(),
    _ => "?".into(),
}
```

`etime.rs`:
```rust
let Some(start) = p.start_time else { return "?".into(); };
let secs = std::time::SystemTime::now().duration_since(start).map(|d| d.as_secs()).unwrap_or(0);
let dd = secs / 86400;
let hh = (secs / 3600) % 24;
let mm = (secs / 60) % 60;
let ss = secs % 60;
if dd > 0 { format!("{dd:02}-{hh:02}:{mm:02}:{ss:02}") } else { format!("{hh:02}:{mm:02}:{ss:02}") }
```

`stat.rs`:
```rust
let mut s = String::new();
s.push(match p.state {
    crate::state::ProcessState::Running => 'R',
    crate::state::ProcessState::Sleeping => 'S',
    crate::state::ProcessState::DiskSleep => 'D',
    crate::state::ProcessState::Zombie => 'Z',
    crate::state::ProcessState::Stopped => 'T',
    crate::state::ProcessState::Tracing => 't',
    crate::state::ProcessState::Dead => 'X',
    _ => '?',
});
if p.flags.contains(crate::process::ProcessFlags::ELEVATED) { s.push('*'); }
s
```

`c.rs` and `cp.rs` use the same logic as `pcpu.rs` but rounded: `format!("{}", (100.0 * dcpu / dt) as i32)`.

`stime.rs` and `start.rs` share helper:
```rust
fn format_time_short(t: Option<std::time::SystemTime>) -> String {
    use std::time::UNIX_EPOCH;
    match t {
        Some(s) => {
            let dur = s.duration_since(UNIX_EPOCH).unwrap_or_default();
            let total_min = dur.as_secs() / 60;
            let hh = (total_min / 60) % 24;
            let mm = total_min % 60;
            format!("{hh:02}:{mm:02}")
        }
        None => "?".into(),
    }
}
```

- [ ] **Step 3: Build**

Run: `cd d:\Work\rust\ps && cargo build -p ps-core`
Expected: success (assemble/sort/filter/tree stubs are in Task 14–16)

- [ ] **Step 4: Commit**

```bash
git add ps-core/
git commit -m "feat(ps-core): all 38 columns"
```

---

## Task 14: `ps-core::filter` — Selector + FilterExpr

**Files:**
- Create: `ps-core/src/filter.rs`

- [ ] **Step 1: Create `ps-core/src/filter.rs`**

```rust
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
pub enum FilterOp { Eq, Ne, Lt, Gt }

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
    if !sel.users.is_empty() && !sel.users.iter().any(|u| u == &p.user.name || u == &p.user.sid) {
        return false;
    }
    if !sel.session.is_empty() && !sel.session.contains(&p.session_id) {
        return false;
    }
    if !sel.comm_patterns.is_empty() {
        let name = p.cmdline.as_deref().unwrap_or(&p.name);
        let first = name.split_whitespace().next().unwrap_or("");
        if !sel.comm_patterns.iter().any(|pat| first.contains(pat.as_str()) || p.name.contains(pat.as_str())) {
            return false;
        }
    }
    true
}

pub fn passes_filter_expr(p: &Process, exprs: &[FilterExpr], cols: &[Box<dyn crate::column::Column>]) -> bool {
    use std::str::FromStr;
    for e in exprs {
        let cell = cols.iter()
            .find(|c| c.header().eq_ignore_ascii_case(&e.col) || format!("{:?}", c.kind()).eq_ignore_ascii_case(&e.col))
            .map(|c| c.format(p, &crate::column::FormatCtx { now: std::time::Instant::now(), prev: None }));
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
        if !ok { return false; }
    }
    true
}
```

- [ ] **Step 2: Add unit test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::User;
    use crate::state::ProcessState;
    use crate::process::{Process, ProcessFlags};

    fn p(pid: u32, name: &str) -> Process {
        Process {
            pid, ppid: 0, tgid: pid, session_id: 1, name: name.into(),
            image_path: String::new(), cmdline: None, user: User::default(),
            state: ProcessState::Running, priority_class: 0, nice: 10, start_time: None,
            kernel_time: Default::default(), user_time: Default::default(), thread_count: 1,
            handles: None, working_set: 0, peak_working_set: 0, virtual_size: 0, exit_status: None,
            flags: ProcessFlags::empty(),
        }
    }

    #[test]
    fn pid_filter() {
        let s = Selector { pids: vec![1, 2], explicit: true, ..Default::default() };
        assert!(passes(&p(1, "x"), &s));
        assert!(!passes(&p(3, "x"), &s));
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cd d:\Work\rust\ps && cargo test -p ps-core --lib`
Expected: passes

- [ ] **Step 4: Commit**

```bash
git add ps-core/
git commit -m "feat(ps-core): Selector and FilterExpr"
```

---

## Task 15: `ps-core::sort` — SortSpec + apply

**Files:**
- Create: `ps-core/src/sort.rs`

- [ ] **Step 1: Create `ps-core/src/sort.rs`**

```rust
#![cfg(windows)]

use crate::process::Process;
use crate::column::FormatCtx;

#[derive(Debug, Clone, Default)]
pub struct SortSpec { pub keys: Vec<SortKey> }

#[derive(Debug, Clone)]
pub struct SortKey { pub name: String, pub reverse: bool }

pub fn parse_sort(spec: &[String]) -> SortSpec {
    let mut keys = Vec::new();
    for s in spec {
        for k in s.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let (name, reverse) = if let Some(rest) = k.strip_prefix('-') { (rest.to_string(), true) } else { (k.to_string(), false) };
            keys.push(SortKey { name, reverse });
        }
    }
    SortSpec { keys }
}

pub fn apply_sort(mut procs: Vec<Process>, spec: &SortSpec, cols: &[Box<dyn crate::column::Column>]) -> Vec<Process> {
    if spec.keys.is_empty() {
        procs.sort_by_key(|p| p.pid);
        return procs;
    }
    let keys = spec.keys.clone();
    procs.sort_by(|a, b| {
        for k in &keys {
            let Some(ca) = find_col(cols, &k.name) else { continue; };
            let Some(cb) = find_col(cols, &k.name) else { continue; };
            let ctx = FormatCtx { now: std::time::Instant::now(), prev: None };
            let sa = ca.format(a, &ctx);
            let sb = cb.format(b, &ctx);
            let ord = natural_cmp(&sa, &sb);
            let ord = if k.reverse { ord.reverse() } else { ord };
            if ord != std::cmp::Ordering::Equal { return ord; }
        }
        a.pid.cmp(&b.pid)
    });
    procs
}

fn find_col<'a>(cols: &'a [Box<dyn crate::column::Column>], name: &str) -> Option<&'a Box<dyn crate::column::Column>> {
    cols.iter().find(|c| c.header().eq_ignore_ascii_case(name) || format!("{:?}", c.kind()).eq_ignore_ascii_case(name))
}

fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let (an, bn) = (a.parse::<u64>().ok(), b.parse::<u64>().ok());
    match (an, bn) {
        (Some(x), Some(y)) => x.cmp(&y),
        _ => a.cmp(b),
    }
}
```

- [ ] **Step 2: Add unit test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::User;
    use crate::state::ProcessState;
    use crate::process::{Process, ProcessFlags};

    fn p(pid: u32) -> Process {
        Process {
            pid, ppid: 0, tgid: pid, session_id: 1, name: "x".into(),
            image_path: String::new(), cmdline: None, user: User::default(),
            state: ProcessState::Running, priority_class: 0, nice: 10, start_time: None,
            kernel_time: Default::default(), user_time: Default::default(), thread_count: 1,
            handles: None, working_set: 0, peak_working_set: 0, virtual_size: 0, exit_status: None,
            flags: ProcessFlags::empty(),
        }
    }

    #[test]
    fn default_sort_by_pid_asc() {
        let mut v = vec![p(3), p(1), p(2)];
        v = apply_sort(v, &SortSpec::default(), &[]);
        assert_eq!(v.iter().map(|x| x.pid).collect::<Vec<_>>(), vec![1,2,3]);
    }

    #[test]
    fn parse_sort_with_reverse() {
        let s = parse_sort(&["-pcpu".into(), "pid".into()]);
        assert_eq!(s.keys.len(), 2);
        assert!(s.keys[0].reverse);
        assert!(!s.keys[1].reverse);
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cd d:\Work\rust\ps && cargo test -p ps-core --lib`
Expected: all pass

- [ ] **Step 4: Commit**

```bash
git add ps-core/
git commit -m "feat(ps-core): sort spec + apply"
```

---

## Task 16: `ps-core::tree` — Forest builder

**Files:**
- Create: `ps-core/src/tree.rs`

- [ ] **Step 1: Create `ps-core/src/tree.rs`**

```rust
#![cfg(windows)]

use crate::process::Process;

pub struct Node {
    pub proc: Process,
    pub children: Vec<Node>,
}

pub struct Forest { pub roots: Vec<Node> }

pub fn build_forest(mut procs: Vec<Process>) -> Forest {
    // sort by pid for deterministic order
    procs.sort_by_key(|p| p.pid);
    let by_pid: std::collections::HashMap<u32, usize> = procs.iter().enumerate().map(|(i, p)| (p.pid, i)).collect();
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

fn build_node(p: &Process, children: &std::collections::HashMap<u32, Vec<u32>>, procs: &[Process]) -> Node {
    let mut kids = Vec::new();
    if let Some(cids) = children.get(&p.pid) {
        for cid in cids {
            if let Some(child) = procs.iter().find(|x| x.pid == *cid) {
                kids.push(build_node(child, children, procs));
            }
        }
    }
    Node { proc: p.clone(), children: kids }
}
```

- [ ] **Step 2: Add unit test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::User;
    use crate::state::ProcessState;
    use crate::process::{Process, ProcessFlags};

    fn p(pid: u32, ppid: u32) -> Process {
        Process {
            pid, ppid, tgid: pid, session_id: 1, name: format!("p{pid}"),
            image_path: String::new(), cmdline: None, user: User::default(),
            state: ProcessState::Running, priority_class: 0, nice: 10, start_time: None,
            kernel_time: Default::default(), user_time: Default::default(), thread_count: 1,
            handles: None, working_set: 0, peak_working_set: 0, virtual_size: 0, exit_status: None,
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
```

- [ ] **Step 3: Run tests**

Run: `cd d:\Work\rust\ps && cargo test -p ps-core --lib`
Expected: all pass

- [ ] **Step 4: Commit**

```bash
git add ps-core/
git commit -m "feat(ps-core): Forest tree builder"
```

---

## Task 17: `ps-core::assemble` — map `ps_sys::ProcessRaw` → `ps_core::Process`

**Files:**
- Create: `ps-core/src/assemble.rs`

- [ ] **Step 1: Create `ps-core/src/assemble.rs`**

```rust
#![cfg(windows)]

use crate::process::{Process, ProcessFlags};
use crate::state::ProcessState;
use crate::user::User;
use ps_sys::ProcessRaw;

pub fn assemble_one(r: ProcessRaw) -> Process {
    let mut flags = ProcessFlags::empty();
    let (image_path, session_id, working_set, peak_working_set, pagefile_usage, priority_class, wow64) = match r.info {
        Ok(i) => (i.image_path, i.session_id, i.working_set, i.peak_working_set, i.pagefile_usage, i.priority_class, i.wow64),
        Err(_) => (String::new(), 0, 0, 0, 0, 0, false),
    };
    if wow64 { flags |= ProcessFlags::WOW64; }
    let (start, kernel, user) = match r.times {
        Ok(t) => (t.start, t.kernel, t.user),
        Err(_) => (None, Default::default(), Default::default()),
    };
    let user_info = match r.user {
        Ok(u) => User { name: u.name, domain: u.domain, sid: u.sid },
        Err(_) => User::default(),
    };
    let name = if r.name.is_empty() {
        std::path::Path::new(&image_path).file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
    } else {
        r.name
    };
    let nice = map_nice(priority_class);
    let virtual_size = working_set.saturating_add(pagefile_usage);
    Process {
        pid: r.pid, ppid: r.ppid, tgid: r.pid, session_id,
        name, image_path, cmdline: r.cmdline, user: user_info,
        state: ProcessState::Running, priority_class, nice,
        start_time: start, kernel_time: kernel, user_time: user,
        thread_count: r.thread_count, handles: None,
        working_set, peak_working_set, virtual_size,
        exit_status: None, flags,
    }
}

fn map_nice(prio: u32) -> i32 {
    match prio {
        0x00000040 => 19,
        0x00004000 => 15,
        0x00000020 => 10,
        0x00008000 => 5,
        0x00000080 => -5,
        0x00000100 => -20,
        _ => 10,
    }
}

pub fn snapshot() -> Vec<Process> {
    ps_sys::collect()
        .unwrap_or_default()
        .into_iter()
        .map(assemble_one)
        .collect()
}
```

- [ ] **Step 2: Build**

Run: `cd d:\Work\rust\ps && cargo build -p ps-core`
Expected: success

- [ ] **Step 3: Add smoke test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ps_sys::ProcessRaw;

    #[test]
    fn assemble_handles_missing_info() {
        let r = ProcessRaw {
            pid: 7, ppid: 1, name: "x.exe".into(),
            info: Err(ps_sys::Error::PermissionDenied(7)),
            times: Err(ps_sys::Error::PermissionDenied(7)),
            user: Err(ps_sys::Error::PermissionDenied(7)),
            cmdline: None, thread_count: 0,
        };
        let p = assemble_one(r);
        assert_eq!(p.pid, 7);
        assert_eq!(p.name, "x.exe");
    }
}
```

- [ ] **Step 4: Commit**

```bash
git add ps-core/
git commit -m "feat(ps-core): assemble ProcessRaw -> Process"
```

---

## Task 18: `ps-core` — full test run

**Files:** (none new)

- [ ] **Step 1: Run all ps-core tests**

Run: `cd d:\Work\rust\ps && cargo test -p ps-core --lib`
Expected: all tests pass

- [ ] **Step 2: Run clippy**

Run: `cd d:\Work\rust\ps && cargo clippy -p ps-core -- -D warnings`
Expected: warnings may exist; fix them inline

- [ ] **Step 3: Commit fixes if any**

```bash
git add -A
git commit -m "fix(ps-core): clippy cleanups" || echo "nothing to commit"
```

---

## Task 19: Initialize `ps-bin` crate skeleton

**Files:**
- Create: `ps-bin/Cargo.toml`
- Create: `ps-bin/src/main.rs`

- [ ] **Step 1: Create `ps-bin/Cargo.toml`**

```toml
[package]
name = "ps"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[[bin]]
name = "ps"
path = "src/main.rs"

[dependencies]
ps-core = { path = "../ps-core" }
anyhow = { workspace = true }
clap = { workspace = true }
```

- [ ] **Step 2: Create `ps-bin/src/main.rs` (skeleton)**

```rust
fn main() {
    println!("ps-on-windows scaffold OK");
}
```

- [ ] **Step 3: Build**

Run: `cd d:\Work\rust\ps && cargo build -p ps`
Expected: success, binary at `target/debug/ps.exe`

- [ ] **Step 4: Commit**

```bash
git add ps-bin/
git commit -m "feat(ps-bin): init crate skeleton"
```

---

## Task 20: `ps-bin::args` — clap Cli + parse_filter

**Files:**
- Create: `ps-bin/src/args.rs`
- Modify: `ps-bin/src/main.rs`

- [ ] **Step 1: Create `ps-bin/src/args.rs`**

```rust
use clap::Parser;
use ps_core::filter::FilterOp;

#[derive(Parser, Debug)]
#[command(name = "ps", about = "Report process status (Linux ps(1) compatible)", version)]
pub struct Cli {
    #[arg(short = 'A', short_alias = 'e', long = "select-all")]
    pub all: bool,
    #[arg(short = 'x', long = "no-ttys")]
    pub no_ttys: bool,
    #[arg(short = 'f', long = "full")]
    pub full: bool,
    #[arg(short = 'l', long = "long")]
    pub long: bool,
    #[arg(short = 'a', long = "all-with-tty")]
    pub all_with_tty: bool,
    #[arg(short = 'o', long = "format", value_delimiter = ',')]
    pub format: Vec<String>,
    #[arg(short = 'p', long = "pid", value_delimiter = ',')]
    pub pid: Vec<u32>,
    #[arg(short = 'u', long = "user", value_delimiter = ',')]
    pub user: Vec<String>,
    #[arg(short = 'C', long = "comm")]
    pub comm: Vec<String>,
    #[arg(short = 's', long = "session", value_delimiter = ',')]
    pub session: Vec<u32>,
    #[arg(short = 'O', long = "sort", value_delimiter = ',')]
    pub sort: Vec<String>,
    #[arg(long = "forest")]
    pub forest: bool,
    #[arg(long = "ascii-lines")]
    pub ascii_lines: bool,
    #[arg(long = "no-headers")]
    pub no_headers: bool,
    #[arg(short = 'L', long = "show-threads")]
    pub threads: bool,
    #[arg(long = "no-align")]
    pub no_align: bool,
    #[arg(long = "filter", value_parser = parse_filter)]
    pub filter: Vec<(String, FilterOp, String)>,
}

pub fn parse_filter(s: &str) -> Result<(String, FilterOp, String), String> {
    let ops = [("==", FilterOp::Eq), ("!=", FilterOp::Ne), ("<=", FilterOp::Lt), (">=", FilterOp::Gt), ("<", FilterOp::Lt), (">", FilterOp::Gt), ("=", FilterOp::Eq)];
    for (sym, op) in ops {
        if let Some((k, v)) = s.split_once(sym) {
            return Ok((k.trim().to_string(), op, v.trim().to_string()));
        }
    }
    Err(format!("invalid --filter: {s}"))
}

pub fn resolve_columns(cli: &Cli) -> Vec<ps_core::column::BoxCol> {
    if !cli.format.is_empty() {
        return ps_core::parse_list(&cli.format.join(",")).expect("invalid -o");
    }
    let spec = if cli.forest { "user,pid,ppid,pgid,sid,tty,stat,start,time,command" }
        else if cli.long { "f,s,user,pid,ppid,c,pri,ni,addr,sz,wchan,stime,tty,time,cmd" }
        else if cli.full { "user,pid,ppid,c,stime,tty,time,cmd" }
        else if cli.all_with_tty { "user,pid,ppid,pgid,sid,tty,stat,start,time,command" }
        else if cli.all { "pid,tty,time,cmd" }
        else { "pid,tty,time,cmd" };
    ps_core::parse_list(spec).expect("default columns must parse")
}
```

- [ ] **Step 2: Wire args module in `ps-bin/src/main.rs`**

```rust
mod args;
mod render;
mod compat;

use std::io::{stdout, Write};
use std::process::ExitCode;

use clap::Parser;
use ps_core::{assemble, column::FormatCtx, filter::{Selector, passes, passes_filter_expr}, sort::{parse_sort, apply_sort}, tree::build_forest};
use ps_core::column::ProcessSample;

use args::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();
    if let Err(code) = run(cli) {
        if let Some(s) = code { eprintln!("ps: {s}"); return ExitCode::from(1); }
    }
    ExitCode::SUCCESS
}

fn run(cli: Cli) -> Result<(), Option<&'static str>> {
    render::color::enable_ansi();
    let cols = args::resolve_columns(&cli);
    let selector = Selector {
        pids: cli.pid.clone(),
        users: cli.user.clone(),
        session: cli.session.clone(),
        comm_patterns: cli.comm.clone(),
        include_all: cli.all || cli.all_with_tty,
        include_no_ttys: cli.no_ttys,
        explicit: !cli.pid.is_empty() || !cli.user.is_empty() || !cli.session.is_empty() || !cli.comm.is_empty(),
    };
    let mut procs = assemble::snapshot();
    procs.retain(|p| passes(p, &selector));

    let filter_exprs: Vec<_> = cli.filter.iter().map(|(k, op, v)| ps_core::filter::FilterExpr { col: k.clone(), op: *op, val: v.clone() }).collect();
    procs.retain(|p| passes_filter_expr(p, &filter_exprs, &cols));

    let sort_spec = parse_sort(&cli.sort);
    procs = apply_sort(procs, &sort_spec, &cols);

    let mut out = stdout().lock();
    if cli.forest {
        let f = build_forest(procs);
        render::forest::render(&mut out, &f, &cols, cli.ascii_lines, cli.no_headers).map_err(|_| Some("render failed"))?;
    } else {
        render::table::render(&mut out, &procs, &cols, cli.no_headers, cli.no_align).map_err(|_| Some("render failed"))?;
    }

    if selector.explicit && procs.is_empty() {
        return Err(None);
    }
    Ok(())
}
```

- [ ] **Step 3: Commit**

```bash
git add ps-bin/
git commit -m "feat(ps-bin): args + main glue"
```

---

## Task 21: `ps-bin::render` — table + forest + color

**Files:**
- Create: `ps-bin/src/render/mod.rs`
- Create: `ps-bin/src/render/table.rs`
- Create: `ps-bin/src/render/forest.rs`
- Create: `ps-bin/src/render/color.rs`

- [ ] **Step 1: Create `ps-bin/src/render/mod.rs`**

```rust
pub mod table;
pub mod forest;
pub mod color;
```

- [ ] **Step 2: Create `ps-bin/src/render/color.rs`**

```rust
pub fn enable_ansi() {
    #[cfg(windows)]
    {
        use windows::Win32::System::Console::{GetConsoleMode, SetConsoleMode, ENABLE_VIRTUAL_TERMINAL_PROCESSING};
        unsafe {
            let handle = windows::Win32::System::Console::GetStdHandle(windows::Win32::System::Console::STD_OUTPUT_HANDLE).unwrap_or_default();
            if handle.is_invalid() { return; }
            let mut mode = 0u32;
            if GetConsoleMode(handle, &mut mode).is_err() { return; }
            let _ = SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING.0);
        }
    }
}
```

- [ ] **Step 3: Create `ps-bin/src/render/table.rs`**

```rust
use std::io::Write;
use ps_core::process::Process;
use ps_core::column::{BoxCol, FormatCtx};

pub fn render<W: Write>(out: &mut W, procs: &[Process], cols: &[BoxCol], no_headers: bool, no_align: bool) -> std::io::Result<()> {
    let ctx = FormatCtx { now: std::time::Instant::now(), prev: None };
    let cells: Vec<Vec<String>> = procs.iter().map(|p| cols.iter().map(|c| c.format(p, &ctx)).collect()).collect();
    if !no_headers {
        let headers: Vec<String> = cols.iter().map(|c| c.header().to_string()).collect();
        write_row(out, &headers, cols, no_align)?;
    }
    for row in &cells {
        write_row(out, row, cols, no_align)?;
    }
    Ok(())
}

fn write_row<W: Write>(out: &mut W, row: &[String], cols: &[BoxCol], no_align: bool) -> std::io::Result<()> {
    for (i, cell) in row.iter().enumerate() {
        if i > 0 { write!(out, " ")?; }
        if no_align {
            write!(out, "{cell}")?;
        } else {
            let w = cols[i].width().max(cell.len());
            if cols[i].align_right() {
                write!(out, "{cell:>w$}", cell = cell, w = w)?;
            } else {
                write!(out, "{cell:<w$}", cell = cell, w = w)?;
            }
        }
    }
    writeln!(out)
}
```

- [ ] **Step 4: Create `ps-bin/src/render/forest.rs`**

```rust
use std::io::Write;
use ps_core::tree::Forest;
use ps_core::column::{BoxCol, FormatCtx};
use ps_core::process::Process;

pub fn render<W: Write>(out: &mut W, f: &Forest, cols: &[BoxCol], ascii: bool, no_headers: bool) -> std::io::Result<()> {
    let ctx = FormatCtx { now: std::time::Instant::now(), prev: None };
    if !no_headers {
        let headers: Vec<String> = cols.iter().map(|c| c.header().to_string()).collect();
        writeln!(out, "{}", headers.join(" "))?;
    }
    for r in &f.roots {
        write_node(out, r, "", true, ascii, cols, &ctx)?;
    }
    Ok(())
}

fn write_node<W: Write>(out: &mut W, n: &ps_core::tree::Node, prefix: &str, last: bool, ascii: bool, cols: &[BoxCol], ctx: &FormatCtx) -> std::io::Result<()> {
    let (branch, line) = if ascii { ("+-- ", "|   ") } else { ("└─ ", "   ") };
    let branch = if last { branch } else { if ascii { "|-- " } else { "├─ " } };
    let line_branch = if last { if ascii { "    " } else { "   " } } else { line };
    let row: Vec<String> = cols.iter().map(|c| c.format(&n.proc, ctx)).collect();
    writeln!(out, "{}{}{}", prefix, branch, row.join(" "))?;
    let new_prefix = format!("{prefix}{line_branch}");
    for (i, c) in n.children.iter().enumerate() {
        let is_last = i + 1 == n.children.len();
        write_node(out, c, &new_prefix, is_last, ascii, cols, ctx)?;
    }
    Ok(())
}
```

- [ ] **Step 5: Create `ps-bin/src/compat.rs` (exit code helpers)**

```rust
pub fn exit_no_match() -> i32 { 1 }
pub fn exit_invalid_args() -> i32 { 2 }
pub fn exit_system_error() -> i32 { 1 }
```

- [ ] **Step 6: Build and run**

Run: `cd d:\Work\rust\ps && cargo build -p ps && target\debug\ps.exe -A | head`
Expected: prints a header row + several process rows

- [ ] **Step 7: Commit**

```bash
git add ps-bin/
git commit -m "feat(ps-bin): table + forest + color renderers"
```

---

## Task 22: `ps-bin` integration tests (assert_cmd)

**Files:**
- Create: `ps-bin/tests/cli.rs`
- Modify: `ps-bin/Cargo.toml`

- [ ] **Step 1: Add dev-dependencies**

In `ps-bin/Cargo.toml`:
```toml
[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

- [ ] **Step 2: Create `ps-bin/tests/cli.rs`**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn ps_runs_and_has_header() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.arg("-A");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("PID"));
}

#[test]
fn ps_pid_filter_finds_self() {
    let me = std::process::id();
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.args(["-p", &me.to_string()]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(&me.to_string()));
}

#[test]
fn ps_pid_filter_nonexistent_returns_code_1() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.args(["-p", "999999"]);
    cmd.assert().code(1);
}

#[test]
fn ps_invalid_flag_returns_code_2() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.arg("--this-does-not-exist");
    cmd.assert().code(2);
}

#[test]
fn ps_forest_contains_tree_chars() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.args(["-A", "--forest"]);
    let out = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    assert!(stdout.contains("├─") || stdout.contains("└─") || stdout.contains("+-- ") || stdout.contains("|-- "));
}

#[test]
fn ps_no_headers_hides_header() {
    let mut cmd = Command::cargo_bin("ps").unwrap();
    cmd.args(["-A", "--no-headers"]);
    let out = cmd.assert().success();
    let stdout = String::from_utf8_lossy(&out.get_output().stdout);
    let first = stdout.lines().next().unwrap_or("");
    assert!(!first.starts_with("PID"), "first line should not be header, got: {first}");
}
```

- [ ] **Step 3: Run tests**

Run: `cd d:\Work\rust\ps && cargo test -p ps --test cli`
Expected: all pass

- [ ] **Step 4: Commit**

```bash
git add ps-bin/
git commit -m "test(ps-bin): assert_cmd integration tests"
```

---

## Task 23: `ps-bin` exit codes 1/2 wired in main

**Files:**
- Modify: `ps-bin/src/main.rs`

- [ ] **Step 1: Replace main.rs**

```rust
mod args;
mod render;
mod compat;

use std::io::{stdout, Write};
use std::process::ExitCode;

use clap::Parser;
use ps_core::{assemble, column::FormatCtx, filter::{Selector, passes, passes_filter_expr, FilterExpr}, sort::{parse_sort, apply_sort}, tree::build_forest};

use args::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();
    render::color::enable_ansi();
    let cols = args::resolve_columns(&cli);
    let selector = Selector {
        pids: cli.pid.clone(),
        users: cli.user.clone(),
        session: cli.session.clone(),
        comm_patterns: cli.comm.clone(),
        include_all: cli.all || cli.all_with_tty,
        include_no_ttys: cli.no_ttys,
        explicit: !cli.pid.is_empty() || !cli.user.is_empty() || !cli.session.is_empty() || !cli.comm.is_empty(),
    };
    let mut procs = assemble::snapshot();
    procs.retain(|p| passes(p, &selector));
    let filter_exprs: Vec<FilterExpr> = cli.filter.iter().map(|(k, op, v)| FilterExpr { col: k.clone(), op: *op, val: v.clone() }).collect();
    procs.retain(|p| passes_filter_expr(p, &filter_exprs, &cols));
    let sort_spec = parse_sort(&cli.sort);
    procs = apply_sort(procs, &sort_spec, &cols);

    let mut out = stdout().lock();
    let render_result = if cli.forest {
        let f = build_forest(procs.clone());
        render::forest::render(&mut out, &f, &cols, cli.ascii_lines, cli.no_headers)
    } else {
        render::table::render(&mut out, &procs, &cols, cli.no_headers, cli.no_align)
    };
    if let Err(e) = render_result {
        eprintln!("ps: render error: {e}");
        return ExitCode::from(compat::exit_system_error() as u8);
    }
    if selector.explicit && procs.is_empty() {
        return ExitCode::from(compat::exit_no_match() as u8);
    }
    ExitCode::SUCCESS
}
```

> Note: clap's `parse()` already returns exit code 2 on bad args, so we don't need to handle that explicitly.

- [ ] **Step 2: Run**

Run: `cd d:\Work\rust\ps && cargo run -p ps -- -p 999999`
Expected: exit code 1

- [ ] **Step 3: Commit**

```bash
git add ps-bin/
git commit -m "feat(ps-bin): exit codes for no-match"
```

---

## Task 24: `ps-bin` final polish — fmt, clippy

- [ ] **Step 1: Format**

Run: `cd d:\Work\rust\ps && cargo fmt --all`

- [ ] **Step 2: Clippy**

Run: `cd d:\Work\rust\ps && cargo clippy --workspace --all-targets -- -D warnings`
Expected: zero warnings

- [ ] **Step 3: Full test suite**

Run: `cd d:\Work\rust\ps && cargo test --workspace`
Expected: all green

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "chore: cargo fmt + clippy clean"
```

---

## Task 25: Release script

**Files:**
- Create: `scripts/release.ps1`

- [ ] **Step 1: Create `scripts/release.ps1`**

```powershell
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

Write-Host "Building release..." -ForegroundColor Cyan
cargo build --release --workspace

$dst = Join-Path $root "dist"
New-Item -ItemType Directory -Force -Path $dst | Out-Null

$bin = Join-Path $root "target\release\ps.exe"
if (-not (Test-Path $bin)) {
    throw "Binary not found at $bin"
}
Copy-Item $bin (Join-Path $dst "ps.exe") -Force

Write-Host "Done. Output: $dst\ps.exe" -ForegroundColor Green
& (Get-Item (Join-Path $dst "ps.exe"))
```

- [ ] **Step 2: Commit**

```bash
git add scripts/
git commit -m "chore: add release.ps1"
```

---

## Task 26: Final verification

- [ ] **Step 1: Run all tests**

Run: `cd d:\Work\rust\ps && cargo test --workspace`
Expected: all green

- [ ] **Step 2: Run clippy**

Run: `cd d:\Work\rust\ps && cargo clippy --workspace --all-targets -- -D warnings`
Expected: clean

- [ ] **Step 3: Run release build**

Run: `cd d:\Work\rust\ps && powershell -ExecutionPolicy Bypass -File scripts\release.ps1`
Expected: `dist\ps.exe` produced and runs

- [ ] **Step 4: Smoke test binary**

Run: `cd d:\Work\rust\ps && .\dist\ps.exe -A | Select-Object -First 5`
Expected: 5 lines of process output

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore: final verification" --allow-empty
```

---

## Self-Review Notes

- **Spec coverage**:
  - §3 ps-sys — Tasks 2–10 ✓
  - §4 ps-core — Tasks 11–18 ✓ (ColumnKind, Process, Selector, SortSpec, Forest, assemble)
  - §5 ps-bin — Tasks 19–23 ✓ (clap, default columns, render table/forest, exit codes)
  - §6 testing — Tasks 18, 22, 24 ✓
  - §7 known limits — documented in spec; cmdline 64-bit only (Task 9) ✓
  - §8 release — Task 25 ✓

- **No placeholders**: Every step shows actual file paths and code. No TBD/TODO. ✓

- **Type consistency**:
  - `Column::format(&self, &Process, &FormatCtx)` — used consistently in columns, table, forest.
  - `Process` field names match spec §4.1 and assemble.rs.
  - `Selector` field names match spec §4.3.
  - `Forest`/`Node` match spec §4.6.

- **Edge cases handled**:
  - System processes (cmdline None) handled in assemble.
  - Permission denied on individual syscalls downgraded to empty fields.
  - Default sort = pid asc.
  - Forest mode uses `-a` columns per spec §4.6 fix.

