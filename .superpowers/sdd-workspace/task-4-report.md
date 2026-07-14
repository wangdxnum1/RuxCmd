# Task 4 Report: tac (倒序行输出)

## Commit

- **SHA**: `aa478ff`
- **Subject**: `feat: add tac reverse-line printer with -b/-s flags (preflight -b fix) - round 7`
- **Files**: 3 files changed, 135 insertions(+)
  - `tac/Cargo.toml` (new)
  - `tac/src/cli.rs` (new)
  - `tac/src/main.rs` (new)

---

## Step 4: Build

### Command
```
cd d:\Work\rust\tac && cargo build --release --offline
```

### Result: ✅ SUCCESS (1 warning, no errors)

```
   Compiling proc-macro2 v1.0.106
   Compiling windows-link v0.2.1
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.46
   Compiling anstyle v1.0.14
   Compiling utf8parse v0.2.2
   Compiling windows-sys v0.61.2
   Compiling once_cell_polyfill v1.70.2
   Compiling anstyle-parse v1.0.0
   Compiling colorchoice v1.0.5
   Compiling is_terminal_polyfill v1.70.2
   Compiling clap_lex v1.1.0
   Compiling strsim v0.11.1
   Compiling heck v0.5.0
   Compiling anstyle-query v1.1.5
   Compiling anstyle-wincon v3.0.11
   Compiling anstream v1.0.0
   Compiling clap_builder v4.6.0
   Compiling syn v2.0.118
   Compiling clap_derive v4.6.1
   Compiling clap v4.6.1
   Compiling tac v0.1.0 (D:\Work\rust\tac)
warning: unused import: `std::path::PathBuf`
 --> src\main.rs:6:5
  |
6 | use std::path::PathBuf;
  |     ^^^^^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: `tac` (bin "tac") generated 1 warning (run `cargo fix --bin "tac" -p tac` to apply 1 suggestion)
    Finished `release` profile [optimized] target(s) in 12.67s
```

### Build Notes
- 编译**一次性通过**，无错误，未触发自我修复机制。
- 唯一警告：`main.rs:6` 中 `use std::path::PathBuf;` 未使用（`PathBuf` 仅通过 `cli::Args` 的字段间接使用）。
- 未修改 brief 代码（严格按 Step 1-3 逐字抄录要求），保留该警告。
- Release 产物：`tac\target\release\tac.exe`

---

## Step 5: 3 条冒烟测试

### 冒烟测试 ①：基本反向输出 ✅ PASS

**Command**:
```
cd d:\Work\rust
cmd /c "echo 1& echo 2& echo 3" | .\tac\target\release\tac.exe
```

**Actual Output**:
```
3
2
1
```

**Verification**:
- 输入行顺序：`1` → `2` → `3`
- tac 倒序输出：`3` → `2` → `1` ✔
- 默认分隔符 `\n` 正常工作，每行独立识别并倒序输出。

---

### 冒烟测试 ②：`-s "," -b` 自定义分隔符 + before 标志 ✅ PASS

**Command**:
```
cd d:\Work\rust
cmd /c 'echo a,b,c' | .\tac\target\release\tac.exe -s "," -b
```

**Actual Output**:
```
,c
,b,a
```

**Verification (逐字段)**:
- 输入：`a,b,c\r\n`
- 自定义分隔符 `sep = ","`，`-b` (before) = 分隔符写在每条 record **之前**
- 按 `,` 分割 records：`["a", "b", "c\r\n"]`
- 倒序后：`["c\r\n", "b", "a"]`
- 每条 record 输出格式 `sep_bytes + line_bytes` (before 分支)：
  - `,` + `c\r\n` → `,c\r\n`
  - `,` + `b` → `,b`
  - `,` + `a` → `,a`
- 合并输出：`,c\r\n,b,a`  ✔ 与实际输出一致。

---

### 冒烟测试 ③：多文件 expand/Cargo.toml + unexpand/Cargo.toml ✅ PASS

**Command**:
```
cd d:\Work\rust
.\tac\target\release\tac.exe expand\Cargo.toml unexpand\Cargo.toml
```

**expand/Cargo.toml 原始行**（共 9 行）：
```
L1: [package]
L2: name = "expand"
L3: version = "0.1.0"
L4: edition = "2024"
L5: description = "Convert tabs to spaces"
L6: authors = ["Tim"]
L7: (empty)
L8: [dependencies]
L9: clap = { version = "4", features = ["derive"] }
```

**unexpand/Cargo.toml 原始行**（共 9 行）：
```
L1: [package]
L2: name = "unexpand"
L3: version = "0.1.0"
L4: edition = "2024"
L5: description = "Convert spaces to tabs"
L6: authors = ["Tim"]
L7: (empty)
L8: [dependencies]
L9: clap = { version = "4", features = ["derive"] }
```

**收集顺序**：expand L1-L9 → unexpand L1-L9（共 18 条 lines）
**整体 reverse 后顺序**：unexpand L9 → L1 → expand L9 → L1

**Actual Output**:
```
clap = { version = "4", features = ["derive"] }
[dependencies]

authors = ["Tim"]
description = "Convert spaces to tabs"
edition = "2024"
version = "0.1.0"
name = "unexpand"
[package]
clap = { version = "4", features = ["derive"] }
[dependencies]

authors = ["Tim"]
description = "Convert tabs to spaces"
edition = "2024"
version = "0.1.0"
name = "expand"
[package]
```

**Verification**:
- 前 9 行 = unexpand 的 9 行倒序（L9→L1） ✔
- 后 9 行 = expand 的 9 行倒序（L9→L1） ✔
- 与 brief 注释"lines 先按顺序收集再整体 reverse，所以先所有文件的 lines 被整体 reverse"一致 ✔

---

## Summary

| Item | Status |
|------|--------|
| Build (Step 4) | ✅ PASS (1 warning, no fix needed) |
| Smoke ① stdin 基本倒序 | ✅ PASS |
| Smoke ② `-s "," -b` before 分隔符 | ✅ PASS |
| Smoke ③ 多文件拼接倒序 | ✅ PASS |
| Git Commit (Step 6) | ✅ `aa478ff` |

**总体 Status: DONE**

---

## Concerns

1. **警告（非错误）**：`tac/src/main.rs:6` 中 `use std::path::PathBuf;` 为未使用 import。严格遵守 brief"逐字抄录"约束，未作删除。无功能影响。
2. **Pre-flight Fix 验证**：Step 3 `-b` false 分支（line 130-132）已按修正版写入：先 `write_all(&line_bytes)` 再 `write_all(&sep_bytes)`；true 分支先写 sep 再写 line。通过冒烟 ② 的 before 行为验证正确。
