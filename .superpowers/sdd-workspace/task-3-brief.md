# Task 3: expand + unexpand — Tab ↔ 空格双向转换（2 crates）

本任务实现两个相关 crate：expand（Tab → 空格）和 unexpand（空格 → Tab）。共 6 个新文件：

**Files to create:**
- expand/Cargo.toml
- expand/src/cli.rs
- expand/src/main.rs
- unexpand/Cargo.toml
- unexpand/src/cli.rs
- unexpand/src/main.rs

**Pre-req:** 先 mkdir -p expand/src 和 unexpand/src。

---

## Part A: expand crate（Tab → 空格）

### Step 1: expand/Cargo.toml

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

### Step 2: expand/src/cli.rs

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

### Step 3: expand/src/main.rs

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

### Step 4: expand 构建

```
cd /d d:\Work\rust\expand && cargo build --release --offline
```

Expected: 无 error；生成 target\release\expand.exe。

---

## Part B: unexpand crate（空格 → Tab）

### Step 5: unexpand/Cargo.toml

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

### Step 6: unexpand/src/cli.rs

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

### Step 7: unexpand/src/main.rs

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

### Step 8: unexpand 构建

```
cd /d d:\Work\rust\unexpand && cargo build --release --offline
```

---

## Part C: 冒烟测试（Step 9：3 条联合验证）

① Tab→空格验证（expand -t 8）：
```
cd /d d:\Work\rust\expand
cmd /c "echo a<TAB>b<TAB>c | .\target\release\expand.exe -t 8"
```
Expected: 输出中 a 与 b 之间、b 与 c 之间有空格，列对齐到 8 字符。

② 空格→Tab（行首）验证：
```
echo "        text" | d:\Work\rust\unexpand\target\release\unexpand.exe
```
Expected: 8 空格被替换为 1 个 \t，可管道给 expand.exe -t 8 还原验证。

③ 回环一致性：
```
expand -t 4 d:\Work\rust\base64\Cargo.toml | unexpand -t 4 -a | expand -t 4 | diff - d:\Work\rust\base64\Cargo.toml
```
Expected: diff 几乎为空（语义等价）。

## Part D: 提交（Step 10）

```
cd /d d:\Work\rust
git add expand/ unexpand/
git commit -m "feat: add expand + unexpand (tab<->space converters) - round 7"
```

---

## Global Constraints（重要，必须遵守）

- 3 文件 per crate 共 6 文件；Cargo.toml 包名必须与目录名完全一致（expand/unexpand）。
- cli.rs 全部使用 disable_version_flag=true；自定义 -v/--version。
- main.rs 前两行固定 `mod cli;` 紧接着 `use clap::Parser;`（空行可接受）。
- 所有错误通过 eprintln!("expand: ...") 或 eprintln!("unexpand: ...") 前缀输出；退出用 std::process::exit。
- 不添加任何中文注释或多余注释。
- -" 文件名表示 stdin。
- 不提交 target/ 和 Cargo.lock（已被 .gitignore）。
- 代码完全按 brief 写，不要自作主张改算法。
