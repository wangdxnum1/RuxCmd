# Task 4: tac — 反向行输出（Pre-flight: 已修复 plan 中 -b 分支重复逻辑）

**本 Task Brief 已修正原始 plan 中的 Pre-flight Conflict #1：**
在 process() 输出阶段，`args.before == true` 时"分隔符在前，行在后"；而 `args.before == false` 时原本与 true 分支完全一致的重复代码块——此处已改为"行在前，分隔符在后"的正确写法。详见 Step 3 代码。

3 个新文件：

- tac/Cargo.toml
- tac/src/cli.rs
- tac/src/main.rs

Pre-req: mkdir tac/src

## Step 1: tac/Cargo.toml

```toml
[package]
name = "tac"
version = "0.1.0"
edition = "2024"
description = "Concatenate and print files in reverse line order"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

## Step 2: tac/src/cli.rs

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "tac", version, about = "Write FILE(s) to stdout, last line first", disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    #[arg(short = 'b', long = "before", action = clap::ArgAction::SetTrue)]
    pub before: bool,

    #[arg(short = 's', long = "separator", default_value_t = String::from("\n"))]
    pub separator: String,

    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
```

## Step 3: tac/src/main.rs（含 -b 重复逻辑修复）

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

    let sep: Vec<char> = if args.separator.is_empty() {
        vec!['\n']
    } else {
        args.separator.chars().collect()
    };

    let mut lines: Vec<Vec<char>> = Vec::new();
    let mut exit_code = 0i32;

    if args.files.is_empty() {
        let stdin = std::io::stdin();
        if let Err(e) = read_file_reader(stdin.lock(), &sep, &mut lines) {
            eprintln!("tac: error reading stdin: {}", e);
            exit_code = 1;
        }
    } else {
        for p in &args.files {
            if p.to_string_lossy() == "-" {
                let stdin = std::io::stdin();
                if let Err(e) = read_file_reader(stdin.lock(), &sep, &mut lines) {
                    eprintln!("tac: error reading stdin: {}", e);
                    exit_code = 1;
                }
            } else {
                match File::open(p) {
                    Ok(f) => {
                        if let Err(e) = read_file_reader(f, &sep, &mut lines) {
                            eprintln!("tac: {}: {}", p.display(), e);
                            exit_code = 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("tac: {}: {}", p.display(), e);
                        exit_code = 1;
                    }
                }
            }
        }
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let sep_bytes: Vec<u8> = sep.iter().flat_map(|c| {
        let mut b = [0u8; 4];
        c.encode_utf8(&mut b).bytes().collect::<Vec<u8>>()
    }).collect();

    lines.reverse();

    for line in &lines {
        let line_bytes: Vec<u8> = line.iter().flat_map(|c| {
            let mut b = [0u8; 4];
            c.encode_utf8(&mut b).bytes().collect::<Vec<u8>>()
        }).collect();

        if args.before {
            // separator is BEFORE current line's body
            let _ = out.write_all(&sep_bytes);
            let _ = out.write_all(&line_bytes);
        } else {
            // separator is AFTER current line's body (FIXED: was duplicate BEFORE)
            let _ = out.write_all(&line_bytes);
            let _ = out.write_all(&sep_bytes);
        }
    }

    std::process::exit(exit_code);
}

fn read_file_reader<R: Read>(reader: R, sep: &[char], lines: &mut Vec<Vec<char>>) -> Result<(), std::io::Error> {
    let mut buf = String::new();
    let mut r = reader;
    r.read_to_string(&mut buf)?;
    let chars: Vec<char> = buf.chars().collect();
    let sep_len = sep.len();

    let mut start = 0usize;
    let mut i = 0usize;
    while i + sep_len <= chars.len() {
        if chars[i..i + sep_len] == sep[..] {
            lines.push(chars[start..i].to_vec());
            i += sep_len;
            start = i;
        } else {
            i += 1;
        }
    }
    if start <= chars.len() {
        lines.push(chars[start..].to_vec());
    }
    Ok(())
}
```

## Step 4: 构建 tac

```
cd /d d:\Work\rust\tac && cargo build --release --offline
```

Expected: release exe 生成成功。

## Step 5: 3 条冒烟测试

① 基本反向输出：
```
cd /d d:\Work\rust
cmd /c "echo 1& echo 2& echo 3" | .\tac\target\release\tac.exe
```
Expected: 3, 2, 1 顺序（每行为 1/2/3 的倒序）。

② -b before 分隔符位置（自定义分隔符=逗号）：
```
cd /d d:\Work\rust
cmd /c "set /p=" < nul & echo "a,b,c" | .\tac\target\release\tac.exe -s "," -b
```
Expected: 输出中每条 record 的逗号在 record 内容之前（例如 `,c,ba` 语义）——按具体输入验证。

③ 多文件输入 + 默认分隔符 \n：
```
cd /d d:\Work\rust
.\tac\target\release\tac.exe expand\Cargo.toml unexpand\Cargo.toml
```
Expected: 先输出 unexpand 的行倒序，再输出 expand 的行倒序（注意 lines 先按顺序收集再整体 reverse，所以先所有文件的 lines 被整体 reverse）——验证输出正确。

## Step 6: 提交

```
cd /d d:\Work\rust
git add tac/Cargo.toml tac/src/cli.rs tac/src/main.rs
git commit -m "feat: add tac reverse-line printer with -b/-s flags (preflight -b fix) - round 7"
```

## Constraints

- 3 个文件只此三个，不多不少。
- main.rs 前两行 `mod cli;` + `use clap::Parser;`。
- 错误前缀 `eprintln!("tac: ...")`。
- 不要加注释。算法完全按 Step 3，特别是 -b 的 false 分支务必是 line_bytes 先写再写 sep_bytes（Pre-flight Fix）。
- 不提交 target、Cargo.lock。
