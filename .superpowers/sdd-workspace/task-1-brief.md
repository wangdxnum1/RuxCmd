# Task 1: factor — 质因数分解（最简单，热启动）

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
