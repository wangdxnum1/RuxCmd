# 第 10 轮 Linux 工具移植设计：系统监控工具集

**日期**：2026-07-24
**作者**：Tim
**背景**：前 9 轮已移植 106 个 Linux 命令行工具到 Windows (Rust, 静态 CRT MT 模式)。本轮移植 5 个系统监控类工具。

---

## 1. 范围与目标

### 1.1 本轮移植的 5 个工具

| # | 工具 | 功能简述 | 实现难度 | 依赖策略 |
|---|------|----------|----------|----------|
| 1 | `vmstat` | 虚拟内存统计（进程数、内存、交换区、I/O） | ⭐⭐⭐ | Windows API |
| 2 | `iostat` | 磁盘 I/O 统计（读写速率、队列长度） | ⭐⭐⭐ | Windows API |
| 3 | `mpstat` | CPU 使用情况统计（多核/单核） | ⭐⭐⭐ | Windows API |
| 4 | `sar` | 系统活动报告（综合监控） | ⭐⭐⭐ | Windows API |
| 5 | `top` | 实时系统监控（类似任务管理器） | ⭐⭐⭐⭐ | Windows API + 终端交互 |

### 1.2 已有的系统监控工具

| 工具 | 状态 | 说明 |
|------|------|------|
| `free` | ✅ 已实现 | 内存使用情况 |
| `uptime` | ✅ 已实现 | 系统运行时间 |
| `ps` | ✅ 已实现 | 进程列表 |
| `w` | ✅ 已实现 | 登录用户和进程 |
| `who` | ✅ 已实现 | 登录用户 |

### 1.3 不在范围内

- top 不实现完整交互功能（仅静态输出）；
- sar 不实现历史数据收集；
- 不引入图形界面；
- 不支持复杂过滤和排序选项。

---

## 2. 总体架构与代码组织

### 2.1 目录结构

```
d:\Work\rust\
├── vmstat/
├── iostat/
├── mpstat/
├── sar/
└── top/
```

### 2.2 Cargo.toml 统一模板

```toml
[package]
name = "<TOOL_NAME>"
version = "0.1.0"
edition = "2024"
description = "<SHORT_DESCRIPTION>"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

**额外依赖**（Windows API）：
```toml
windows-sys = { version = "0.61", features = ["Win32_System_Performance", "Win32_ProcessStatus", "Win32_System_SystemInformation"] }
```

### 2.3 关键约定

- `disable_version_flag = true` + 自定义 `-v/--version`；
- 错误输出格式：`"<tool>: <message>"`；
- 使用 Windows Performance Counter API 或 WMI 获取系统数据。

---

## 3. 各工具详细设计

### 3.1 vmstat

**CLI**：
```
vmstat [OPTIONS] [DELAY [COUNT]]
  -n        只显示一次头部信息
  -v        版本
```

**输出格式**：
```
procs -----------memory---------- ---swap-- -----io---- --system-- ------cpu-----
 r  b   swpd   free   buff  cache   si   so    bi    bo   in   cs us sy id wa
 0  0      0 8388608      0      0    0    0     0     0    0    0  0  0 100  0
```

**实现要点**：
- 使用 `GlobalMemoryStatusEx` 获取内存信息；
- 使用性能计数器获取进程数、I/O 统计；
- 支持定时刷新输出。

---

### 3.2 iostat

**CLI**：
```
iostat [OPTIONS] [DELAY [COUNT]]
  -d        只显示磁盘统计
  -x        扩展统计
  -v        版本
```

**输出格式**：
```
Device:            tps    kB_read/s    kB_wrtn/s    kB_read    kB_wrtn
C:                0.00         0.00         0.00          0          0
D:                1.25        60.00        40.00        240        160
```

**实现要点**：
- 使用性能计数器获取磁盘 I/O；
- 遍历所有磁盘驱动器；
- 计算读写速率。

---

### 3.3 mpstat

**CLI**：
```
mpstat [OPTIONS] [DELAY [COUNT]]
  -P CPU    指定 CPU（ALL 表示所有）
  -v        版本
```

**输出格式**：
```
CPU    %usr   %nice    %sys %iowait    %irq   %soft  %steal  %guest  %gnice   %idle
  0      5.0      0.0      2.0      0.0      0.0      0.0      0.0      0.0     93.0
  1      3.0      0.0      1.5      0.0      0.0      0.0      0.0      0.0     95.5
```

**实现要点**：
- 使用性能计数器获取每个 CPU 的使用率；
- 支持指定 CPU 或显示全部；
- 计算用户态/系统态/空闲时间比例。

---

### 3.4 sar

**CLI**：
```
sar [OPTIONS] [DELAY [COUNT]]
  -u        CPU 使用率
  -r        内存使用率
  -d        磁盘 I/O
  -v        版本
```

**输出格式**：
```
10:00:00 AM     CPU     %user     %nice   %system   %iowait    %steal     %idle
10:00:01 AM     all      5.25      0.00      2.15      0.00      0.00     92.60
```

**实现要点**：
- 综合 CPU、内存、磁盘统计；
- 支持不同参数显示不同指标；
- 定时采样输出。

---

### 3.5 top

**CLI**：
```
top [OPTIONS]
  -n NUM    刷新次数后退出
  -d SEC    刷新间隔（秒）
  -v        版本
```

**输出格式**：
```
top - 10:00:00 up 2 days, 15:30,  2 users,  load average: 0.10, 0.05, 0.01
Tasks: 150 total,   1 running, 149 sleeping,   0 stopped,   0 zombie
%Cpu(s):  5.2 us,  2.1 sy,  0.0 ni, 92.6 id,  0.1 wa,  0.0 hi,  0.0 si,  0.0 st
MiB Mem :   8192.0 total,   1024.0 free,   4096.0 used,   3072.0 buff/cache
MiB Swap:      0.0 total,      0.0 free,      0.0 used.   3584.0 avail Mem

  PID USER      PR  NI    VIRT    RES    SHR S  %CPU %MEM     TIME+ COMMAND
 1234 user      20   0  500.0m  50.0m  10.0m S   2.0  0.6   0:30.00 chrome
 5678 user      20   0  300.0m  30.0m   5.0m S   1.0  0.4   0:15.00 explorer
```

**实现要点**：
- 使用 `CreateToolhelp32Snapshot` 获取进程列表；
- 使用性能计数器获取 CPU/内存使用；
- 支持定时刷新和指定刷新次数。

---

## 4. 错误处理规范

| 场景 | stderr 格式 | 退出码 |
|------|-------------|--------|
| 无法获取性能数据 | `"<tool>: cannot access performance data"` | 1 |
| 参数解析失败 | `"<tool>: invalid argument: '<arg>'"` | 1 |
| 权限不足 | `"<tool>: permission denied"` | 1 |

---

## 5. 构建集成

PROJECTS 变量末尾追加：
```
vmstat iostat mpstat sar top
```

---

## 6. 验收检查清单

| 项 | 说明 |
|----|------|
| 编译通过 | 5 个工具全部成功 |
| 基本功能冒烟 | 每个工具至少 3 条验证 |
| build-all.bat 集成 | 新工具已加入 |
| 产物复制 | 输出目录出现新 exe |
| Git 状态 | 只显示预期文件 |

---

## 7. 风险与回退

| 风险 | 缓解策略 |
|------|----------|
| 性能计数器访问受限 | 在帮助信息中注明可能需要管理员权限 |
| top 交互复杂 | 先实现静态输出，交互功能留待后续 |
| Windows API 差异 | 使用 windows-sys 库抽象平台差异 |
