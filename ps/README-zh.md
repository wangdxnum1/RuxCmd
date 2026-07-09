# ps — Linux 风格的 Windows 进程查看器 (Rust)

一个原生的 Windows 版 Linux `ps` 命令。列名、标志和快捷键遵循 procps-ng `ps` 的风格，让你在 Windows 上也能使用 Linux 的操作习惯。

> 基于 Win32 API (EnumProcesses + Toolhelp32 + PEB) 实现。
> 无法获取的值（如权限拒绝、令牌缺失等）会显示为 `?`，而不是中止列表。

## 快速开始

```powershell
# 从源码构建
cargo build --release
.\target\x86_64-pc-windows-msvc\release\ps.exe

# 或运行发布脚本
.\scripts\release.ps1      # 将 ps.exe 放到 .\dist\ 目录
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
- `-F`, `--extra-full` — 扩展完整列表 (`…+rss,psr,comm,args`)
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
| `psr` | 当前分配的处理器 |
| `wchan` | 等待通道 / 原因 |
| `sz` | 大小（4 KiB 页数） |
| `cputime` / `time` | CPU 时间（HH:MM:SS / MM:SS） |

## 不支持 / 尽力映射的列

Windows 并不直接暴露每个 Linux 字段的概念。以下是尽力映射（见 [跨平台映射](#跨平台映射)）：

- `vsz`, `rss` — 从 `GetProcessMemoryInfo` 派生
- `cputime` — 从 `GetProcessTimes` 派生
- `pcpu` — 从 CPU 时间与进程启动后的流逝时间计算
- `stat` — 从 `GetExitCodeProcess` + 主窗口状态合成
- `ni`, `pri` — 从 `GetPriorityClass` 合成
- `tty` — Windows 没有 TTY；无控制台窗口时显示 `?`
- `ruser` — 目前与 `user` 相同（Windows 令牌是每会话的）

没有 Windows 对应概念的字段（如大多数 GUI 应用的 `wchan`）会显示为 `?` 而非省略。

## 项目架构

```
ps/
├── ps-sys/        # 原始 Win32 FFI（支持 no_std，使用 thiserror）
│   ├── snapshot   # EnumProcesses + Toolhelp32
│   ├── procinfo   # PEB + NtQueryInformationProcess
│   ├── token      # OpenProcessToken + LookupAccountSidW
│   ├── times      # GetProcessTimes
│   ├── threads    # 线程计数
│   └── cmdline    # 从 PEB 获取 RTL_USER_PROCESS_PARAMETERS
│
├── ps-core/       # 数据模型 + 业务逻辑（无 Win32 依赖）
│   ├── process    # Process 结构体，Windows → Linux 字段映射
│   ├── columns/   # Column trait + 38+ 实现
│   ├── filter     # -p, -u, -C 选择器
│   ├── sort       # --sort 解析器 + 应用器
│   ├── tree       # Forest 构建器 + 渲染器
│   ├── state      # 状态字母映射
│   └── user       # 用户解析
│
├── ps-bin/        # CLI 入口点
│   ├── cli.rs     # clap 解析器
│   ├── render.rs  # 表格 + 树渲染器
│   └── main.rs    # 编排 snapshot → filter → sort → render
│
└── scripts/
    └── release.ps1
```

## 跨平台映射

| Linux 概念 | Windows 数据源 |
| ---------- | -------------- |
| `uid` / `user` | `OpenProcessToken` + `LookupAccountSidW` |
| `ppid` | `ProcessBasicInformation.InheritedFromUniqueProcessId` |
| `rss` / `vsz` | `GetProcessMemoryInfo` (WorkingSetSize / PagefileUsage) |
| `pcpu` | (CPU 时间) ÷ (自 `CreateTime` 以来的运行时间) |
| `cputime` | `GetProcessTimes` |
| `pri` / `ni` | `GetPriorityClass` 映射到 0..40 范围 |
| `stat` | `GetExitCodeProcess` + 主窗口存在性 |
| `start` | `GetProcessTimes`（创建时间） |
| `cmd` / `args` | 从 PEB 获取 `RTL_USER_PROCESS_PARAMETERS` |
| `tty` | `GetConsoleWindow`（为空则显示 `?`） |
| `thcount` | `Thread32First/Next` 计数 |
| `pgid` / `sid` | 两者都使用 `GetProcessId`（Windows 使用会话 = sid） |

## 测试

```powershell
cargo test -p ps-sys -p ps-core
```

单元测试覆盖 Win32 层 (`ps-sys`) 和数据模型/列层 (`ps-core`)。不需要外部服务，可以离线运行。

## 许可证

MIT — 详见 [LICENSE](LICENSE)。