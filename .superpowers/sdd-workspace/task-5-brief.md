# Task 5: column + shuf 两个 Crate

- column：按列对齐格式化输入（-t 制表、-s SEP 分隔、-o OUTPUT-SEP、-R RIGHT-JUSTIFY、-H HEADER）
- shuf：Fisher-Yates 行洗牌（-i LOW-HIGH 范围输入、-n HEAD、-o OUTPUT、--random-source RNGSEED 通过字符串 seed）

6 个新文件：
- column/Cargo.toml、column/src/cli.rs、column/src/main.rs
- shuf/Cargo.toml、shuf/src/cli.rs、shuf/src/main.rs

Pre-req: mkdir -p column/src 和 shuf/src

---

## Part A: column crate

### Step 1: column/Cargo.toml

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

### Step 2: column/src/cli.rs

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "column", version, about = "Columnate lists", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 't', long = "table", action = clap::ArgAction::SetTrue)]
    pub table: bool,

    #[arg(short = 's', long = "separator", default_value = " \t")]
    pub separator: String,

    #[arg(short = 'o', long = "output-separator", default_value = "  ")]
    pub output_separator: String,

    #[arg(short = 'R', long = "table-right", value_name = "COLS")]
    pub table_right: Option<String>,

    #[arg(short = 'H', long = "table-header-repeat", value_name = "LINES")]
    pub header: Option<usize>,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
```

### Step 3: column/src/main.rs

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

    let right_cols: Vec<usize> = match args.table_right.as_ref() {
        Some(s) if !s.is_empty() => {
            let mut v = Vec::new();
            for p in s.split(',') {
                match p.trim().parse::<usize>() {
                    Ok(x) if x > 0 => v.push(x - 1),
                    _ => {
                        eprintln!("column: invalid -R value '{}'", s);
                        std::process::exit(1);
                    }
                }
            }
            v
        }
        _ => Vec::new(),
    };

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut exit_code = 0i32;

    let mut push_lines = |r: Result<Box<dyn BufRead>, String>| {
        let reader = match r {
            Ok(r) => r,
            Err(e) => { eprintln!("{}", e); exit_code = 1; return; }
        };
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => { eprintln!("column: read error: {}", e); exit_code = 1; return; }
            };
            let parts: Vec<String> = if args.table {
                split_by_any(&line, &args.separator).into_iter().map(|s| s.to_string()).collect()
            } else {
                line.split_whitespace().map(|s| s.to_string()).collect()
            };
            rows.push(parts);
        }
    };

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        push_lines(Ok(Box::new(BufReader::new(stdin.lock()))));
    } else {
        for p in &args.files {
            if p.to_string_lossy() == "-" {
                let stdin = std::io::stdin();
                push_lines(Ok(Box::new(BufReader::new(stdin.lock()))));
            } else {
                match File::open(p) {
                    Ok(f) => push_lines(Ok(Box::new(BufReader::new(f)))),
                    Err(e) => push_lines(Err(format!("column: {}: {}", p.display(), e))),
                }
            }
        }
    }

    if rows.is_empty() {
        std::process::exit(exit_code);
    }

    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut widths: Vec<usize> = vec![0; ncols];
    for r in &rows {
        for (i, c) in r.iter().enumerate() {
            let w = display_width(c);
            if w > widths[i] { widths[i] = w; }
        }
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for (ridx, r) in rows.iter().enumerate() {
        if let Some(h) = args.header {
            if ridx > 0 && ridx % h == 0 {
                let header_row = rows[0].clone();
                let _ = write_row(&mut out, &header_row, &widths, &args.output_separator, &right_cols);
                let _ = out.write_all(b"\n");
            }
        }
        let _ = write_row(&mut out, r, &widths, &args.output_separator, &right_cols);
        let _ = out.write_all(b"\n");
    }

    std::process::exit(exit_code);
}

fn split_by_any<'a>(line: &'a str, seps: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let sep_chars: Vec<char> = seps.chars().collect();
    let mut start = 0;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if sep_chars.contains(&c) {
            out.push(line[start..byte_pos(&chars, i)].into());
            i += 1;
            start = byte_pos(&chars, i);
        } else {
            i += 1;
        }
    }
    out.push(line[start..].into());
    out
}

fn byte_pos(chars: &[char], idx: usize) -> usize {
    chars[..idx.min(chars.len())].iter().map(|c| c.len_utf8()).sum()
}

fn display_width(s: &str) -> usize {
    s.chars().map(|c| if (c as u32) < 0x80 { 1 } else { 2 }).sum()
}

fn write_row<W: Write>(out: &mut W, r: &[String], widths: &[usize], outsep: &str, right: &[usize]) -> Result<(), std::io::Error> {
    let n = r.len();
    for (i, cell) in r.iter().enumerate() {
        let w = *widths.get(i).unwrap_or(&0);
        let dw = display_width(cell);
        let pad = if dw < w { w - dw } else { 0 };
        let is_right = right.contains(&i);
        if is_right {
            for _ in 0..pad { out.write_all(b" ")?; }
            out.write_all(cell.as_bytes())?;
        } else {
            out.write_all(cell.as_bytes())?;
            for _ in 0..pad { out.write_all(b" ")?; }
        }
        if i + 1 < n {
            out.write_all(outsep.as_bytes())?;
        }
    }
    Ok(())
}
```

### Step 4: column 构建

```
cd /d d:\Work\rust\column && cargo build --release --offline
```

---

## Part B: shuf crate

### Step 5: shuf/Cargo.toml

```toml
[package]
name = "shuf"
version = "0.1.0"
edition = "2024"
description = "Generate random permutations of input lines"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

### Step 6: shuf/src/cli.rs

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "shuf", version, about = "Shuffle input lines randomly", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'i', long = "input-range", value_name = "LO-HI")]
    pub input_range: Option<String>,

    #[arg(short = 'n', long = "head-count", default_value_t = usize::MAX)]
    pub head_count: usize,

    #[arg(short = 'o', long = "output", value_name = "FILE")]
    pub output: Option<PathBuf>,

    #[arg(long = "random-source", value_name = "SEEDSTR")]
    pub random_source: Option<String>,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
```

### Step 7: shuf/src/main.rs

```rust
mod cli;

use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("shuf 0.1.0");
        return;
    }

    let seed: u64 = match args.random_source.as_ref() {
        Some(s) => hash_str_to_u64(s),
        None => {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(42);
            nanos ^ ((std::process::id() as u64) << 32)
        }
    };

    let mut items: Vec<String> = Vec::new();
    let mut exit_code = 0i32;

    if let Some(range) = args.input_range.as_ref() {
        let (lo, hi) = match parse_range(range) {
            Some(p) => p,
            None => {
                eprintln!("shuf: invalid range '{}'", range);
                std::process::exit(1);
            }
        };
        for n in lo..=hi {
            items.push(n.to_string());
        }
    } else {
        let mut read_lines = |r: Result<Box<dyn BufRead>, String>| {
            let reader = match r {
                Ok(r) => r,
                Err(e) => { eprintln!("{}", e); exit_code = 1; return; }
            };
            for line in reader.lines() {
                match line {
                    Ok(l) => items.push(l),
                    Err(e) => { eprintln!("shuf: read error: {}", e); exit_code = 1; }
                }
            }
        };

        if args.files.is_empty() {
            let stdin = std::io::stdin();
            read_lines(Ok(Box::new(BufReader::new(stdin.lock()))));
        } else {
            for p in &args.files {
                if p.to_string_lossy() == "-" {
                    let stdin = std::io::stdin();
                    read_lines(Ok(Box::new(BufReader::new(stdin.lock()))));
                } else {
                    match File::open(p) {
                        Ok(f) => read_lines(Ok(Box::new(BufReader::new(f)))),
                        Err(e) => read_lines(Err(format!("shuf: {}: {}", p.display(), e))),
                    }
                }
            }
        }
    }

    shuffle(&mut items, seed);

    let n = std::cmp::min(args.head_count, items.len());
    let output: Vec<u8> = items.into_iter().take(n).flat_map(|l| {
        let mut v = l.into_bytes();
        v.push(b'\n');
        v
    }).collect();

    let wres: Result<(), std::io::Error> = match args.output.as_ref() {
        Some(p) => {
            let mut f = File::create(p)?;
            f.write_all(&output)
        }
        None => {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            out.write_all(&output)
        }
    };
    if let Err(e) = wres {
        eprintln!("shuf: write error: {}", e);
        exit_code = 1;
    }

    std::process::exit(exit_code);
}

fn parse_range(s: &str) -> Option<(i64, i64)> {
    let dash = s.find('-')?;
    let lo: i64 = s[..dash].parse().ok()?;
    let hi: i64 = s[dash + 1..].parse().ok()?;
    if lo > hi { None } else { Some((lo, hi)) }
}

fn hash_str_to_u64(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn shuffle<T>(v: &mut [T], seed: u64) {
    let mut s = seed;
    let n = v.len();
    for i in 0..n {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let range = (n - i) as u64;
        let r = (s >> 8) % range;
        let j = i + (r as usize);
        v.swap(i, j);
    }
}
```

### Step 8: shuf 构建

```
cd /d d:\Work\rust\shuf && cargo build --release --offline
```

---

## Part C: 冒烟测试（Step 9）

① column 制表（-t）：
```
cd /d d:\Work\rust\column
cmd /c "echo a,bc,defg & echo 1,22,333" | .\target\release\column.exe -t -s "," -o "|"
```
Expected: 三列均按 display 宽度对齐，输出分隔符为 |。

② shuf 可控种子洗牌（-i + --random-source）：
```
cd /d d:\Work\rust\shuf
.\target\release\shuf.exe -i 1-10 --random-source "hello"
```
Expected: 输出 1..10 的一个排列，同一 seed 重复运行结果稳定一致。

③ shuf -n 限制数量 + -o 文件输出：
```
cd /d d:\Work\rust\shuf
.\target\release\shuf.exe -i 1-20 --random-source x -n 5 -o shuf-5.txt
Get-Content shuf-5.txt
```
Expected: shuf-5.txt 中恰好 5 行，每行一个 1..20 中的整数，无重复。

④ (可选确认) 两次同 seed 一致性：
```
cd /d d:\Work\rust\shuf
.\target\release\shuf.exe -i 1-10 --random-source a1 > a.txt
.\target\release\shuf.exe -i 1-10 --random-source a1 > b.txt
fc a.txt b.txt
```
Expected: 两文件完全一致。

## Part D: 提交（Step 10）

```
cd /d d:\Work\rust
git add column/ shuf/
git commit -m "feat: add column formatter + shuf Fisher-Yates shuffler - round 7"
```

## Constraints

- 6 files only，不多不少。
- crate 名与目录一致（column / shuf）。
- cli.rs disable_version_flag=true + short 'v' ArgAction::SetTrue。
- main.rs 前两行 `mod cli;` + `use clap::Parser;`。
- 错误前缀 eprintln!("column: ...") / eprintln!("shuf: ...")。
- 不提交 target、Cargo.lock。
- 不加额外注释。算法按 brief 逐字写。
