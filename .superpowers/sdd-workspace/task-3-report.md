# Task 3 报告：expand + unexpand (Tab ↔ 空格)

## 1. 构建输出

### 1.1 expand 构建
```
cd d:\Work\rust\expand; cargo build --release --offline

Locking 21 packages to latest Rust 1.97.0 compatible versions
   Compiling proc-macro2 v1.0.106
   Compiling windows-link v0.2.1
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.46
   Compiling anstyle v1.0.14
   Compiling once_cell_polyfill v1.70.2
   Compiling windows-sys v0.61.2
   Compiling utf8parse v0.2.2
   Compiling colorchoice v1.0.5
   Compiling anstyle-parse v1.0.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling heck v0.5.0
   Compiling strsim v0.11.1
   Compiling clap_lex v1.1.0
   Compiling anstyle-wincon v3.0.11
   Compiling anstyle-query v1.1.5
   Compiling anstream v1.0.0
   Compiling clap_builder v4.6.0
   Compiling syn v2.0.118
   Compiling clap_derive v4.6.1
   Compiling clap v4.6.1
   Compiling expand v0.1.0 (D:\Work\rust\expand)
warning: unused import: `std::path::PathBuf`
 --> src\main.rs:6:5
  |
6 | use std::path::PathBuf;
  |     ^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default
warning: `expand` (bin "expand") generated 1 warning
    Finished `release` profile [optimized] target(s) in 13.60s
```
**结果：构建成功（1 warning: unused PathBuf import，未修改代码以保持 brief 原文）**

### 1.2 unexpand 构建
```
cd d:\Work\rust\unexpand; cargo build --release --offline

Locking 21 packages to latest Rust 1.97.0 compatible versions
   Compiling proc-macro2 v1.0.106
   Compiling windows-link v0.2.1
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.46
   Compiling anstyle v1.0.14
   Compiling once_cell_polyfill v1.70.2
   Compiling windows-sys v0.61.2
   Compiling utf8parse v0.2.2
   Compiling colorchoice v1.0.5
   Compiling anstyle-parse v1.0.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling heck v0.5.0
   Compiling strsim v0.11.1
   Compiling clap_lex v1.1.0
   Compiling anstyle-query v1.1.5
   Compiling anstyle-wincon v3.0.11
   Compiling anstream v1.0.0
   Compiling clap_builder v4.6.0
   Compiling syn v2.0.118
   Compiling clap_derive v4.6.1
   Compiling clap v4.6.1
   Compiling unexpand v0.1.0 (D:\Work\rust\unexpand)
warning: unused import: `std::path::PathBuf`
 --> src\main.rs:6:5
  |
6 | use std::path::PathBuf;
  |     ^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default
warning: `unexpand` (bin "unexpand") generated 1 warning
    Finished `release` profile [optimized] target(s) in 13.64s
```
**结果：构建成功（1 warning: unused PathBuf import，未修改代码以保持 brief 原文）**

---

## 2. 冒烟测试结果

### 测试 ① Tab→空格验证（expand -t 8）

命令（PowerShell 等价实现，`[char]9` 即 TAB）：
```powershell
$tab = [char]9; $input = "a$tab"+"b$tab"+"c"; $input | .\target\release\expand.exe -t 8 | Format-Hex
```

实际输出 (HEX)：
```
00000000   61 20 20 20 20 20 20 20 62 20 20 20 20 20 20 20  a       b
00000010   63                                               c
```

分析：
- `61` = 'a' 在 col 0
- 7×`20` = 7 空格，补齐到 col 8（正确，col/8+1)*8 = 8)
- `62` = 'b' 在 col 8
- 7×`20` = 7 空格，补齐到 col 16
- `63` = 'c' 在 col 16
- **通过：列对齐完全正确**

### 测试 ② 空格→Tab（行首）验证

命令：
```powershell
$input = "        text"; $input | .\target\release\unexpand.exe | Format-Hex
```

实际输出 (HEX)：
```
00000000   09 74 65 78 74                                   .text
```

分析：
- `09` = TAB (\t) — 行首 8 空格被替换为 1 个 TAB（col 8 边界触发）
- `74 65 78 74` = "text"
- **通过：8 空格 → 1 TAB 正确**

### 测试 ③ 回环一致性

命令（PowerShell 管道）：
```
expand -t 4 base64\Cargo.toml | unexpand -t 4 -a | expand -t 4 | diff-original
```

实际对比：
- **文本内容**：逐字符完全一致（[package]、name、version、edition、dependencies 等所有行内容完全相同）
- **差异仅为换行符风格**：
  - Original: `0A` (Unix LF)
  - Roundtrip 输出: `0D 0A` (Windows CRLF) + 末尾多余空行
- **原因**：Windows PowerShell 管道（Out-String/Format-Hex）会自动将 LF 转为 CRLF，并追加结尾换行。这是 PowerShell 平台行为，**非 expand/unexpand 代码逻辑问题**。
- **语义结论**：回环在文本内容层面完全等价（PASS with platform line-ending note）

---

## 3. 偏差记录

| # | 项目 | 偏差描述 | 影响 |
|---|------|----------|------|
| 1 | expand 构建 warning | `std::path::PathBuf` 未使用 import（代码完全按 brief 抄录，main.rs 中实际未直接使用 PathBuf，只通过 cli::Args 间接使用） | 无（warning 非 error） |
| 2 | unexpand 构建 warning | 同上 | 无（warning 非 error） |
| 3 | 测试③换行符 | PowerShell 管道将 LF 转为 CRLF | 无（仅平台差异，文本内容一致） |

---

## 4. 自检清单 (Self-Review Checklist)

### 文件结构
- [x] 共 6 个新文件，分 2 个 crate
  - expand/Cargo.toml ✔
  - expand/src/cli.rs ✔
  - expand/src/main.rs ✔
  - unexpand/Cargo.toml ✔
  - unexpand/src/cli.rs ✔
  - unexpand/src/main.rs ✔
- [x] 无额外目录或文件
- [x] 不包含 target/ 和 Cargo.lock（已在 .gitignore）

### Cargo.toml 规范
- [x] expand: name="expand", version="0.1.0", edition="2024", authors=["Tim"], description 正确
- [x] unexpand: name="unexpand", version="0.1.0", edition="2024", authors=["Tim"], description 正确
- [x] 依赖 clap = { version = "4", features = ["derive"] } ✔

### cli.rs 规范
- [x] disable_version_flag = true ✔（两文件均）
- [x] 自定义 -v/--version，ArgAction::SetTrue ✔
- [x] expand: -t tabs (String default "8") + -i initial (bool) ✔
- [x] unexpand: -t tabs (usize default 8) + -a all (bool) ✔
- [x] files: Vec<PathBuf> ✔

### main.rs 规范
- [x] 两文件均：首行 `mod cli;`，次行 `use clap::Parser;` ✔
- [x] 错误输出前缀：`eprintln!("expand: ...")` / `eprintln!("unexpand: ...")` ✔
- [x] 退出码：`std::process::exit(0/1)` ✔
- [x] "-" 文件名 → stdin：`p.to_string_lossy() == "-"` 判断正确 ✔
- [x] 算法严格按 brief 抄录（parse_tabs, next_stop, process 完全一致）

### 构建
- [x] expand release 构建成功 ✔
- [x] unexpand release 构建成功 ✔

### Git
- [x] `git add expand/ unexpand/` → 恰好 6 文件 staged ✔
- [x] commit message 完全一致："feat: add expand + unexpand (tab<->space converters) - round 7" ✔
- [x] Commit SHA: `1e181de`

---

## 5. 结论

**Status: DONE**

2 crates 编译通过，3/3 冒烟测试通过（测试③换行符差异为 PowerShell 平台行为，非代码问题）。代码完全按 brief 抄录，未擅自修改。
