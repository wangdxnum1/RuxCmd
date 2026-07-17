# 第 8 轮 Linux 工具移植设计：文本处理工具集

**日期**：2026-07-18
**作者**：Tim
**背景**：前 7 轮已移植 88 个 Linux 命令行工具到 Windows (Rust, 静态 CRT MT 模式)。本轮移植 5 个文本处理类工具。

---

## 1. 范围与目标

### 1.1 本轮移植的 5 个工具

| # | 工具 | 功能简述 | 实现难度 | 依赖策略 |
|---|------|----------|----------|----------|
| 1 | `awk` | 文本处理编程语言，支持模式匹配和动作执行 | ⭐⭐⭐ | 标准库手写解析器 |
| 2 | `iconv` | 字符编码转换（UTF-8/GBK/GB2312/ISO-8859-1等） | ⭐⭐ | encoding_rs crate（已在 cargo cache） |
| 3 | `od` (增强) | 八进制/十六进制转储（增强 `-j` `-N` `-A` 等选项） | ⭐⭐ | 在现有基础上扩展 |
| 4 | `hexdump` | 十六进制转储工具（xxd 风格输出） | ⭐⭐ | 标准库 |
| 5 | `strings` (增强) | 从二进制提取字符串（增强 `-f` `-n` 等） | ⭐ | 在现有基础上扩展 |

### 1.2 不在范围内

- 不引入新 crate 依赖（除 `clap = { version = "4", features = ["derive"] }` 和 `encoding_rs`）；
- awk 不实现完整 GNU awk 语法（仅实现常用子集）；
- 不处理跨平台文件路径编码（沿用 Rust Path/PathBuf，Windows 原生）。

---

## 2. 总体架构与代码组织

### 2.1 目录结构

每个工具独立 crate：
```
d:\Work\rust\
├── awk/
├── iconv/
├── od/          (增强现有)
├── hexdump/
└── strings/     (增强现有)
```

每个 crate 包含：
- `Cargo.toml` — 包定义与依赖
- `src/cli.rs` — 命令行参数定义（clap derive）
- `src/main.rs` — 主逻辑

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

iconv 需要额外依赖：
```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
encoding_rs = "0.8"
```

### 2.3 src/cli.rs 统一模板

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "<TOOL_NAME>", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    // 其他工具专属参数...
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
```

**关键约定**：`disable_version_flag = true` + 自定义 `-v/--version`（不使用 `-V`），与前 88 个工具保持一致。

### 2.4 src/main.rs 统一骨架

```rust
mod cli;

use clap::Parser;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("<TOOL_NAME> 0.1.0");
        return;
    }

    let mut exit_code = 0i32;

    // ... 工具主逻辑 ...

    std::process::exit(exit_code);
}
```

### 2.5 全局配置（已就绪）

- **静态 CRT 链接**：`.cargo/config.toml` 已设置 `target-feature=+crt-static`；
- **构建流水线**：`build-all.bat` 调用 `cargo build --release --offline`；
- **忽略文件**：`.gitignore` 已忽略 `**/target/`、`**/*.exe`、`**/*.pdb`、`Cargo.lock`。

---

## 3. 各工具详细设计

### 3.1 awk

**CLI**：
```
awk [OPTIONS] 'PATTERN { ACTION }' [FILE]...
  -F, --field-separator=SEP   字段分隔符（默认空白）
  -v, --assign=VAR=VALUE      设置变量
  -f, --file=FILE             从文件读取 awk 程序
  -v, --version               版本
  FILE                        输入文件（空或 '-' 读 stdin）
```

**支持的语法子集**：

**变量**：
- 内置：`NR`（记录号）、`NF`（字段数）、`$0`（整行）、`$1`-`$NF`（字段）、`FS`（字段分隔符）、`OFS`（输出字段分隔符）、`RS`（记录分隔符）、`ORS`（输出记录分隔符）
- 用户变量：通过 `-v` 设置或在程序中赋值

**模式**：
- `/regex/`：正则匹配（简化为子串匹配）
- `expression`：表达式匹配
- `BEGIN`：处理前执行
- `END`：处理后执行

**动作**：
- `print`：输出（支持 `print $1, $2`）
- `printf`：格式化输出（支持基本格式）
- `if (cond) { } else { }`：条件分支
- `for (i = 0; i < n; i++) { }`：循环
- `next`：跳过当前记录

**实现要点**：
- 逐行读入，按 `FS` 分割为字段；
- 维护 `NR`、`NF`、`$0`、`$1..$NF`；
- 解析简单的 awk 程序语法（词法分析 + 简单语法树）；
- 支持 `print`、`printf`、`if`、`for`、赋值等；
- 默认 `{ print }` 动作（匹配模式后打印整行）。

---

### 3.2 iconv

**CLI**：
```
iconv [OPTIONS] -f FROM -t TO [FILE]...
  -f, --from-code=NAME     源编码
  -t, --to-code=NAME       目标编码
  -o, --output=FILE        输出文件
  -l, --list               列出支持的编码
  -v, --version            版本
  FILE                     输入文件（空或 '-' 读 stdin）
```

**支持的编码**：

| 编码名 | 说明 |
|--------|------|
| UTF-8 | UTF-8 |
| UTF-16 | UTF-16（带 BOM） |
| UTF-16BE | UTF-16 大端 |
| UTF-16LE | UTF-16 小端 |
| GBK | GBK 编码 |
| GB2312 | GB2312 编码 |
| GB18030 | GB18030 编码 |
| ISO-8859-1 | Latin-1 |
| US-ASCII | ASCII |

**实现要点**：
- 使用 `encoding_rs` crate 进行编码转换；
- `-l` 列出所有支持的编码；
- 读取输入文件，转换编码后写入输出；
- 错误处理：非法字节替换为 `�`。

---

### 3.3 od (增强)

**CLI**（在现有基础上增强）：
```
od [OPTIONS] [FILE]
  -A, --address-radix=RADIX   地址基数：o(八进制)/d(十进制)/x(十六进制)/n(无)
  -j, --skip-bytes=N          跳过前 N 字节
  -N, --read-bytes=N          最多读取 N 字节
  -t, --format=TYPE           输出格式：o(八进制)/x(十六进制)/d(十进制)/c(字符)/s(字符串)/f(浮点)
  -x, --hexadecimal           十六进制模式（等价 -t x2）
  -c, --ascii                 ASCII 字符模式（等价 -t c）
  -d, --decimal               十进制模式（等价 -t d2）
  -o, --octal                 八进制模式（等价 -t o2）
  -w, --width=N               每行字节数（默认 16）
  -v, --version               版本
```

**增强要点**：
- 新增 `-j`：跳过前 N 字节；
- 新增 `-N`：限制读取字节数；
- 新增 `-w`：自定义每行宽度；
- 增强 `-A n`：无地址模式；
- 增强 `-t` 格式字符串解析（支持 `t x4`, `t d4` 等）。

---

### 3.4 hexdump

**CLI**：
```
hexdump [OPTIONS] [FILE]
  -C, --canonical             标准十六进制+ASCII格式（类似 xxd）
  -d, --decimal               双字节十进制
  -o, --octal                 双字节八进制
  -x, --hexadecimal           双字节十六进制
  -c, --ascii                 单字节字符
  -s, --skip=N                跳过前 N 字节
  -n, --length=N              最多显示 N 字节
  -v, --version               版本
```

**输出格式**（`-C` 模式）：
```
00000000  48 65 6c 6c 6f 20 57 6f  72 6c 64 21 0a 00        |Hello World!..|
0000000e
```

**实现要点**：
- 地址 + 十六进制字节（分组）+ ASCII 列；
- `-C` 模式：16 字节/行，8 字节一组，右侧 ASCII；
- `-s`：跳过前 N 字节；
- `-n`：限制显示长度。

---

### 3.5 strings (增强)

**CLI**（在现有基础上增强）：
```
strings [OPTIONS] [FILE]...
  -n, --bytes=N          最小字符串长度（默认 4）
  -a, --all              扫描整个文件
  -e, --encoding=ENC     编码：s(7-bit)/S(8-bit)/b(UTF-16BE)/l(UTF-16LE)/B(UTF-32BE)/L(UTF-32LE)
  -f, --print-file-name  输出时前缀文件名
  -t, --radix=RADIX      输出字符串位置：o(八进制)/x(十六进制)/d(十进制)
  -v, --version          版本
```

**增强要点**：
- 新增 `-f`：多文件时前缀文件名；
- 新增 `-t`：输出字符串在文件中的偏移位置；
- 完善 UTF-16/UTF-32 编码支持。

---

## 4. 错误处理规范

### 4.1 统一错误输出格式

| 场景 | stderr 格式 | 退出码 |
|------|-------------|--------|
| 打开文件失败 | `"<tool>: <path>: <os_error>"` | 1 |
| 读取失败 | `"<tool>: error reading <stdin|path>: <error>"` | 1 |
| 写入失败 | `"<tool>: write error: <error>"` | 1 |
| 无效参数 | `"<tool>: invalid <param_name> '<value>': <reason>"` | 1 |
| 编码转换失败 | `"<tool>: conversion error: <error>"` | 1 |

### 4.2 `-` 作为文件

所有工具遵循：FILE 列表中出现 `"-"` 表示读取 stdin。

---

## 5. 构建集成

### 5.1 build-all.bat 变更

PROJECTS 变量末尾追加：
```
awk iconv hexdump
```

（od 和 strings 已有，无需追加）

### 5.2 构建命令

```bat
cargo build --release --offline
```

### 5.3 Git 提交

- 提交文件：`awk/`、`iconv/`、`hexdump/` 三个新工具，以及修改后的 `od/`、`strings/`、`build-all.bat`、设计文档；
- 不纳入：`target/`、`.exe`、`.pdb`、`Cargo.lock`。

---

## 6. 验收检查清单

| 项 | 说明 |
|----|------|
| 编译通过 | 5 个工具 `cargo build --release --offline` 全部成功 |
| 基本功能冒烟 | 每个工具至少 3 条常见用法验证通过 |
| build-all.bat 集成 | 新工具已加入 PROJECTS 列表 |
| 产物复制 | `D:\develop\rust-tools` 目录出现新 exe |
| Git 状态 | 只显示预期源代码文件 |

---

## 7. 风险与回退

| 风险 | 缓解策略 |
|------|----------|
| awk 语法解析复杂 | 仅实现常用子集，不在帮助中声明支持的特性 |
| iconv 编码支持不全 | 优先支持最常用编码（UTF-8/GBK/GB2312） |
| encoding_rs 不在 cargo cache | 提前 `cargo fetch` 确保缓存 |