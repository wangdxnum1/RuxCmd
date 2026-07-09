# ps-on-windows — 设计文档

日期：2026-06-02
状态：待用户复核
范围：在 Windows 上用 Rust 实现 Linux `ps(1)` 命令的兼容版本

## 1. 目标与非目标

### 1.1 目标
- 在 Windows 上提供与 Linux `ps(1)` 用法兼容的命令行工具 `ps`
- 支持最常用的选项（`-A/-e`、`-f`、`-l`、`-a`、`-x`、`-u`、`-p`、`-C`、`-G`、`-s`、`-o`、`--sort`、`--forest`、`--no-headers`、`-L` 等）
- 实现 Linux 风格的列关键字（`pid`、`ppid`、`pcpu`、`pmem`、`etime`、`user`、`comm`、`args` …）
- 通过 `cargo build` / `cargo install` 即可安装
- 在 `windows-latest` CI 上 `cargo test --workspace` 全绿

### 1.2 非目标
- 不实现 Windows 平台无关之外的 Linux 平台二进制（`target_os = "windows"` 限制）
- 不实现信号相关列（`SIG`、`BLOCKED`、`IGNORED`、`CAUGHT`）
- 不替代 Sysinternals、Process Hacker 等专业工具
- 不提供实时监控（`top`/`htop`）；本项目只做 `ps` 的快照输出

## 2. 工作区结构

```
ps/
├─ Cargo.toml                  # [workspace] members = ["ps-sys","ps-core","ps-bin"]
├─ ps-sys/                     # 第 1 层：低层 Win32 绑定
├─ ps-core/                    # 第 2 层：数据模型 + 列/过滤/排序/树
├─ ps-bin/                     # 第 3 层：CLI 入口
├─ scripts/release.ps1         # release 构建脚本
├─ docs/superpowers/specs/2026-06-02-ps-on-windows-design.md
└─ .gitignore                  # target/, dist/, *.exe
```

依赖方向：`ps-bin → ps-core → ps-sys`（单向无循环）。

## 3. ps-sys 边界

### 3.1 模块
- `error.rs` — `thiserror` 错误枚举
- `snapshot.rs` — 进程枚举入口（`EnumProcesses` 主，`NtQuerySystemInformation(SystemProcessInformation)` 补 PPID）
- `procinfo.rs` — 单进程详细信息查询
- `token.rs` — `OpenProcessToken` + `LookupAccountSidW`
- `threads.rs` — `Toolhelp32` 线程快照，输出 PID→thread_count map
- `cmdline.rs` — 从 PEB 读命令行（`NtQueryInformationProcess` + `ReadProcessMemory`）
- `times.rs` — `GetProcessTimes` → `SystemTime` / `Duration`
- `util.rs` — SID→string 转换、UTF-16 ↔ String 工具

### 3.2 主要 Win32 调用
| 信息 | API |
|---|---|
| PID 全集 | `EnumProcesses` |
| 父 PID | `NtQuerySystemInformation(SystemProcessInformation)` |
| ImageName | `QueryFullProcessImageNameW` |
| SessionId | `ProcessIdToSessionId` |
| 用户 / SID | `OpenProcessToken` + `GetTokenInformation(TokenUser)` + `LookupAccountSidW` |
| 启动时间 / CPU 时间 | `GetProcessTimes` |
| 优先级 | `GetPriorityClass` |
| 内存 | `GetProcessMemoryInfo` |
| 线程数 | `CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0)` + `THREADENTRY32` |
| 命令行 | `NtQueryInformationProcess` 拿 PEB → `ReadProcessMemory` 读 CommandLine |
| Wow64 / Elevation | `IsWow64Process` / `GetTokenInformation(TokenElevation)` |
| 句柄数 | `GetProcessHandleCount` |

### 3.3 错误类型
`Error` 变体：`Win32`、`InvalidPid`、`PermissionDenied(u32)`、`NotFound(u32)`、`PebRead(u32)`、`Internal`。

### 3.4 nice 映射
| Priority Class | nice |
|---|---|
| IDLE_PRIORITY_CLASS | 19 |
| BELOW_NORMAL_PRIORITY_CLASS | 15 |
| NORMAL_PRIORITY_CLASS | 10 |
| ABOVE_NORMAL_PRIORITY_CLASS | 5 |
| HIGH_PRIORITY_CLASS | -5 |
| REALTIME_PRIORITY_CLASS | -20 |

### 3.5 平台守卫
`Cargo.toml` 用 `windows = { version = "0.58", features = [...] }`，并由 `target_os = "windows"` 限制；`lib.rs` 顶部 `#[cfg(windows)]` 守卫；非 Windows 平台 stub 返回固定值，方便未来在 Linux 上做 mock 测试。

## 4. ps-core 边界

### 4.1 Process 结构
字段：`pid, ppid, tgid, session_id, name, image_path, cmdline, user(name,domain,sid), state, priority_class, nice, start_time, cpu_time, kernel_time, user_time, thread_count, handles, working_set, peak_working_set, virtual_size, exit_status, flags(FOREGROUND|DEBUGGED|ELEVATED|WOW64)`。

设计原则：字段按需添加，禁止把 `Option` 字段变成"信息丢弃"借口；未支持列用 trait `Column::supported() == false` 处理。

### 4.2 Column trait
```rust
pub trait Column {
    fn kind(&self) -> ColumnKind;
    fn header(&self) -> &'static str;
    fn width(&self) -> usize;
    fn format(&self, p: &Process, ctx: &FormatCtx) -> String;
    fn supported(&self) -> bool { true }
}
```
`FormatCtx { now: Instant, prev_sample: Option<&ProcessSample>, width: WidthMode }` 用于 `pcpu/pmem/etime/etimes`。

### 4.3 Selector
字段：`pids`、`users`、`group`、`session`、`comm_patterns`、`tty`、`include_all`、`include_no_ttys`、通用 `filter: Vec<FilterExpr>`。

`FilterExpr`：`Eq(String,String)` / `Lt(String,String)` / `Gt(String,String)` / `Ne(String,String)`。

### 4.4 列清单
**标识**：`pid`(PID)、`ppid`(PPID)、`pgid`(PGID)、`sid`(SID)、`tid`(TID)、`pgrp`(PGRP)。

**凭据**：`uid`(UID)、`user`(USER)、`gid`(GID)、`group`(GROUP)、`ruser`(RUSER)、`rgroup`(RGROUP)、`suser`(SUSER)。

**资源**：`vsz`(VSZ)、`rss`(RSS)、`pmem`(PMEM)、`pcpu`(PCPU)、`pri`(PRI)、`nice`(NI)、`class`(CLASS)。

**时间**：`etime`(ELAPSED)、`etimes`(ELAPSED 秒)、`times`(TIMES)、`time`(TIME)、`stime`(STIME)、`start`(START)、`lstart`(LSTART)。

**状态/标志**：`stat`(STAT)、`flags`(FLAGS)、`tty`(TTY)、`tt`(TT)、`sess`(SESS)。

**命令**：`comm`(COMM)、`args`(ARGS)、`cmd`(CMD)、`command`(COMMAND)。

**其他**：`c`(C)、`cp`(CP)、`thcount`(THCNT)、`nlwp`(NLWP)。

**未实现（输出 `?`）**：`wchan`、`sig`、`blocked`、`ignored`、`caught`、`fpe`、`bsdstart`、`bsdtime`。

### 4.5 排序
`SortSpec { key: String, reverse: bool }`，支持逗号多键，前缀 `-` 表示降序。`--sort` 未给时默认按 PID 升序（与 Linux ps 一致）。

### 4.6 进程树
- 构建：HashMap<PID, PPID> → DFS
- 根：PID 4（System）或所有 PPID 缺失的进程
- 兄弟节点默认按 PID 升序；`--sort` 时按其 key
- 前缀符号 UTF-8（默认）：`─ ├─ └─ │`
- `--ascii-lines` 切换：`+ \ | -`
- `--forest` 模式只输出 comm/command 列 + 树前缀

### 4.7 错误
`Error` 变体：`Sys`、`ParseColumn`、`ParseSelector`、`PermissionDenied`，用 `thiserror`。

## 5. ps-bin 边界

### 5.1 模块
- `main.rs` — 入口
- `args.rs` — clap 派生结构
- `render/{mod,table,forest,color}.rs` — 渲染层
- `compat.rs` — 退出码、行为兼容

### 5.2 clap Cli 结构
`Clap` 字段（节选）：`all(-A/-e)`、`no_ttys(-x)`、`full(-f)`、`long(-l)`、`all_with_tty(-a)`、`format(-o)`、`pid(-p)`、`user(-u)`、`comm(-C)`、`session(-s)`、`sort(-O/--sort)`、`forest`、`ascii_lines`、`no_headers`、`threads(-L)`、`no_align`、`filter`、`raw`。

### 5.3 POSIX 默认列
| 触发 | 默认列 |
|---|---|
| 啥也没给 | `pid,tty,time,cmd` |
| `-f` / `--full` | `uid,pid,ppid,c,stime,tty,time,cmd` |
| `-l` / `--long` | `f,s,uid,pid,ppid,c,pri,ni,addr,sz,wchan,stime,tty,time,cmd` |
| `-a -x` | `user,pid,ppid,pgid,sid,tty,stat,start,time,command` |
| `-A` / `-e` | `pid,tty,time,cmd` |
| `--forest` | `-a` 风格 + 树前缀 |

### 5.4 渲染
- **Table**：先收集每列实际宽度，数字右对齐，字符串左对齐，列间单空格；`--no-align` 跳过对齐
- **Forest**：前序遍历节点，按 `─ ├─ └─ │` 加前缀；`--ascii-lines` 切到 ASCII
- **颜色**：默认关闭；Windows 10 1607+ 上 `ENABLE_VIRTUAL_TERMINAL_PROCESSING` 启用 ANSI

### 5.5 退出码
| 情况 | code |
|---|---|
| 正常 | 0 |
| 参数错误 | 2 |
| selector 无匹配 | 1 |
| 系统调用失败 | 1 |

### 5.6 main 流程
```
Cli::parse → resolve_columns → build_selector
→ ps_core::snapshot() → filter(selector) → filter(filter_expr) → sort
→ render Table/Forest
→ 退出码
```

## 6. 测试

### 6.1 ps-sys
- `snapshot()` 至少包含当前进程 PID
- `process(current_pid).user` 包含 SID
- `process(system_pid).cmdline` 在 SYSTEM 上 = `None`（不 panic）

### 6.2 ps-core 单元测试
- `column::pid` 输出正确
- `column::pcpu` 在无 prev 时输出 `?`
- `column::etime` 格式 `[[DD-]hh:]mm:ss`
- `Selector` 多个 `-p` 取并集
- `Selector` `-C` 匹配命令行第一个 token
- `SortSpec` 多键 + 负号降序
- `Forest` 单根、深度
- `Forest` ASCII 模式

### 6.3 ps-bin 集成测试（assert_cmd）
- `ps` 退出 0 + ≥1 行
- `ps -A` 行数 > 50
- `ps -p <current>` ≥1 行
- `ps -p 999999` 退出 1
- `ps -o pid,comm` 表头正确
- `ps -f` ≥7 列
- `ps --forest` 含 `─` 或 `+`
- `ps --sort -pcpu` 第一行 CPU 不为 `?`
- `ps --no-headers` 第一行非表头
- `ps --invalid-flag` 退出 2

### 6.4 CI
`.github/workflows/ci.yml` 在 `windows-latest` 上跑 `cargo build`、`cargo test`、`cargo clippy -- -D warnings`、`cargo fmt --check`。

## 7. 已知限制

- `ps -ef` 的 cmdline 来自 `QueryFullProcessImageName`，可能与 Linux `/proc/<pid>/cmdline` 字节不完全一致
- SYSTEM 进程（PID 4）cmdline 通常 = `None`
- 32 位 ps 自身在 64 位 Windows 上看不到部分 32 位进程的内核时间
- `tty` 在 Windows 上无法准确获取，绝大多数情况输出 `?`
- `wchan` 在 Windows 上无对应概念，永远输出 `?`

## 8. 发布

`scripts/release.ps1`：
```powershell
cargo build --release --workspace
$dst = "dist"
New-Item -ItemType Directory -Force -Path $dst | Out-Null
Copy-Item "target/release/ps.exe" "$dst/ps.exe"
Get-ChildItem $dst
```

不发布预编译二进制（用户需求中未要求 CI 交叉编译），仅提供 release 脚本。
