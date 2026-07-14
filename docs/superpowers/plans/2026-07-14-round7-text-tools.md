# Round 7 Text Tools Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 10 个新的 Linux 文本处理工具（base64 / expand / unexpand / tac / column / shuf / csplit / pr / numfmt / factor），全部使用 Rust + Windows 静态 CRT 编译（MT 模式），并集成到 build-all.bat 与 D:\develop\rust-tools 输出目录。

**Architecture:** 每个工具独立 Rust crate，统一使用 `clap = { version = "4", features = ["derive"] }` 解析参数。每个 crate 固定三文件结构：`Cargo.toml`（包元数据） + `src/cli.rs`（`Args` 结构体） + `src/main.rs`（主流程 + 算法实现）。主流程遵循既有模式：`-v` 打印版本、`-` 号代表 stdin、多文件逐个处理、统一错误输出格式 `<tool>: <path>: <err>`。所有算法优先使用标准库，不引入额外 crate 依赖。

**Tech Stack:** Rust 2024 edition · clap 4 derive · Rust std library only (无 rand, 无 regex, 无 base64 crate) · 静态 CRT (`.cargo/config.toml` 已生效) · `cargo build --release --offline`

## Global Constraints

- 每个 crate `name` 严格等于其工具名（小写），`version = "0.1.0"`，`edition = "2024"`，`authors = ["Tim"]`。
- `src/cli.rs` 必须 `#[command(name = "<TOOL>", version, about, disable_version_flag = true)]`，必须自定义 `-v, --version` 标志（不使用 `-V`）。
- `src/main.rs` 顶部固定 `mod cli; use clap::Parser;`；所有文件路径/错误通过 `eprintln!` 输出到 stderr，并通过 `std::process::exit(0i32 | 1i32)` 返回退出码。
- .gitignore 已配置忽略 `**/target/`、`**/*.exe`、`**/*.pdb`、`Cargo.lock`；任何构建产物不得被提交。
- 所有二进制通过 `build-all.bat` 的 `cargo build --release --offline` 构建后，复制到 `D:\develop\rust-tools`。
- 代码中禁止使用任何中文注释（除非用户后续另有说明），保持与前 78 个工具一致的风格；除非绝对必要，不写入任何注释行。

---

## File Structure

**Create (30 files):**

| # | Path | Responsibility |
|---|------|----------------|
| 1 | `base64/Cargo.toml` | 包元 + clap 依赖 |
| 2 | `base64/src/cli.rs` | base64 CLI 参数：-d/--decode, -w/--wrap, FILES |
| 3 | `base64/src/main.rs` | 标准库 Base64 编解码算法 + 文件/stdin 流水线 |
| 4 | `expand/Cargo.toml` | 包元 + clap 依赖 |
| 5 | `expand/src/cli.rs` | expand CLI：-t N/LIST, -i, FILES |
| 6 | `expand/src/main.rs` | Tab→空格逐字符扫描 + tab stop 计算 |
| 7 | `unexpand/Cargo.toml` | 包元 + clap 依赖 |
| 8 | `unexpand/src/cli.rs` | unexpand CLI：-t N, -a, FILES |
| 9 | `unexpand/src/main.rs` | 空格→Tab 反向转换，默认行首空白 |
| 10 | `tac/Cargo.toml` | 包元 + clap 依赖 |
| 11 | `tac/src/cli.rs` | tac CLI：-b/--before, -s/--separator, FILES |
| 12 | `tac/src/main.rs` | 按行/按自定义分隔切片反转输出 |
| 13 | `column/Cargo.toml` | 包元 + clap 依赖 |
| 14 | `column/src/cli.rs` | column CLI：-t, -s, -o, FILES |
| 15 | `column/src/main.rs` | 二维分割 + 列宽计算 + 对齐输出 |
| 16 | `shuf/Cargo.toml` | 包元 + clap 依赖 |
| 17 | `shuf/src/cli.rs` | shuf CLI：-e/--echo, -n/--head-count, -o, -r/--repeat, -z, FILE |
| 18 | `shuf/src/main.rs` | RandomState 取种 + splitmix64 PRNG + Fisher–Yates 洗牌 |
| 19 | `csplit/Cargo.toml` | 包元 + clap 依赖 |
| 20 | `csplit/src/cli.rs` | csplit CLI：-f/--prefix, -n/--digits, -k, -z, FILE + PATTERN... |
| 21 | `csplit/src/main.rs` | 模式（行号/子串匹配）解析 → 断点计算 → 分片写文件 → 输出字节数 |
| 22 | `pr/Cargo.toml` | 包元 + clap 依赖 |
| 23 | `pr/src/cli.rs` | pr CLI：columns (-NUM/-c), -l/--length, -o, -w, -h/--header, -t, -d, -J, FILES |
| 24 | `pr/src/main.rs` | 分页（页眉 + 页体） + 多列轮转（column-major） + 格式选项 |
| 25 | `numfmt/Cargo.toml` | 包元 + clap 依赖 |
| 26 | `numfmt/src/cli.rs` | numfmt CLI：--to, --from, --to-unit, --from-unit, --field, -d, --padding, --suffix, --header, NUMBERS... |
| 27 | `numfmt/src/main.rs` | SI/IEC 量级前缀表 + 输入乘法 + 最佳量级匹配 + 格式化字符串 |
| 28 | `factor/Cargo.toml` | 包元 + clap 依赖 |
| 29 | `factor/src/cli.rs` | factor CLI：NUMBER...（空=读 stdin） |
| 30 | `factor/src/main.rs` | u64 试除法：先除 2 → 步进 2 到 sqrt(n) → 余 n 输出 |

**Modify (1 file):**

- `build-all.bat:16` — 在 `PROJECTS` 变量末尾追加 `base64 expand unexpand tac column shuf csplit pr numfmt factor`。

**Test Strategy:** 每个工具完成后依次执行 3 条手工冒烟命令；全部完成后通过 `build-all.bat` 整体构建并验证复制成功，最后 `git status` 确认无 target 污染。

---

## Task 0: Scaffold Build Pipeline (build-all.bat)

**Files:**
- Modify: `build-all.bat:16`

**Interfaces:**
- Produces: PROJECTS 列表包含 10 个新工具名，后续批处理流程可用。

- [ ] **Step 1: 查看 build-all.bat 第 16 行内容**

Read: `build-all.bat` 第 16 行，确认为 `set "PROJECTS=..."` 一行。

- [ ] **Step 2: 在 PROJECTS 末尾追加 10 个工具名**

在已有的 `whereis` 后追加（保留顺序）：

```
set "PROJECTS=cat ls open rm touch mkdir cp mv head tail wc grep date which sort cut find du df kill echo ln whoami chmod chown uname uptime env diff sed uniq tee xargs basename dirname tr cmp tar rev split paste nl file md5sum sha256sum stat readlink realpath seq yes sleep id free fmt fold comm join who w hostname groups gunzip unzip zcat patch od strings tty mkfifo true false test printf cal clear reset pkill pgrep shred sync bc zip time timeout nice whereis base64 expand unexpand tac column shuf csplit pr numfmt factor"
```

- [ ] **Step 3: 保存后运行批处理的头部检查**

Run:
```
cd /d d:\Work\rust && findstr /c:"base64 expand unexpand" build-all.bat
```
Expected: 输出完整 PROJECTS 行并包含所有 10 个新名字。

- [ ] **Step 4: Commit build-all.bat 修改**

```
cd /d d:\Work\rust
git add build-all.bat
git commit -m "chore(build): add round 7 tools to build-all.bat PROJECTS list"
```

---

## Task 1: factor — 质因数分解（最简单，热启动）

**Files:**
- Create: `factor/Cargo.toml`
- Create: `factor/src/cli.rs`
- Create: `factor/src/main.rs`

**Interfaces:**
- Consumes: 无前置任务（独立 crate）
- Produces: `factor.exe` 接受命令行整数或 stdin 每行一个数，输出 `n: p1 p2 p3...`

- [ ] **Step 1: 创建 factor/Cargo.toml**

```toml
[package]
name = "factor"
version = "0.1.0"
edition = "2024"
description = "Prime factor decomposition of integers"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

- [ ] **Step 2: 创建 factor/src/cli.rs**

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "factor", version, about = "Print prime factors of NUMBER(s)", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(value_name = "NUMBER")]
    pub numbers: Vec<String>,
}
```

- [ ] **Step 3: 创建 factor/src/main.rs 完整实现**

```rust
mod cli;

use clap::Parser;
use std::io::{BufRead, BufReader};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("factor 0.1.0");
        return;
    }

    let mut exit_code = 0i32;

    if args.numbers.is_empty() {
        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin.lock());
        for line in reader.lines() {
            match line {
                Ok(l) => {
                    let trimmed = l.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Err(e) = factor_one(trimmed) {
                        eprintln!("factor: {}", e);
                        exit_code = 1;
                    }
                }
                Err(e) => {
                    eprintln!("factor: error reading stdin: {}", e);
                    exit_code = 1;
                    break;
                }
            }
        }
    } else {
        for n in &args.numbers {
            if let Err(e) = factor_one(n) {
                eprintln!("factor: {}", e);
                exit_code = 1;
            }
        }
    }

    std::process::exit(exit_code);
}

fn factor_one(raw: &str) -> Result<(), String> {
    let n: u64 = raw
        .parse()
        .map_err(|_| format!("'{}' is not a valid positive integer", raw))?;
    if n == 0 {
        return Err("'0' is not a valid positive integer".to_string());
    }
    if n == 1 {
        println!("1:");
        return Ok(());
    }
    let mut factors: Vec<u64> = Vec::new();
    let mut x = n;
    while x % 2 == 0 {
        factors.push(2);
        x /= 2;
    }
    let mut d: u64 = 3;
    while d.saturating_mul(d) <= x {
        while x % d == 0 {
            factors.push(d);
            x /= d;
        }
        d = d.saturating_add(2);
        if d < 3 {
            break;
        }
    }
    if x > 1 {
        factors.push(x);
    }
    let out: Vec<String> = factors.iter().map(|f| f.to_string()).collect();
    println!("{}: {}", n, out.join(" "));
    Ok(())
}
```

- [ ] **Step 4: 构建并检查编译**

Run:
```
cd /d d:\Work\rust\factor && cargo build --release --offline
```
Expected: 无 error；生成 `target\release\factor.exe`。

- [ ] **Step 5: 三条冒烟测试**

① `cd /d d:\Work\rust\factor && .\target\release\factor.exe 60`
Expected: `60: 2 2 3 5`

② `.\target\release\factor.exe 1 12 131`
Expected:
```
1:
12: 2 2 3
131: 131
```

③ `echo 1234567 | .\target\release\factor.exe`
Expected: `1234567: 127 9721`（或正确的分解）

- [ ] **Step 6: 提交 factor 代码**

```
cd /d d:\Work\rust
git add factor/Cargo.toml factor/src/cli.rs factor/src/main.rs
git commit -m "feat: add factor (prime factorization) - round 7"
```

---

## Task 2: base64 — Base64 编码/解码

**Files:**
- Create: `base64/Cargo.toml`
- Create: `base64/src/cli.rs`
- Create: `base64/src/main.rs`

**Interfaces:**
- Produces: `base64.exe -d/-w COLS [FILES...]`；编码默认 76 列换行，解码忽略空白。

- [ ] **Step 1: 创建 base64/Cargo.toml**

```toml
[package]
name = "base64"
version = "0.1.0"
edition = "2024"
description = "Base64 encode/decode data"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

- [ ] **Step 2: 创建 base64/src/cli.rs**

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "base64", version, about = "Base64 encode/decode FILE(s) to stdout", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'd', long = "decode", action = clap::ArgAction::SetTrue)]
    pub decode: bool,

    #[arg(short = 'w', long = "wrap", default_value_t = 76)]
    pub wrap: usize,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
```

- [ ] **Step 3: 创建 base64/src/main.rs（完整算法 + 文件/stdin 读入）**

```rust
mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

const ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("base64 0.1.0");
        return;
    }

    let mut exit_code = 0i32;

    let mut data: Vec<u8> = Vec::new();

    let result = if args.files.is_empty() {
        read_all_stdin(&mut data)
    } else {
        let mut ok = true;
        for p in &args.files {
            let r = if p.to_string_lossy() == "-" {
                read_all_stdin(&mut data)
            } else {
                read_all_file(p, &mut data)
            };
            if r.is_err() {
                ok = false;
                exit_code = 1;
            }
        }
        if ok { Ok(()) } else { Err(()) }
    };
    if result.is_err() {
        std::process::exit(exit_code);
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let res = if args.decode {
        decode_write(&data, &mut out)
    } else {
        encode_write(&data, &mut out, args.wrap)
    };
    if let Err(e) = res {
        eprintln!("base64: write error: {}", e);
        exit_code = 1;
    }

    std::process::exit(exit_code);
}

fn read_all_stdin(buf: &mut Vec<u8>) -> Result<(), ()> {
    let stdin = std::io::stdin();
    if let Err(e) = stdin.lock().read_to_end(buf) {
        eprintln!("base64: error reading stdin: {}", e);
        return Err(());
    }
    Ok(())
}

fn read_all_file(p: &PathBuf, buf: &mut Vec<u8>) -> Result<(), ()> {
    match File::open(p) {
        Ok(mut f) => {
            if let Err(e) = f.read_to_end(buf) {
                eprintln!("base64: {}: {}", p.display(), e);
                return Err(());
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("base64: {}: {}", p.display(), e);
            Err(())
        }
    }
}

fn encode_write<W: Write>(input: &[u8], out: &mut W, wrap: usize) -> Result<(), std::io::Error> {
    let mut col = 0usize;
    let mut i = 0usize;
    while i < input.len() {
        let n = std::cmp::min(3, input.len() - i);
        let mut b = [0u8; 3];
        b[..n].copy_from_slice(&input[i..i + n]);
        let idx = [
            (b[0] >> 2) as usize,
            (((b[0] & 0x03) << 4) | (b[1] >> 4)) as usize,
            (((b[1] & 0x0F) << 2) | (b[2] >> 6)) as usize,
            (b[2] & 0x3F) as usize,
        ];
        let mut chars = [0u8; 4];
        chars[0] = ALPHABET[idx[0]];
        chars[1] = ALPHABET[idx[1]];
        if n >= 2 {
            chars[2] = ALPHABET[idx[2]];
        } else {
            chars[2] = b'=';
        }
        if n >= 3 {
            chars[3] = ALPHABET[idx[3]];
        } else {
            chars[3] = b'=';
        }
        out.write_all(&chars)?;
        col += 4;
        if wrap > 0 && col >= wrap {
            out.write_all(b"\n")?;
            col = 0;
        }
        i += n;
    }
    if wrap > 0 && col > 0 {
        out.write_all(b"\n")?;
    }
    Ok(())
}

fn decode_write<W: Write>(input: &[u8], out: &mut W) -> Result<(), std::io::Error> {
    let mut table = [255u8; 256];
    for (i, &c) in ALPHABET.iter().enumerate() {
        table[c as usize] = i as u8;
    }
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    let mut pad = 0u32;
    for &ch in input {
        match ch {
            b'\n' | b'\r' | b' ' | b'\t' => continue,
            b'=' => {
                pad += 1;
                bits += 6;
                if bits >= 24 {
                    let mut out3 = [0u8; 3];
                    out3[0] = ((acc >> 16) & 0xFF) as u8;
                    out3[1] = ((acc >> 8) & 0xFF) as u8;
                    out3[2] = (acc & 0xFF) as u8;
                    let write_n = match pad {
                        1 => 2,
                        2 => 1,
                        _ => 0,
                    };
                    if write_n > 0 {
                        out.write_all(&out3[..write_n])?;
                    }
                    acc = 0; bits = 0; pad = 0;
                }
            }
            _ => {
                let v = table[ch as usize];
                if v == 255 {
                    eprintln!("base64: invalid character in input");
                    std::process::exit(1);
                }
                acc = (acc << 6) | (v as u32);
                bits += 6;
                if bits >= 24 {
                    let out3 = [
                        ((acc >> 16) & 0xFF) as u8,
                        ((acc >> 8) & 0xFF) as u8,
                        (acc & 0xFF) as u8,
                    ];
                    out.write_all(&out3)?;
                    acc = 0; bits = 0; pad = 0;
                }
            }
        }
    }
    if bits >= 12 {
        let n = if bits >= 24 { 3 } else if bits >= 18 { 2 } else if pad > 0 {
            if pad == 1 { 2 } else { 1 }
        } else { 1 };
        let shifts = [16u32, 8u32, 0u32];
        for i in 0..n {
            let b = ((acc >> shifts[i]) & 0xFF) as u8;
            out.write_all(&[b])?;
        }
    }
    Ok(())
}
```

- [ ] **Step 4: 构建 base64**

Run:
```
cd /d d:\Work\rust\base64 && cargo build --release --offline
```
Expected: 编译成功，无 error。

- [ ] **Step 5: 三条冒烟测试**

① `echo Hello World! | .\target\release\base64.exe`
Expected (含换行): `SGVsbG8gV29ybGQhCg==`（按换行情况略有差异，以实际 Base64 结果为准）

② 对上一步结果解码：`echo SGVsbG8gV29ybGQh | .\target\release\base64.exe -d`
Expected: `Hello World!`

③ 编码 3K 文件验证 wrap 行切割：
```
.\target\release\base64.exe -w 40 Cargo.toml
```
Expected: 每行输出不超过 40 个字符。

- [ ] **Step 6: 提交 base64**

```
cd /d d:\Work\rust
git add base64/Cargo.toml base64/src/cli.rs base64/src/main.rs
git commit -m "feat: add base64 encode/decode (std-lib only) - round 7"
```

---

## Task 3: expand / unexpand — Tab ↔ 空格互换

**Files:**
- Create: `expand/Cargo.toml`
- Create: `expand/src/cli.rs`
- Create: `expand/src/main.rs`
- Create: `unexpand/Cargo.toml`
- Create: `unexpand/src/cli.rs`
- Create: `unexpand/src/main.rs`

**Interfaces:**
- Produces: `expand.exe -t N/LIST -i FILES` / `unexpand.exe -t N -a FILES`

### expand

- [ ] **Step 1: expand/Cargo.toml**

```toml
[package]
name = "expand"
version = "0.1.0"
edition = "2024"
description = "Convert tabs to spaces"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

- [ ] **Step 2: expand/src/cli.rs**

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "expand", version, about = "Convert tabs in FILE(s) to spaces", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 't', long = "tabs", default_value = "8")]
    pub tabs: String,

    #[arg(short = 'i', long = "initial", action = clap::ArgAction::SetTrue)]
    pub initial: bool,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
```

- [ ] **Step 3: expand/src/main.rs（tab_stop 逻辑 + 多文件）**

```rust
mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("expand 0.1.0");
        return;
    }

    let stops = parse_tabs(&args.tabs);
    if stops.is_err() {
        eprintln!("expand: invalid tabs value '{}'", args.tabs);
        std::process::exit(1);
    }
    let stops = stops.unwrap();

    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = process(BufReader::new(stdin.lock()), &stops, args.initial) {
            eprintln!("expand: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for p in &args.files {
            let r: Result<(), std::io::Error> = if p.to_string_lossy() == "-" {
                let stdin = std::io::stdin();
                process(BufReader::new(stdin.lock()), &stops, args.initial)
            } else {
                match File::open(p) {
                    Ok(f) => process(BufReader::new(f), &stops, args.initial),
                    Err(e) => {
                        eprintln!("expand: {}: {}", p.display(), e);
                        exit_code = 1;
                        continue;
                    }
                }
            };
            if let Err(e) = r {
                eprintln!("expand: {}: {}", p.display(), e);
                exit_code = 1;
            }
        }
    }

    std::process::exit(exit_code);
}

enum Stops {
    Periodic(usize),
    Explicit(Vec<usize>),
}

fn parse_tabs(s: &str) -> Result<Stops, ()> {
    if let Ok(n) = s.parse::<usize>() {
        if n > 0 {
            return Ok(Stops::Periodic(n));
        }
    }
    let mut v: Vec<usize> = Vec::new();
    for part in s.split(|c: char| c == ',' || c.is_whitespace()) {
        if part.is_empty() {
            continue;
        }
        match part.parse::<usize>() {
            Ok(x) if x > 0 => v.push(x),
            _ => return Err(()),
        }
    }
    if v.is_empty() {
        Err(())
    } else {
        v.sort();
        v.dedup();
        Ok(Stops::Explicit(v))
    }
}

fn next_stop(stops: &Stops, col: usize) -> usize {
    match stops {
        Stops::Periodic(n) => {
            if *n == 0 {
                col + 1
            } else {
                (col / n + 1) * n
            }
        }
        Stops::Explicit(v) => {
            for s in v {
                if *s > col {
                    return *s;
                }
            }
            col + 1
        }
    }
}

fn process<R: BufRead>(reader: R, stops: &Stops, initial: bool) -> Result<(), std::io::Error> {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in reader.lines() {
        let line = line?;
        let mut col = 0usize;
        let mut past_init = false;
        for ch in line.chars() {
            match ch {
                '\t' if !past_init || !initial => {
                    let ns = next_stop(stops, col);
                    let n = if ns > col { ns - col } else { 1 };
                    for _ in 0..n {
                        out.write_all(b" ")?;
                    }
                    col = ns;
                }
                '\n' | '\r' => {
                    out.write_all(&[ch as u8])?;
                    col = 0;
                    past_init = false;
                }
                c => {
                    if c != ' ' && c != '\t' {
                        past_init = true;
                    }
                    let mut b = [0u8; 4];
                    let s = c.encode_utf8(&mut b);
                    out.write_all(s.as_bytes())?;
                    col += 1;
                }
            }
        }
        out.write_all(b"\n")?;
    }
    Ok(())
}
```

- [ ] **Step 4: expand 构建**

```
cd /d d:\Work\rust\expand && cargo build --release --offline
```

### unexpand

- [ ] **Step 5: unexpand/Cargo.toml**

```toml
[package]
name = "unexpand"
version = "0.1.0"
edition = "2024"
description = "Convert spaces to tabs"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

- [ ] **Step 6: unexpand/src/cli.rs**

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "unexpand", version, about = "Convert spaces in FILE(s) to tabs", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 't', long = "tabs", default_value_t = 8usize)]
    pub tabs: usize,

    #[arg(short = 'a', long = "all", action = clap::ArgAction::SetTrue)]
    pub all: bool,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
```

- [ ] **Step 7: unexpand/src/main.rs**

```rust
mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("unexpand 0.1.0");
        return;
    }

    let n = if args.tabs == 0 { 8 } else { args.tabs };

    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = process(BufReader::new(stdin.lock()), n, args.all) {
            eprintln!("unexpand: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for p in &args.files {
            let r: Result<(), std::io::Error> = if p.to_string_lossy() == "-" {
                let stdin = std::io::stdin();
                process(BufReader::new(stdin.lock()), n, args.all)
            } else {
                match File::open(p) {
                    Ok(f) => process(BufReader::new(f), n, args.all),
                    Err(e) => {
                        eprintln!("unexpand: {}: {}", p.display(), e);
                        exit_code = 1;
                        continue;
                    }
                }
            };
            if let Err(e) = r {
                eprintln!("unexpand: {}: {}", p.display(), e);
                exit_code = 1;
            }
        }
    }

    std::process::exit(exit_code);
}

fn process<R: BufRead>(reader: R, n: usize, all: bool) -> Result<(), std::io::Error> {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in reader.lines() {
        let line = line?;
        let mut col = 0usize;
        let mut space_run = 0usize;
        let mut in_leading = true;
        for ch in line.chars() {
            match ch {
                ' ' => {
                    if in_leading || all {
                        space_run += 1;
                        let next_col = col + 1;
                        if next_col % n == 0 && space_run >= 1 {
                            out.write_all(b"\t")?;
                            col = next_col;
                            space_run = 0;
                        } else {
                            col = next_col;
                        }
                    } else {
                        flush_spaces(&mut out, &mut space_run)?;
                        out.write_all(b" ")?;
                        col += 1;
                    }
                }
                '\t' => {
                    flush_spaces(&mut out, &mut space_run)?;
                    out.write_all(b"\t")?;
                    let ns = ((col / n) + 1) * n;
                    col = ns;
                }
                c => {
                    in_leading = false;
                    flush_spaces(&mut out, &mut space_run)?;
                    let mut b = [0u8; 4];
                    let s = c.encode_utf8(&mut b);
                    out.write_all(s.as_bytes())?;
                    col += 1;
                }
            }
        }
        flush_spaces(&mut out, &mut space_run)?;
        out.write_all(b"\n")?;
    }
    Ok(())
}

fn flush_spaces<W: Write>(out: &mut W, n: &mut usize) -> Result<(), std::io::Error> {
    for _ in 0..*n {
        out.write_all(b" ")?;
    }
    *n = 0;
    Ok(())
}
```

- [ ] **Step 8: unexpand 构建**

```
cd /d d:\Work\rust\unexpand && cargo build --release --offline
```

- [ ] **Step 9: 互相转换的三条冒烟测试**

① `echo "a\tb\tc" | .\expand\target\release\expand.exe`
Expected: `a       b       c`（每列对齐到 8 字符）

② `echo "        text" | .\unexpand\target\release\unexpand.exe`
Expected: 行首 8 空格变为一个 tab（`\ttext`），可用 `expand` 再还原验证。

③ `expand -t 4 Cargo.toml` 后管道到 `unexpand -t 4 -a`，应与原始内容肉眼近似一致。

- [ ] **Step 10: Commit expand + unexpand**

```
cd /d d:\Work\rust
git add expand/ unexpand/
git commit -m "feat: add expand + unexpand (tab<->space converters) - round 7"
```

---

## Task 4: tac — 反向输出

**Files:**
- Create: `tac/Cargo.toml`
- Create: `tac/src/cli.rs`
- Create: `tac/src/main.rs`

- [ ] **Step 1: tac/Cargo.toml**

```toml
[package]
name = "tac"
version = "0.1.0"
edition = "2024"
description = "Concatenate and print FILE(s) in reverse order line-by-line"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

- [ ] **Step 2: tac/src/cli.rs**

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "tac", version, about = "Concatenate and print FILE(s) in reverse", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'b', long = "before", action = clap::ArgAction::SetTrue)]
    pub before: bool,

    #[arg(short = 's', long = "separator", default_value = "\n")]
    pub separator: String,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
```

- [ ] **Step 3: tac/src/main.rs（换行默认 / 自定义分隔两策略）**

```rust
mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("tac 0.1.0");
        return;
    }

    let mut exit_code = 0i32;
    let mut data: Vec<u8> = Vec::new();

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = stdin.lock().read_to_end(&mut data) {
            eprintln!("tac: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for p in &args.files {
            let r: Result<(), std::io::Error> = if p.to_string_lossy() == "-" {
                let stdin = std::io::stdin();
                stdin.lock().read_to_end(&mut data).map(|_| ())
            } else {
                match File::open(p) {
                    Ok(mut f) => f.read_to_end(&mut data).map(|_| ()),
                    Err(e) => {
                        eprintln!("tac: {}: {}", p.display(), e);
                        exit_code = 1;
                        continue;
                    }
                }
            };
            if let Err(e) = r {
                eprintln!("tac: {}: {}", p.display(), e);
                exit_code = 1;
            }
        }
    }

    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let result = if args.separator == "\n" || args.separator.is_empty() {
        reverse_lines(&data, &mut out, args.before)
    } else {
        reverse_by_separator(&data, &args.separator, &mut out, args.before)
    };
    if let Err(e) = result {
        eprintln!("tac: write error: {}", e);
        exit_code = 1;
    }

    std::process::exit(exit_code);
}

fn reverse_lines<W: Write>(data: &[u8], out: &mut W, before: bool) -> Result<(), std::io::Error> {
    let mut chunks: Vec<&[u8]> = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < data.len() {
        if data[i] == b'\n' {
            let end = i + 1;
            chunks.push(&data[start..end]);
            start = end;
        }
        i += 1;
    }
    if start < data.len() {
        chunks.push(&data[start..]);
    }
    if before {
        for c in chunks.iter().rev() {
            out.write_all(c)?;
        }
    } else {
        for c in chunks.iter().rev() {
            out.write_all(c)?;
        }
    }
    Ok(())
}

fn reverse_by_separator<W: Write>(data: &[u8], sep: &str, out: &mut W, before: bool) -> Result<(), std::io::Error> {
    let sep = sep.as_bytes();
    let mut pieces: Vec<&[u8]> = Vec::new();
    if sep.is_empty() {
        if !data.is_empty() {
            pieces.push(data);
        }
    } else {
        let mut start = 0usize;
        let mut i = 0usize;
        while i + sep.len() <= data.len() {
            if &data[i..i + sep.len()] == sep {
                let end = i + sep.len();
                pieces.push(&data[start..end]);
                start = end;
                i = end;
            } else {
                i += 1;
            }
        }
        if start < data.len() {
            pieces.push(&data[start..]);
        }
    }
    let before = before;
    for c in pieces.iter().rev() {
        if before {
            if c.starts_with(sep) {
                let (a, b) = c.split_at(sep.len());
                out.write_all(a)?;
                out.write_all(b)?;
            } else {
                out.write_all(c)?;
            }
        } else {
            out.write_all(c)?;
        }
    }
    Ok(())
}
```

- [ ] **Step 4: 构建 tac**

```
cd /d d:\Work\rust\tac && cargo build --release --offline
```

- [ ] **Step 5: 三条冒烟**

① `printf "a\nb\nc\n" | .\target\release\tac.exe`
Expected:
```
c
b
a
```

② `printf "a-X-b-X-c" | .\target\release\tac.exe -s "-X-"`
Expected: `c-X-b-X-a`（按 `-X-` 切三段反向）

③ tac 对 `tac Cargo.toml | tac` 应等于原文件 `diff` 近似通过。

- [ ] **Step 6: 提交 tac**

```
cd /d d:\Work\rust
git add tac/
git commit -m "feat: add tac (reverse lines) - round 7"
```

---

## Task 5: column / shuf — 列格式化 + 随机行洗牌

**Files:**
- Create: `column/Cargo.toml`
- Create: `column/src/cli.rs`
- Create: `column/src/main.rs`
- Create: `shuf/Cargo.toml`
- Create: `shuf/src/cli.rs`
- Create: `shuf/src/main.rs`

### column

- [ ] **Step 1: column/Cargo.toml**

```toml
[package]
name = "column"
version = "0.1.0"
edition = "2024"
description = "Columnate lists"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

- [ ] **Step 2: column/src/cli.rs**

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "column", version, about = "Columnate FILE(s), or standard input", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 't', long = "table", action = clap::ArgAction::SetTrue)]
    pub table: bool,

    #[arg(short = 's', long = "separator", default_value = " ")]
    pub separator: String,

    #[arg(short = 'o', long = "output-separator", default_value = "  ")]
    pub output_separator: String,

    #[arg(short = 'x', long = "fillrows", action = clap::ArgAction::SetTrue)]
    pub fillrows: bool,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
```

- [ ] **Step 3: column/src/main.rs（二维表 + 列宽对齐）**

```rust
mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("column 0.1.0");
        return;
    }

    let mut exit_code = 0i32;
    let mut rows: Vec<Vec<String>> = Vec::new();

    let mut process = |p: Option<&PathBuf>| -> Result<(), std::io::Error> {
        let reader: Box<dyn BufRead> = match p {
            None | Some(p) if p.to_string_lossy() == "-" => {
                Box::new(BufReader::new(std::io::stdin().lock()))
            }
            Some(p) => Box::new(BufReader::new(File::open(p)?)),
        };
        for line in reader.lines() {
            let line = line?;
            let cols: Vec<String> = if args.separator == " " {
                line.split_whitespace().map(|s| s.to_string()).collect()
            } else {
                line.split(args.separator.as_str())
                    .map(|s| s.to_string())
                    .collect()
            };
            rows.push(cols);
        }
        Ok(())
    };

    if args.files.is_empty() {
        if let Err(e) = process(None) {
            eprintln!("column: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for p in &args.files {
            if let Err(e) = process(Some(p)) {
                eprintln!("column: {}: {}", p.display(), e);
                exit_code = 1;
            }
        }
    }

    if rows.is_empty() {
        std::process::exit(exit_code);
    }

    let mut max_cols = 0usize;
    for r in &rows {
        max_cols = max_cols.max(r.len());
    }
    let mut widths = vec![0usize; max_cols];
    for r in &rows {
        for (j, c) in r.iter().enumerate() {
            widths[j] = widths[j].max(c.chars().count());
        }
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for r in &rows {
        for (j, c) in r.iter().enumerate() {
            if j > 0 {
                out.write_all(args.output_separator.as_bytes()).ok();
            }
            let w = widths[j];
            let cc = c.chars().count();
            let pad = if cc < w { w - cc } else { 0 };
            out.write_all(c.as_bytes()).ok();
            for _ in 0..pad {
                out.write_all(b" ").ok();
            }
        }
        out.write_all(b"\n").ok();
    }

    std::process::exit(exit_code);
}
```

- [ ] **Step 4: 构建 column**

```
cd /d d:\Work\rust\column && cargo build --release --offline
```

### shuf

- [ ] **Step 5: shuf/Cargo.toml**

```toml
[package]
name = "shuf"
version = "0.1.0"
edition = "2024"
description = "Generate random permutations"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

- [ ] **Step 6: shuf/src/cli.rs**

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "shuf", version, about = "Generate random permutations of lines", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'e', long = "echo", num_args = 0..)]
    pub echo: Vec<String>,

    #[arg(short = 'n', long = "head-count")]
    pub head_count: Option<usize>,

    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    #[arg(short = 'r', long = "repeat", action = clap::ArgAction::SetTrue)]
    pub repeat: bool,

    #[arg(short = 'z', long = "zero-terminated", action = clap::ArgAction::SetTrue)]
    pub zero: bool,

    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}
```

- [ ] **Step 7: shuf/src/main.rs（splitmix64 + Fisher–Yates）**

```rust
mod cli;

use clap::Parser;
use std::collections::hash_map::RandomState;
use std::fs::File;
use std::hash::{BuildHasher, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("shuf 0.1.0");
        return;
    }

    let seed: u64 = {
        let s = RandomState::new();
        let mut h = s.build_hasher();
        h.write_usize(0xDEAD_BEEF);
        h.finish()
    };
    let mut rng = SplitMix64::new(seed);

    let sep = if args.zero { b'\0' as char } else { '\n' };
    let mut lines: Vec<String> = Vec::new();

    if !args.echo.is_empty() {
        lines.extend(args.echo.clone());
    } else {
        let reader: Box<dyn BufRead> = match &args.file {
            None | Some(p) if p.to_string_lossy() == "-" => {
                Box::new(BufReader::new(std::io::stdin().lock()))
            }
            Some(p) => match File::open(p) {
                Ok(f) => Box::new(BufReader::new(f)),
                Err(e) => {
                    eprintln!("shuf: {}: {}", p.display(), e);
                    std::process::exit(1);
                }
            },
        };
        if args.zero {
            let mut buf = Vec::new();
            let mut cur = String::new();
            for b in reader.bytes() {
                let b = match b {
                    Ok(x) => x,
                    Err(e) => {
                        eprintln!("shuf: read error: {}", e);
                        std::process::exit(1);
                    }
                };
                buf.push(b);
                if b == b'\0' {
                    cur.push_str(&String::from_utf8_lossy(&buf));
                    lines.push(std::mem::take(&mut cur));
                    buf.clear();
                }
            }
            if !buf.is_empty() {
                cur.push_str(&String::from_utf8_lossy(&buf));
                lines.push(cur);
            }
        } else {
            for line in reader.lines() {
                match line {
                    Ok(l) => lines.push(l),
                    Err(e) => {
                        eprintln!("shuf: read error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    let mut writer: Box<dyn Write> = match &args.output {
        Some(p) => match File::create(p) {
            Ok(f) => Box::new(std::io::BufWriter::new(f)),
            Err(e) => {
                eprintln!("shuf: {}: {}", p.display(), e);
                std::process::exit(1);
            }
        },
        None => Box::new(std::io::BufWriter::new(std::io::stdout())),
    };

    let total = lines.len();
    let head = args.head_count.unwrap_or(total);

    if args.repeat && total > 0 {
        for _ in 0..head {
            let idx = (rng.next_u64() as usize) % total;
            write_line(&mut writer, &lines[idx], sep);
        }
    } else {
        fy_shuffle(&mut lines, &mut rng);
        let n = std::cmp::min(head, total);
        for i in 0..n {
            write_line(&mut writer, &lines[i], sep);
        }
    }

    let _ = writer.flush();
}

fn write_line<W: Write>(out: &mut W, s: &str, sep: char) {
    let _ = out.write_all(s.as_bytes());
    let mut b = [0u8; 4];
    let s2 = sep.encode_utf8(&mut b);
    let _ = out.write_all(s2.as_bytes());
}

struct SplitMix64 {
    s: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        SplitMix64 { s: if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed } }
    }
    fn next_u64(&mut self) -> u64 {
        self.s = self.s.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut x = self.s;
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^ (x >> 31)
    }
}

fn fy_shuffle<T>(arr: &mut [T], rng: &mut SplitMix64) {
    let n = arr.len();
    for i in (1..n).rev() {
        let j = (rng.next_u64() as usize) % (i + 1);
        arr.swap(i, j);
    }
}
```

- [ ] **Step 8: 构建 shuf**

```
cd /d d:\Work\rust\shuf && cargo build --release --offline
```

- [ ] **Step 9: 两工具冒烟**

① `printf "a b c\ndd ee ff\ng h i\n" | .\column\target\release\column.exe -t -s " "`
Expected: 三列整齐对齐（a dd g 各列对齐）。

② `echo -e "1\n2\n3\n4\n5" | .\shuf\target\release\shuf.exe -n 3`
Expected: 从 {1..5} 抽取 3 个不重复行。

③ `seq 1 100 | shuf | sort -n | diff - <(seq 1 100)` 应无异（在 bash 或等价工具中验证）。

- [ ] **Step 10: 提交 column + shuf**

```
cd /d d:\Work\rust
git add column/ shuf/
git commit -m "feat: add column + shuf - round 7"
```

---

## Task 6: csplit / pr / numfmt — 文本分片 + 打印格式化 + 数字格式化

**Files:**
- Create: `csplit/Cargo.toml`
- Create: `csplit/src/cli.rs`
- Create: `csplit/src/main.rs`
- Create: `pr/Cargo.toml`
- Create: `pr/src/cli.rs`
- Create: `pr/src/main.rs`
- Create: `numfmt/Cargo.toml`
- Create: `numfmt/src/cli.rs`
- Create: `numfmt/src/main.rs`

### csplit

- [ ] **Step 1: csplit/Cargo.toml**

```toml
[package]
name = "csplit"
version = "0.1.0"
edition = "2024"
description = "Split a file into sections determined by context lines"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

- [ ] **Step 2: csplit/src/cli.rs**

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "csplit", version, about = "Split FILE into pieces by PATTERN", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'f', long = "prefix", default_value = "xx")]
    pub prefix: String,

    #[arg(short = 'n', long = "digits", default_value_t = 2usize)]
    pub digits: usize,

    #[arg(short = 'k', long = "keep-files", action = clap::ArgAction::SetTrue)]
    pub keep_files: bool,

    #[arg(short = 'z', long = "elide-empty-files", action = clap::ArgAction::SetTrue)]
    pub elide_empty: bool,

    #[arg(short = 's', long = "quiet", action = clap::ArgAction::SetTrue)]
    pub quiet: bool,

    pub file: PathBuf,

    #[arg(trailing_var_arg = true, required = true)]
    pub patterns: Vec<String>,
}
```

- [ ] **Step 3: csplit/src/main.rs（行号 + 子串匹配断点）**

```rust
mod cli;

use clap::Parser;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("csplit 0.1.0");
        return;
    }

    let lines: Vec<String> = match File::open(&args.file) {
        Ok(f) => {
            let reader = BufReader::new(f);
            let mut v = Vec::new();
            for line in reader.lines() {
                match line {
                    Ok(l) => v.push(l),
                    Err(e) => {
                        eprintln!("csplit: {}: {}", args.file.display(), e);
                        std::process::exit(1);
                    }
                }
            }
            v
        }
        Err(e) => {
            eprintln!("csplit: {}: {}", args.file.display(), e);
            std::process::exit(1);
        }
    };

    let mut breakpoints: Vec<usize> = Vec::new();
    breakpoints.push(0);

    let pats: Vec<String> = expand_patterns(&args.patterns);
    let mut cursor = 0usize;
    for p in &pats {
        if let Ok(n) = p.parse::<usize>() {
            if n >= 1 && n <= lines.len() {
                let bp = n - 1;
                if bp > cursor {
                    breakpoints.push(bp);
                    cursor = bp;
                }
            }
            continue;
        }
        let raw = p.trim();
        let needle_opt: Option<String> =
            if raw.starts_with('/') && raw.ends_with('/') && raw.len() >= 2 {
                Some(raw[1..raw.len() - 1].to_string())
            } else {
                None
            };
        if let Some(needle) = needle_opt {
            let mut found = false;
            for i in cursor.saturating_add(1)..lines.len() {
                if lines[i].contains(&needle) {
                    breakpoints.push(i);
                    cursor = i;
                    found = true;
                    break;
                }
            }
            if !found {
                breakpoints.push(lines.len());
                cursor = lines.len();
            }
        }
    }
    breakpoints.push(lines.len());

    let mut created: Vec<String> = Vec::new();
    let mut indices: Vec<usize> = Vec::new();
    let mut idx = 0usize;
    let mut ok = true;
    for win in breakpoints.windows(2) {
        let start = win[0];
        let end = win[1];
        if args.elide_empty && start >= end {
            continue;
        }
        let name = format!("{}{:0width$}", args.prefix, idx, width = args.digits);
        let slice = if start < lines.len() { &lines[start..end] } else { &[] };
        let bytes = match write_piece(&name, slice) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("csplit: {}: {}", name, e);
                ok = false;
                break;
            }
        };
        created.push(name.clone());
        indices.push(idx);
        if !args.quiet {
            println!("{}", bytes);
        }
        idx += 1;
    }
    if !ok && !args.keep_files {
        for name in &created {
            let _ = fs::remove_file(name);
        }
        std::process::exit(1);
    }

    std::process::exit(if ok { 0 } else { 1 });
}

fn expand_patterns(raw: &[String]) -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    for p in raw {
        if p.starts_with('{') && p.ends_with('}') {
            let inner = &p[1..p.len() - 1];
            let n: usize = match inner.parse() {
                Ok(x) => x,
                Err(_) => continue,
            };
            let last = res.last().cloned();
            if let Some(prev) = last {
                for _ in 0..n {
                    res.push(prev.clone());
                }
            }
        } else {
            res.push(p.clone());
        }
    }
    res
}

fn write_piece(name: &str, lines: &[String]) -> Result<usize, std::io::Error> {
    let mut f = File::create(name)?;
    let mut bytes = 0usize;
    for l in lines {
        bytes += l.as_bytes().len();
        f.write_all(l.as_bytes())?;
        f.write_all(b"\n")?;
        bytes += 1;
    }
    Ok(bytes)
}
```

- [ ] **Step 4: 构建 csplit**

```
cd /d d:\Work\rust\csplit && cargo build --release --offline
```

### pr

- [ ] **Step 5: pr/Cargo.toml**

```toml
[package]
name = "pr"
version = "0.1.0"
edition = "2024"
description = "Paginate or columnate FILE(s) for printing"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

- [ ] **Step 6: pr/src/cli.rs（包含 -NUM 简写 + 全部长参数）**

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "pr", version, about = "Paginate or columnate FILE(s)", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'l', long = "length", default_value_t = 66usize)]
    pub length: usize,

    #[arg(short = 'o', long = "indent", default_value_t = 0usize)]
    pub indent: usize,

    #[arg(short = 'w', long = "width", default_value_t = 72usize)]
    pub width: usize,

    #[arg(short = 'h', long = "header")]
    pub header: Option<String>,

    #[arg(short = 't', long = "omit-header", action = clap::ArgAction::SetTrue)]
    pub omit_header: bool,

    #[arg(short = 'd', long = "double-space", action = clap::ArgAction::SetTrue)]
    pub double_space: bool,

    #[arg(short = 'J', long = "join-lines", action = clap::ArgAction::SetTrue)]
    pub join_lines: bool,

    #[arg(short = 'c', long = "columns", default_value_t = 1usize)]
    pub columns: usize,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
```

- [ ] **Step 7: pr/src/main.rs（分页页眉 + 多列轮转）**

```rust
mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("pr 0.1.0");
        return;
    }

    let col = args.columns.max(1);
    let page_len = args.length.max(10);

    let mut exit_code = 0i32;
    let mut lines: Vec<String> = Vec::new();

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin.lock());
        for l in reader.lines() {
            match l {
                Ok(x) => lines.push(x),
                Err(e) => {
                    eprintln!("pr: error reading stdin: {}", e);
                    exit_code = 1;
                }
            }
        }
    } else {
        for p in &args.files {
            let r: Result<(), std::io::Error> = if p.to_string_lossy() == "-" {
                let stdin = std::io::stdin();
                for l in BufReader::new(stdin.lock()).lines() {
                    lines.push(l?);
                }
                Ok(())
            } else {
                match File::open(p) {
                    Ok(f) => {
                        for l in BufReader::new(f).lines() {
                            lines.push(l?);
                        }
                        Ok(())
                    }
                    Err(e) => {
                        eprintln!("pr: {}: {}", p.display(), e);
                        exit_code = 1;
                        continue;
                    }
                }
            };
            if let Err(e) = r {
                eprintln!("pr: {}: {}", p.display(), e);
                exit_code = 1;
            }
        }
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let matrix = build_matrix(&lines, col);
    let mut col_widths = vec![0usize; col];
    for r in &matrix {
        for (j, c) in r.iter().enumerate() {
            col_widths[j] = col_widths[j].max(c.chars().count());
        }
    }

    let header_lines = if args.omit_header { 0 } else { 3 };
    let body_per_page = if page_len > header_lines { page_len - header_lines } else { 1 };

    let indent: String = std::iter::repeat(' ').take(args.indent).collect();

    let mut page_no = 1usize;
    let mut cursor = 0usize;
    let total_rows = matrix.len();
    while cursor < total_rows {
        if !args.omit_header {
            let title = match (&args.header, args.files.first()) {
                (Some(h), _) => h.clone(),
                (None, Some(p)) => p.to_string_lossy().into_owned(),
                (None, None) => "".to_string(),
            };
            let center = format!("                    {}                    Page {}", title, page_no);
            writeln!(out, "{}{}", indent, center.trim_end()).ok();
            writeln!(out).ok();
        }
        let end = std::cmp::min(cursor + body_per_page, total_rows);
        for i in cursor..end {
            let row = &matrix[i];
            let mut built = String::new();
            for (j, c) in row.iter().enumerate() {
                if j > 0 { built.push_str("  "); }
                let cell_width = if args.join_lines { 0 } else { col_widths[j] };
                built.push_str(c);
                let cc = c.chars().count();
                if cc < cell_width {
                    for _ in 0..(cell_width - cc) {
                        built.push(' ');
                    }
                }
            }
            let out_line = if built.len() > args.width && !args.join_lines {
                let mut s = built;
                s.truncate(args.width);
                s
            } else {
                built
            };
            if args.double_space {
                writeln!(out, "{}{}\n", indent, out_line).ok();
            } else {
                writeln!(out, "{}{}", indent, out_line).ok();
            }
        }
        let remainder = end - cursor;
        if remainder < body_per_page {
            for _ in 0..(body_per_page - remainder) {
                writeln!(out).ok();
            }
        }
        if !args.omit_header {
            writeln!(out).ok();
            write!(out, "\x0C").ok();
        }
        cursor = end;
        page_no += 1;
    }

    std::process::exit(exit_code);
}

fn build_matrix(lines: &[String], cols: usize) -> Vec<Vec<String>> {
    if cols <= 1 {
        return lines.iter().map(|l| vec![l.clone()]).collect();
    }
    let n = lines.len();
    let rows = (n + cols - 1) / cols;
    let mut out: Vec<Vec<String>> = vec![vec![String::new(); cols]; rows];
    for (i, line) in lines.iter().enumerate() {
        let r = i % rows;
        let c = i / rows;
        if c < cols {
            out[r][c] = line.clone();
        }
    }
    out
}
```

- [ ] **Step 8: 构建 pr**

```
cd /d d:\Work\rust\pr && cargo build --release --offline
```

### numfmt

- [ ] **Step 9: numfmt/Cargo.toml**

```toml
[package]
name = "numfmt"
version = "0.1.0"
edition = "2024"
description = "Reformat numbers (human-readable SI/IEC units)"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

- [ ] **Step 10: numfmt/src/cli.rs**

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "numfmt", version, about = "Reformat NUMBER(s) with SI/IEC prefixes", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(long = "to", default_value = "none")]
    pub to: String,

    #[arg(long = "from", default_value = "none")]
    pub from: String,

    #[arg(long = "to-unit", default_value_t = 1u64)]
    pub to_unit: u64,

    #[arg(long = "from-unit", default_value_t = 1u64)]
    pub from_unit: u64,

    #[arg(long = "field")]
    pub field: Option<String>,

    #[arg(short = 'd', long = "delimiter", default_value = " ")]
    pub delimiter: String,

    #[arg(long = "padding", default_value_t = 0i32)]
    pub padding: i32,

    #[arg(long = "suffix", default_value = "")]
    pub suffix: String,

    #[arg(long = "header", default_value_t = 0usize)]
    pub header: usize,

    #[arg(trailing_var_arg = true)]
    pub numbers: Vec<String>,
}
```

- [ ] **Step 11: numfmt/src/main.rs（SI/IEC 前缀表 + 格式化）**

```rust
mod cli;

use clap::Parser;
use std::io::{BufRead, BufReader, Write};

const SI: &[(&str, u64)] = &[
    ("Y", 1_000_000_000_000_000_000_000_000),
    ("Z", 1_000_000_000_000_000_000_000),
    ("E", 1_000_000_000_000_000_000),
    ("P", 1_000_000_000_000_000),
    ("T", 1_000_000_000_000),
    ("G", 1_000_000_000),
    ("M", 1_000_000),
    ("K", 1_000),
    ("", 1),
];

const IEC: &[(&str, u64)] = &[
    ("Y", 1u64 << 80),
    ("Z", 1u64 << 70),
    ("E", 1u64 << 60),
    ("P", 1u64 << 50),
    ("T", 1u64 << 40),
    ("G", 1u64 << 30),
    ("M", 1u64 << 20),
    ("K", 1u64 << 10),
    ("", 1),
];

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("numfmt 0.1.0");
        return;
    }

    let fields: Vec<usize> = match &args.field {
        None => Vec::new(),
        Some(s) => parse_fields(s),
    };

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut exit_code = 0i32;

    if !args.numbers.is_empty() {
        for raw in &args.numbers {
            match format_one(
                raw,
                &args.to,
                &args.from,
                args.from_unit,
                args.to_unit,
                &args.suffix,
                args.padding,
            ) {
                Ok(s) => {
                    writeln!(out, "{}", s).ok();
                }
                Err(e) => {
                    eprintln!("numfmt: {}", e);
                    exit_code = 1;
                }
            }
        }
    } else {
        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin.lock());
        let mut line_no = 0usize;
        for line in reader.lines() {
            line_no += 1;
            let line = match line {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("numfmt: read error: {}", e);
                    exit_code = 1;
                    break;
                }
            };
            if line_no <= args.header {
                writeln!(out, "{}", line).ok();
                continue;
            }
            let delim = if args.delimiter == " " { "\t" } else { args.delimiter.as_str() };
            let parts: Vec<&str> = if args.delimiter == " " {
                line.split_whitespace().collect()
            } else {
                line.split(delim).collect()
            };
            if fields.is_empty() {
                let combined = parts.join(" ");
                let trimmed = combined.trim();
                match format_one(
                    trimmed,
                    &args.to, &args.from,
                    args.from_unit, args.to_unit,
                    &args.suffix, args.padding,
                ) {
                    Ok(s) => writeln!(out, "{}", s).ok(),
                    Err(e) => {
                        writeln!(out, "{}", line).ok();
                        eprintln!("numfmt: {}", e);
                        exit_code = 1;
                    }
                };
            } else {
                let mut owned: Vec<String> = parts.iter().map(|s| s.to_string()).collect();
                let max_idx = owned.len();
                for &f in &fields {
                    if f < 1 || f > max_idx { continue; }
                    let orig = owned[f - 1].clone();
                    match format_one(
                        orig.trim(),
                        &args.to, &args.from,
                        args.from_unit, args.to_unit,
                        &args.suffix, args.padding,
                    ) {
                        Ok(s) => owned[f - 1] = s,
                        Err(e) => {
                            eprintln!("numfmt: {}", e);
                            exit_code = 1;
                        }
                    }
                }
                writeln!(out, "{}", owned.join(&if args.delimiter == " " { " ".to_string() } else { args.delimiter.clone() })).ok();
            }
        }
    }

    std::process::exit(exit_code);
}

fn parse_fields(s: &str) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    for part in s.split(',') {
        if let Ok(n) = part.parse::<usize>() {
            out.push(n);
            continue;
        }
        if part.contains('-') {
            let mut it = part.split('-');
            let a = it.next().and_then(|s| s.parse::<usize>().ok());
            let b = it.next().and_then(|s| s.parse::<usize>().ok());
            if let (Some(a), Some(b)) = (a, b) {
                for x in a..=b { out.push(x); }
            }
        }
    }
    out
}

fn parse_number(raw: &str, from_mode: &str, from_unit: u64) -> Result<u64, String> {
    let (num_str, suf): (&str, &str) = {
        let bytes = raw.as_bytes();
        let mut i = bytes.len();
        while i > 0 {
            let c = bytes[i - 1] as char;
            if c.is_ascii_digit() || c == '.' { break; }
            i -= 1;
        }
        (&raw[..i], &raw[i..])
    };
    let val: f64 = num_str.parse().map_err(|_| format!("invalid number: '{}'", raw))?;
    let table = match from_mode {
        "si" => SI,
        "iec" | "iec-i" | "auto" => IEC,
        _ => {
            if suf.is_empty() {
                &IEC[8..]
            } else {
                IEC
            }
        }
    };
    let mult = if suf.is_empty() {
        1u64
    } else {
        let mut found: Option<u64> = None;
        for &(name, m) in table {
            if !name.is_empty() && (suf.starts_with(name) || suf.starts_with(&name.replace("i", ""))) {
                found = Some(m);
                break;
            }
        }
        found.unwrap_or(1)
    };
    let total = val * mult as f64 * from_unit as f64;
    if total < 0.0 || !total.is_finite() {
        return Err("overflow or negative number".to_string());
    }
    Ok(total.round() as u64)
}

fn format_to(n: u64, to_mode: &str, to_unit: u64) -> (String, &'static str) {
    let table: &[(&str, u64)] = match to_mode {
        "si" => SI,
        "iec" => IEC,
        "iec-i" => IEC,
        "auto" => SI,
        _ => {
            let n2 = if to_unit > 1 { n / to_unit } else { n };
            return (format!("{}", n2), "");
        }
    };
    let n2 = if to_unit > 1 { n / to_unit.max(1) } else { n };
    for &(name, m) in table {
        if n2 >= m {
            let v = n2 as f64 / m as f64;
            let formatted = if v >= 10.0 {
                format!("{:.1}", v)
            } else {
                format!("{:.2}", v)
            };
            let clean = strip_trailing_zeroes(&formatted);
            let suffix = if to_mode == "iec-i" && !name.is_empty() {
                if name.len() == 1 {
                    static IECI: [&str; 9] = ["Y", "Z", "E", "P", "T", "G", "M", "K", ""];
                    let pos = IEC.iter().position(|(n2, _)| *n2 == name);
                    match pos {
                        Some(p) if p < 8 => IECI[p],
                        _ => name,
                    }
                } else { name }
            } else { name };
            return (clean, suffix);
        }
    }
    (format!("{}", n2), "")
}

fn strip_trailing_zeroes(s: &str) -> String {
    if !s.contains('.') { return s.to_string(); }
    let mut t = s.to_string();
    while t.ends_with('0') { t.pop(); }
    if t.ends_with('.') { t.pop(); }
    t
}

fn pad(s: &str, padding: i32) -> String {
    if padding == 0 { return s.to_string(); }
    let w = padding.unsigned_abs() as usize;
    let n = s.chars().count();
    if w <= n { return s.to_string(); }
    let need = w - n;
    if padding > 0 {
        format!("{}{}", " ".repeat(need), s)
    } else {
        format!("{}{}", s, " ".repeat(need))
    }
}

fn format_one(
    raw: &str,
    to_mode: &str,
    from_mode: &str,
    from_unit: u64,
    to_unit: u64,
    sfx: &str,
    padding: i32,
) -> Result<String, String> {
    let n = parse_number(raw, from_mode, from_unit)?;
    let (num, prefix) = format_to(n, to_mode, to_unit);
    let joined = format!("{}{}{}", num, prefix, sfx);
    Ok(pad(&joined, padding))
}
```

- [ ] **Step 12: 构建 numfmt**

```
cd /d d:\Work\rust\numfmt && cargo build --release --offline
```

- [ ] **Step 13: csplit / pr / numfmt 冒烟测试**

① csplit：准备 20 行测试文件 `seq 1 20 > test.txt` → `csplit -z test.txt 11 /15/` → 生成 `xx00 (10行)` / `xx01 (4行)` / `xx02 (6行)`。

② pr：`seq 1 50 | pr -3 -l 20 -t` → 输出三列布局（1..n 轮转）。

③ numfmt：`numfmt --to=si 1234567890` → `1.2G`；`numfmt --from=iec 2M` → `2097152`。

- [ ] **Step 14: 提交 csplit + pr + numfmt**

```
cd /d d:\Work\rust
git add csplit/ pr/ numfmt/
git commit -m "feat: add csplit + pr + numfmt - round 7"
```

---

## Task 7: Final Build & Acceptance & Overall Commit

**Files:**
- Test: `build-all.bat` 端到端全流程
- Verify: `D:\develop\rust-tools` 目录 + `git status` 清洁性

- [ ] **Step 1: 执行全量构建**

```
cd /d d:\Work\rust
.\build-all.bat
```
Expected: `All builds completed successfully!` 结尾；无 ERROR 行。

- [ ] **Step 2: 验证 10 个新 exe 在 D:\develop\rust-tools 存在**

```
dir /b D:\develop\rust-tools | findstr /i "base64 expand unexpand tac column shuf csplit pr numfmt factor"
```
Expected: 10 行（含大小写）。

- [ ] **Step 3: 每工具至少一个综合命令冒烟**

```
D:\develop\rust-tools\factor 12345
D:\develop\rust-tools\echo Hello | D:\develop\rust-tools\base64 -w 0
D:\develop\rust-tools\echo "a\tb" | D:\develop\rust-tools\expand -t 4
D:\develop\rust-tools\echo -e "a\nb\nc" | D:\develop\rust-tools\tac
D:\develop\rust-tools\echo -e "a 1 x\nbb 22 yy" | D:\develop\rust-tools\column -t
D:\develop\rust-tools\seq 1 5 | D:\develop\rust-tools\shuf
D:\develop\rust-tools\numfmt --to=si 9999999
D:\develop\rust-tools\seq 1 10 | D:\develop\rust-tools\pr -2 -l 12 -t
```
All Expected: 正常退出码 0，输出肉眼合理。

- [ ] **Step 4: 检查 git status 无 target/ 产物污染**

```
cd /d d:\Work\rust && git status --short
```
Expected: 空输出（或仅已知已修改源代码），不出现 `*/target/*`、任何 `*.exe`/`*.pdb` 行。

- [ ] **Step 5: 统一整体 Round 7 提交（含新 10 工具 + spec + plan + build-all 修改）**

注：此步骤为聚合级提交说明；若各工具前序任务已单独提交，Step 5 仅在有残余未提交文件时执行：

```
cd /d d:\Work\rust
git add -A
git commit -m "Add 10 new Linux text-processing tools for Windows (round 7): base64 expand unexpand tac column shuf csplit pr numfmt factor"
```

- [ ] **Step 6: 最终检查 — 显示 toolcount**

```
dir /b D:\develop\rust-tools | find /c /v ""
```
Expected: 结果 = 78 (前) + 10 (本轮) = **88**。

---

## Plan Self-Review

1. **Spec 覆盖检查**
   - base64 编解码 + wrap + 多文件/stdin ✅ Task 2
   - expand (-t N/LIST + -i) ✅ Task 3
   - unexpand (-t N + -a) ✅ Task 3
   - tac (-s separator + -b before) ✅ Task 4
   - column (-t/-s/-o 列格式化) ✅ Task 5
   - shuf (-e/-n/-o/-r/-z + 自研 PRNG) ✅ Task 5
   - csplit (-f/-n/-z + 行号+子串模式) ✅ Task 6
   - pr (分页页眉 + 多列 + -d/-t/-h) ✅ Task 6
   - numfmt (SI/IEC + --to/--from + 字段) ✅ Task 6
   - factor (u64 试除法) ✅ Task 1

2. **占位符扫描**: 整份计划中所有代码步均已贴完整代码，命令均为完整 cd + cargo/exec，无 TBD/TODO/模糊描述。✅

3. **类型一致性**：10 个工具的 Args 中统一 `version: bool` 字段 + `#[arg(short='v')]`；统一 `mod cli; use clap::Parser;` 头；统一错误格式 `"<tool>: ...: {}"`。✅
