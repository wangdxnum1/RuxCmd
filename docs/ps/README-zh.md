# ps — Linux 风格的 Windows 进程查看器 (Rust)

一个原生的 Windows 版 Linux `ps` 命令。列名、标志和快捷键遵循 procps-ng `ps` 的风格，让你在 Windows 上也能使用 Linux 的操作习惯。

> 基于 Win32 API (EnumProcesses + Toolhelp32 + PEB) 实现。
> 无法获取的值（如权限拒绝、令牌缺失等）会显示为 `?`，而不是中止列表。

## 快速开始

```powershell
# 从仓库根目录构建
cargo build -p ps-bin --release --target x86_64-pc-windows-msvc --locked
.\target\x86_64-pc-windows-msvc\release\ps.exe

# 或构建完整且经过验证的套件 ZIP
pwsh -File scripts/release.ps1
```

```bash
# 默认列：PID TTY TIME COMM
ps

# 完整列表
ps -ef

# 自定义列
ps -o pid,user,pcpu,etime,comm

# 按 CPU 降序、PID 升序排序
ps --sort=-pcpu,pid

# 进程树
ps --forest

# 按 PID / 用户 / 命令过滤
ps -p 1234,5678
ps -u Tim
ps -C chrome

# 跳过 PEB 查询（在有大量特权进程的系统上更快）
ps -ef --no-cmdline
```

## 命令标志

### 进程选择
- `-e`, `--everyone` — 选择所有进程
- `-A` — `-e` 的别名
- `-a` — 选择所有有终端的进程
- `-x` — 包含无终端的进程

### 输出格式
- `-f`, `--full` — 完整列表 (`uid,pid,ppid,c,stime,tty,time,cmd`)
- `-F`, `--extra-full` — 扩展完整列表 (`…+vsz,rss,comm,args`)
- `-l`, `--long` — 长格式
- `-j`, `--jobs` — 作业格式
- `-o`, `--format COLS` — 自定义列（逗号分隔）
- `--no-headers` — 省略表头行
- `--list-columns` — 打印所有可用列名并退出
- `--write FILE` — 写入文件而非标准输出
- `--headers-repeat N` — 每 N 行重复一次表头

### 过滤器
- `-p`, `--pid LIST` — 按 PID 过滤（逗号分隔，可重复）
- `-u`, `--user LIST` — 按有效用户过滤（用户名或 SID）
- `-U`, `--User LIST` — 按真实用户过滤（用户名或 SID）
- `-C`, `--comm LIST` — 按命令名字符串匹配
- `--sort KEY[,KEY…]` — 按列排序，前缀 `-` 表示降序

### 进程树
- `--forest` — 显示父子层级关系
- `--ascii` — 使用 ASCII 分支字符（默认使用 UTF-8 框线字符）

### 性能优化
- `--no-cmdline` — 跳过基于 PEB 的命令行查询。当系统中有大量无法访问的系统进程时很有用。

## 支持的列

运行 `ps --list-columns` 可打印完整列表。主要列如下：

| 列名 | 描述 |
| ---- | ---- |
| `pid` | 进程 ID |
| `ppid` | 父进程 ID |
| `user` | 有效用户（解析后的名称） |
| `ruser` | 真实用户（解析后的名称） |
| `comm` | 可执行文件名（如 `powershell.exe`） |
| `args` | 完整命令行（带引号） |
| `cmd` | 截断的命令行（`-f` 风格） |
| `pcpu` | `%CPU`（CPU 时间 / 运行时间） |
| `pmem` | `%MEM`（工作集 / 总内存） |
| `vsz` | 虚拟内存大小（KiB） |
| `rss` | 驻留集大小（KiB） |
| `stat` | 单字母状态（从 Windows 状态映射） |
| `ni` | Nice 值（从 Windows 优先级类映射） |
| `pri` | 优先级 |
| `start` / `stime` / `etime` | 启动时间 / 启动时刻 / 运行时长 |
| `tty` | 控制终端（无则显示 `?`） |
| `sid` | 会话 ID |
| `pgid` | 进程组 ID |
| `thcount` / `nlwp` | 线程数 |
| `f` | 标志（进程标志） |
| `c` | CPU 利用率（CPU * 100 取整） |
| `time` / `times` | CPU 时间（HH:MM:SS / 秒数） |

后续计划实现的列（P1，尚未注册——请求会打印
`ps: warning: unknown column '<name>', skipped`）：`sz`、`psr`、`wchan`、
`cputime`。

## 不支持 / 尽力映射的列

Windows 并不直接暴露每个 Linux 字段的概念。以下是尽力映射（见 [跨平台映射](#跨平台映射)）：

- `vsz`, `rss` — 从 `GetProcessMemoryInfo` 派生
- `pmem` — 工作集除以 `GlobalMemoryStatusEx` 返回的物理内存总量
- `time` / `times` — 从 `GetProcessTimes` 派生
- `pcpu` — 从 CPU 时间与进程启动后的流逝时间计算
- `stat` — 从 `GetExitCodeProcess` + 主窗口状态合成
- `ni`, `pri` — 从 `GetPriorityClass` 合成
- `tty` — Windows 没有 TTY；无控制台窗口时显示 `?`
- `ruser` — 目前与 `user` 相同（Windows 令牌是每会话的）

暂时没有 Windows 对应概念的列（`sz`、`psr`、`wchan`）一律不注册——见上
方的"后续计划实现的列"说明——而不是显示为 `?`。

## 项目架构

```text
crates/
├── ps-sys/          # ps、pgrep、pkill 共用的 Win32 采集层
│   └── src/         # 快照、进程信息、令牌、时间和 PEB 命令行
└── ps-bin/          # 一个命令包，生成 ps.exe
    └── src/
        ├── main.rs / cli.rs
        ├── process.rs                 # 内部进程与用户模型
        ├── assemble.rs / meminfo.rs    # 原生数据映射及总内存缓存
        ├── columns/                   # 静态元数据表 + 四组格式化函数
        │   ├── mod.rs
        │   ├── identity.rs / resources.rs
        │   └── time.rs / command.rs
        └── filter.rs / sort.rs / tree.rs / render.rs
```

原先只有一个调用方的 `ps-core` 已收回 `ps-bin` 内部。列定义使用静态描述表与
普通函数，不再一列一个 trait 对象和文件。42 个已注册名称（含别名）的标题、
宽度、对齐及格式均保留；排序键只针对选定列解析一次。归档的原始设计计划记录
的是过去的三包结构。

## 跨平台映射

| Linux 概念 | Windows 数据源 |
| ---------- | -------------- |
| `uid` / `user` | `OpenProcessToken` + `LookupAccountSidW` |
| `ppid` | Toolhelp32 parent PID |
| `rss` / `vsz` | `GetProcessMemoryInfo` (WorkingSetSize / PagefileUsage) |
| `pcpu` | (CPU 时间) ÷ (自 `CreateTime` 以来的运行时间) |
| `time` / `times` | `GetProcessTimes` |
| `pri` / `ni` | `GetPriorityClass` 映射到 0..40 范围 |
| `stat` | `S` 占位（无法取得每个进程的可运行状态） |
| `start` | `GetProcessTimes`（创建时间） |
| `cmd` / `args` | 从 PEB 获取 `RTL_USER_PROCESS_PARAMETERS` |
| `tty` | `?`（无法取得控制终端） |
| `thcount` | `Thread32First/Next` 计数 |
| `pgid` / `sid` | pgid 使用 PID，sid 使用 Windows 会话 ID |

## 测试

```powershell
cargo test -p ps-bin -p ps-sys -p pgrep -p pkill --all-targets --target x86_64-pc-windows-msvc --locked
```

测试覆盖共享 Win32 层、ps 内部模型/列及 pgrep/pkill 消费者。42 列基线快照及别名/排序/缺失数据用例锁定重构前行为；需要正常 Windows API 权限，无需外部服务。

## 许可证

MIT — 详见 [LICENSE](LICENSE)。
