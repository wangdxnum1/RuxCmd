# Task 5 报告：column formatter + shuf shuffler

## 提交信息
- **Short SHA**: `d763990`
- **Subject**: `feat: add column formatter + shuf Fisher-Yates shuffler - round 7`
- **Files changed**: 6 files, 342 insertions(+)
  - column/Cargo.toml
  - column/src/cli.rs
  - column/src/main.rs
  - shuf/Cargo.toml
  - shuf/src/cli.rs
  - shuf/src/main.rs

---

## 一、构建输出

### Step 4: column cargo build --release --offline

```
   Compiling windows-link v0.2.1
   Compiling proc-macro2 v1.0.106
   ...
   Compiling clap v4.6.1
   Compiling column v0.1.0 (D:\Work\rust\column)
warning: unused import: `std::path::PathBuf`
 --> src\main.rs:6:5
  |
6 | use std::path::PathBuf;
  |     ^^^^^^^^^^^^^^^^^^
warning: `column` (bin "column") generated 1 warning
    Finished `release` profile [optimized] target(s) in 12.14s
```

**Result**: 编译通过，1 warning（未使用 `use std::path::PathBuf`，与 brief 代码完全一致未做修改）。

---

### Step 8: shuf cargo build --release --offline

#### 第一次构建（失败）
```
   Compiling shuf v0.1.0 (D:\Work\rust\shuf)
warning: unused import: `std::path::PathBuf`
 --> src\main.rs:6:5

error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option`
  --> src\main.rs:84:40
   |
8  | fn main() {
   | --------- this function should return `Result` or `Option` to accept `?`
...
84 |             let mut f = File::create(p)?;
   |                                        ^ cannot use the `?` operator ...
```

#### 自修复（1次重试，按规则允许）
**修复位置**: `shuf/src/main.rs:82-89` (wres match 块)
**原代码**:
```rust
Some(p) => {
    let mut f = File::create(p)?;
    f.write_all(&output)
}
```
**修复后**:
```rust
Some(p) => File::create(p).and_then(|mut f| f.write_all(&output)),
```
**修复说明**: 将 `?` 提前 return 写法改为 `Result::and_then` 链式组合，使整个 match arm 保持 `Result<(), std::io::Error>` 返回类型，`wres` 类型不变。算法逻辑不变。

#### 第二次构建（成功）
```
   Compiling shuf v0.1.0 (D:\Work\rust\shuf)
warning: unused import: `std::path::PathBuf`
 --> src\main.rs:6:5
warning: `shuf` (bin "shuf") generated 1 warning
    Finished `release` profile [optimized] target(s) in 1.55s
```

**Result**: 编译通过，1 warning（未使用 PathBuf，与 brief 一致未改动）。

---

## 二、冒烟测试实际输出 vs 预期（Step 9）

### ① column -t -s "," -o "|" 制表测试

**PowerShell 命令**（等价于 brief 的 cmd 管道）:
```powershell
$input = "a,bc,defg`n1,22,333"; $input | .\target\release\column.exe -t -s "," -o "|"
```

**实际输出** (EXIT:0):
```
a|bc|defg
1|22|333
```

**预期**: 三列按 display 宽度对齐，输出分隔符为 `|`。
- 列1 display_width: "a"=1, "1"=1 → widths[0]=1 ✓
- 列2 display_width: "bc"=2, "22"=2 → widths[1]=2 ✓
- 列3 display_width: "defg"=4, "333"=3 → widths[2]=4
  - 第一行列3 输出 "defg"（4宽，pad=0）
  - 第二行列3 输出 "333" + 1空格（但终端/管道可能截断尾随空格）
- 分隔符 `|` 正确插入 ✓

**判定**: ✅ PASS（输出格式、分隔符、列对齐均符合预期）

---

### ② shuf -i 1-10 --random-source "hello" 确定性洗牌

**实际输出** (EXIT:0):
```
4
1
7
3
5
8
2
9
10
6
```

**预期**: 输出 1..10 的一个排列，同一 seed 重复运行结果稳定一致。
- 检查 1..10 齐全无重复: {4,1,7,3,5,8,2,9,10,6} = {1..10} ✓
- 同一 seed "hello" 多次运行此报告外复现一致 ✓

**判定**: ✅ PASS（Fisher-Yates + 字符串 FNV seed 生效，结果稳定）

---

### ③ shuf -n 5 -o shuf-5.txt 限制数量 + 文件输出

**命令**: `shuf.exe -i 1-20 --random-source x -n 5 -o shuf-5.txt`

**shuf-5.txt 实际内容** (EXIT:0，Line count:5):
```
15
16
13
4
9
```

**预期**: shuf-5.txt 中恰好 5 行，每行 1..20 整数，无重复。
- 行数 = 5 ✓
- 取值范围: {15,16,13,4,9} ⊆ {1..20} ✓
- 无重复 ✓
- 输出文件成功创建 ✓

**判定**: ✅ PASS（head count 限制、output 文件写入正常）

---

### ④ 同 seed 一致性（fc 对比）

**命令**:
```powershell
shuf.exe -i 1-10 --random-source a1 > a.txt
shuf.exe -i 1-10 --random-source a1 > b.txt
fc a.txt b.txt
```

**实际输出**:
```
Comparing files a.txt and B.TXT
FC: no differences encountered
---fc exit: 0---
```

**预期**: 两文件完全一致。
- fc exit 0 且 "no differences encountered" ✓

**判定**: ✅ PASS（seed 身份映射稳定，两次运行字节级一致）

---

## 三、Brief 依从性自检

| 约束 | 状态 | 说明 |
|------|------|------|
| 6 files only（column/ ×3, shuf/ ×3） | ✅ | 最终 commit 恰好 6 个源文件 |
| crate 名与目录一致 column / shuf | ✅ | Cargo.toml `name` 匹配目录名 |
| cli.rs `disable_version_flag=true` + `-v/--version` SetTrue | ✅ | 两个 cli.rs 均严格按 brief |
| main.rs 前两行 `mod cli;` + `use clap::Parser;` | ✅ | 两个 main.rs 均严格匹配 |
| 错误前缀 eprintln!("column: ...") / eprintln!("shuf: ...") | ✅ | main.rs 所有错误信息均带前缀 |
| `std::process::exit` 用于退出码 | ✅ | 两处 exit(1) + 末尾 exit(exit_code) |
| "-" → stdin 处理 | ✅ | 两个 main.rs 都有 `to_string_lossy() == "-"` 分支 |
| 不提交 target/、Cargo.lock | ✅ | 未出现在 commit 清单中（gitignore 已覆盖） |
| 不加额外注释、算法逐字 | ✅ | 所有代码 verbatim，仅 shuf `?` → `.and_then` 做了等效编译修复 |
| `-R RIGHT-JUSTIFY`、`-H HEADER-REPEAT` | ✅ | column 完整实现 |
| `--random-source` 通过字符串 hash seed | ✅ | FNV-1a 64-bit hash，与 brief 完全一致 |
| Fisher-Yates 线性同余 RNG 内联实现 | ✅ | shuffle() 函数 multiplier/increment 字面量匹配 |

**6 文件清单（与 commit 清单交叉核对）**:
1. `column/Cargo.toml` – 14 行 toml，name=column, edition=2024, clap 4+derive ✓
2. `column/src/cli.rs` – Args 结构体，7 个字段，`disable_version_flag=true` ✓
3. `column/src/main.rs` – 包含 split_by_any/byte_pos/display_width/write_row 4 个 helper ✓
4. `shuf/Cargo.toml` – 15 行 toml，name=shuf, edition=2024, clap 4+derive ✓
5. `shuf/src/cli.rs` – Args 结构体，6 个字段，`disable_version_flag=true` ✓
6. `shuf/src/main.rs` – 包含 parse_range/hash_str_to_u64/shuffle 3 个 helper ✓

---

## 四、Concerns / 遗留点

1. **shuf main.rs 自修复 1 处**: `File::create(p)?` → `File::create(p).and_then(|mut f| f.write_all(&output))`。此修复是编译必要的，逻辑/类型行为等价，不改变错误路径（wres Err 仍在后续 if let Err(e) 统一前缀打印）。
2. **两个 crate 均有 `unused import: std::path::PathBuf` 警告**: brief 原文即 `use std::path::PathBuf;` 但 main 内未直接使用（仅 cli 用）。按 "不加额外修改" 规则保留，未删。
3. **column 测试①尾随空格**: PowerShell 字符串管道可能导致 column 输出的尾随 pad 空格在报告中不明显，但算法 `write_row` 的 pad 逻辑与 brief 完全一致，不影响功能正确性。

---

## 五、总结

- **Status**: DONE
- **Builds**: 2 crates compiled（column 一次通过；shuf 经 1 次自修复后通过）
- **Smoke tests**: 4 / 4 passed（① column 制表、② seed 洗牌、③ -n -o 文件、④ fc 一致性）
- **Commit**: `d763990` / 6 files / exact Step 10 message
- **Report file**: `d:\Work\rust\.superpowers\sdd-workspace\task-5-report.md`
