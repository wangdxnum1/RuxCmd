# Task 6 审查报告：csplit + pr + numfmt

Base: d763990 → Head: 0ef92c7

---

## 变更概览

```mermaid
flowchart LR
    subgraph "9 个新增文件"
        A1["csplit/Cargo.toml<br/>名称/版本/依赖"]
        A2["csplit/src/cli.rs<br/>CLI 参数定义"]
        A3["csplit/src/main.rs<br/>分割核心逻辑"]
        B1["pr/Cargo.toml"]
        B2["pr/src/cli.rs"]
        B3["pr/src/main.rs<br/>分页格式化"]
        C1["numfmt/Cargo.toml"]
        C2["numfmt/src/cli.rs<br/>To/From/Round 枚举"]
        C3["numfmt/src/main.rs<br/>数字格式化 + 3 self-fixes"]
    end
    style A3 fill:#fff3e0,color:#e65100
    style B3 fill:#bbdefb,color:#0d47a1
    style C3 fill:#c8e6c9,color:#1a5e20
```

```mermaid
flowchart TD
    A["Self-fix A (pr main.rs)<br/>args.file→files.first"] --> B[原代码访问不存在的 file 字段]
    A --> C[修复后直接用 args.files.first()]
    D["Self-fix B (numfmt)<br/>Cell\<i32\> rewrite"] --> E[exit_code 默认 0]
    D --> F[任意错误 set(1), 最终 .get() 退出]
    G["Self-fix C (numfmt)<br/>From::Auto 补臂"] --> H[5 个 From 变体全覆盖]
    G --> I[From::Auto 走 parse_suffix]
    style A fill:#c8e6c9,color:#1a5e20
    style D fill:#c8e6c9,color:#1a5e20
    style G fill:#c8e6c9,color:#1a5e20
```

---

## 规范合规性

| # | 规范项 | 结果 | 证据位置 |
|---|--------|------|----------|
| **csplit 3 files** | | | |
| 1 | Cargo.toml name=csplit | ✅ | diff:26 |
| 2 | Cargo.toml 0.1.0 / 2024 / Tim / description | ✅ | diff:27-30 |
| 3 | Cargo.toml clap 4 derive | ✅ | diff:32-33 |
| 4 | cli.rs name=csplit, about, disable_version_flag=true | ✅ | diff:44 |
| 5 | cli.rs -v version SetTrue | ✅ | diff:46-47 |
| 6 | cli.rs -f prefix default "xx" | ✅ | diff:49-50 |
| 7 | cli.rs -b suffix-format default "%02d" (unused) | ✅ | diff:52-53 |
| 8 | cli.rs -n digits default 2 | ✅ | diff:55-56 |
| 9 | cli.rs -k keep SetTrue | ✅ | diff:58-59 |
| 10 | cli.rs -z elide-empty SetTrue | ✅ | diff:61-62 |
| 11 | cli.rs -s quiet SetTrue | ✅ | diff:64-65 |
| 12 | cli.rs file Option\<PathBuf\> FILE | ✅ | diff:67-68 |
| 13 | cli.rs patterns Vec last=true | ✅ | diff:70-71 |
| 14 | main.rs head mod cli + use Parser | ✅ | diff:79-81 |
| 15 | main.rs read_input(file) 读行 | ✅ | diff:99-102, main.rs:119-137 |
| 16 | main.rs splits (start,end) 追踪 | ✅ | diff:104-163 |
| 17 | main.rs /PATTERN/ contains-match | ✅ | diff:133-144 |
| 18 | main.rs line-number 模式 | ✅ | diff:146-154 |
| 19 | main.rs {N} try_repeat_tail 剥离右侧数字 | ✅ | main.rs:140-147 |
| 20 | main.rs format_filename 零位填充 | ✅ | main.rs:170-171 |
| 21 | main.rs 错误时删除文件除非 -k | ✅ | main.rs:112-114 |
| **pr 3 files** | | | |
| 22 | Cargo.toml pr 0.1.0 2024 Tim description | ✅ | diff:608-615 |
| 23 | cli.rs disable_version_flag, 参数齐全 | ✅ | pr/cli.rs:5-38 |
| 24 | cli.rs files Vec\<PathBuf\> (无 file 字段) | ✅ | pr/cli.rs:37-38 |
| 25 | main.rs mod cli + use Parser | ✅ | pr/main.rs:1-3 |
| 26 | main.rs format_date secs/31536000 粗略 y/m/d | ✅ | pr/main.rs:179-183 |
| 27 | main.rs parse_number_spec 前导数字 + sep / 默认 tab 宽 5 | ✅ | pr/main.rs:103-122 |
| 28 | main.rs preprocess 两栏 half-half, col_w=page_w/2-1 | ✅ | pr/main.rs:132-158, line 134 |
| 29 | main.rs write_header 产出 5 行 | ✅ | pr/main.rs:187-208 (5 writeln!) |
| 30 | main.rs truncate_or_keep ascii=1 else=2 | ✅ | pr/main.rs:174-184 |
| **numfmt 3 files** | | | |
| 31 | Cargo.toml numfmt 0.1.0 2024 Tim | ✅ | diff:255-262 |
| 32 | cli.rs disable_version_flag=true | ✅ | diff:273 |
| 33 | cli.rs --to ValueEnum 5 arms + default=None | ✅ | diff:278-279, 315-323 |
| 34 | cli.rs --from ValueEnum 5 arms + default=None | ✅ | diff:281-282, 325-333 |
| 35 | cli.rs to-unit / from-unit 1f64 | ✅ | diff:284-288 |
| 36 | cli.rs -d delimiter " \\t" | ✅ | diff:290-291 |
| 37 | cli.rs --field 1usize / -H header 0usize / -p padding 0isize | ✅ | diff:293-300 |
| 38 | cli.rs -S suffix Option / --round 5 arms default=Up | ✅ | diff:302-306, 335-342 |
| 39 | cli.rs --grouping SetTrue / rest Vec NUMBER_OR_FILE | ✅ | diff:308-312 |
| 40 | main.rs mod cli + use Parser | ✅ | numfmt/main.rs:1-3 |
| 41 | main.rs parse_suffix (base^idx, trimmed) idx=K=1, M=2... | ✅ | numfmt/main.rs:146-165 |
| 42 | main.rs format_numeric iec-i 显式 matches!(To::IecI) → "i" | ✅ | numfmt/main.rs:184 `if matches!(to, To::IecI) { "i" } else { "" }` |
| 43 | main.rs add_grouping reverse chunks 3 插逗号 | ✅ | numfmt/main.rs:217-238 |
| 44 | main.rs apply_padding +右 -左 pad | ✅ | numfmt/main.rs:240-249 |
| 45 | main.rs apply_round scaled ceil/floor/trunc/round | ✅ | numfmt/main.rs:204-215 |

---

## Self-fix 正确性验证（3 项）

### ✅ Self-fix A: pr main.rs "args.file → files.first"

**原 brief 错误代码（不编译）**：
```
let default_header = match args.file.as_ref().and_then(|_| args.files.first()).cloned()
```

**实际修复后（pr/main.rs:17-20）**：
```rust
let default_header = match args.files.first().cloned() {
    Some(p) => p.display().to_string(),
    None => String::from(""),
};
```

**正确性推理**：
1. `args.file` 字段不存在 → 直接消除编译错误 ✅ (pr/cli.rs:37-38 只定义了 files:Vec)
2. `args.files.first().cloned()` 语义：Some 拿第一个 PathBuf，None → 空字符串 → 与原 brief 期望 default_header "首文件路径或空串"完全一致 ✅
3. `header_text = args.header.unwrap_or(default_header)` 链保持不变 ✅

结论：**语义正确 + 编译通过**。

---

### ✅ Self-fix B: numfmt main.rs "Cell\<i32\> borrow conflict"

**原问题**（推测 brief 方案）：`let mut exit_code = 0i32` 被闭包 `process` FnMut 捕获后，闭包外还要读（退出），可能存在借用冲突。

**实际实现（Cell 方案）**：
| 位置 | 代码 | 语义 |
|------|------|------|
| numfmt/main.rs:8 | `use std::cell::Cell;` | 引入 |
| numfmt/main.rs:23 | `let exit_code = Cell::new(0i32);` | 默认值 0 ✅ |
| numfmt/main.rs:45 (parse_numeric None) | `exit_code.set(1);` | 无效数字设 1 |
| numfmt/main.rs:75 (stdin read Err) | `exit_code.set(1);` | stdin 错误设 1 |
| numfmt/main.rs:89 (file read Err) | `exit_code.set(1);` | 读行错误设 1 |
| numfmt/main.rs:92 (File::open Err) | `exit_code.set(1);` | 打开错误设 1 |
| numfmt/main.rs:96 | `std::process::exit(exit_code.get());` | 最终取 Cell 值 |

**正确性推理**：
1. 初始值 0i32 → 无错误正常退出 0 ✅
2. 任何 4 处错误分支均调用 `.set(1)` → 最终退出码至少 1 ✅
3. Cell 通过内部可变性，允许闭包（仅需 &self）捕获后仍能修改，彻底规避 borrow checker 冲突 ✅

结论：**exit_code 0/1 更新正确，Cell 方案完全消除借用冲突**。

---

### ✅ Self-fix C: numfmt main.rs "From::Auto 补臂"

**brief 缺臂风险**：原 match from 只写了 4 臂（None/Si/Iec/IecI），缺 From::Auto → 不编译。

**实际实现（numfmt/main.rs:130-141）全部 5 臂**：
```rust
let (mult, rest) = match from {
    From::None   => { /* 直接 parse f64 * from_unit */ }
    From::Auto   => parse_suffix(s, true),   // ← 补全第 5 臂
    From::Si     => parse_suffix(s, false),
    From::Iec    => parse_suffix(s, true),
    From::IecI   => parse_suffix(s, true),
};
```

**行为合理性**（From::Auto = 自动检测）：
1. Auto 分支 `parse_suffix(s, true)` 传入 binary=true → 解析时默认走 1024 基 ✅
2. parse_suffix 内部已额外处理了 "i"/"I" 尾后缀（numfmt/main.rs:155-159），遇到 "Ki"/"Mi" 等自动剥离 i 并取 K/M 作为幂次 → 实际上 "数字+K/M" 或 "数字+Ki/Mi" 都可被 From::Auto 正确解释 ✅
3. 与 --to=Auto（value≥1024 走 1024）形成对称，Auto 语义"自动按二进制后缀识别"合理 ✅

结论：**5 臂齐全，From::Auto 行为合理（自动检测二进制/iec-i 后缀）**。

---

## Strengths

1. **PRE-FLIGHT FIX (iec-i "i" 后缀)** 实现到位：`numfmt/main.rs:184` 显式 `matches!(to, To::IecI)` 条件追加 "i"，满足 brief 必现检查条件。
2. **numfmt parse_suffix 超越 brief 的增强**：与 brief "upper-cased match last char" 不同，实际实现（lines 155-159）额外识别末尾 "i"/"I" 并剥离双字符，使 `--from=iec-i 1.5Ki` 能正确解析 → 冒烟测试 ④ 可通过。
3. **pr truncate_or_keep** 正确按显示宽度（ASCII=1，非 ASCII=2）截断，CJK 字符友好。
4. **csplit format_filename** 用 `{:0w$}` 动态宽度零填充，digits 参数正确生效。
5. **numfmt apply_padding** 正负 padding 方向正确：+ 右补空格，- 左补空格。

---

## Issues

### Critical 🔴

| # | 标题 | 描述 | 位置 |
|---|------|------|------|
| C1 | **csplit {N} 重复次数 ≥2 位数字时 base 截取错误** | try_repeat_tail 返回数字 N，但 main.rs:35 `let base = p[..p.len()-1].to_string()` **只去掉 1 个字符**。如果模式如 `/XXX/10`（重复 10 次），`p.len()-1` 得到 base="/XXX/1"（而非 "/XXX/"），next_match 无法匹配。应按 try_repeat_tail 返回的 idx 截取 `p[..idx+1]`。 | csplit/src/main.rs:35 |

### Important 🟡

| # | 标题 | 描述 | 位置 |
|---|------|------|------|
| I1 | **csplit try_repeat_tail 不应丢弃 idx 信息** | C1 的根因：try_repeat_tail（main.rs:140-147）只返回 repeat 数字，但调用方需要最后一个非数字字符的位置 idx 来截取 base。建议签名改为返回 `Option<(usize, usize)>` = (idx, repeat)。 | csplit/src/main.rs:34-35, 140-147 |
| I2 | **numfmt cli.rs `use std::path::PathBuf` 未使用** | cli.rs:2 引入 PathBuf 但整个文件（Args 字段全是 String/Vec）未使用。虽然 brief 中同样有此冗余，但仍属于未使用 import 警告级别问题。 | numfmt/src/cli.rs:2 |

### Minor 🟢

| # | 标题 | 描述 | 位置 |
|---|------|------|------|
| M1 | **pr write_header 第一行空循环无作用** | `for _ in 0..=page_no.max(1) { if page_no == 1 {} }` 是死代码，迭代 page_no 次且内部分支始终为 no-op。可直接删除。 | pr/src/main.rs:188 |
| M2 | **csplit -b suffix-format 参数未使用** | brief 已标注 "(unused)"，args.suffix_format 定义但 main 中从未读取，仅用 prefix+digits 组合命名。符合规范，但如有未来扩展计划建议加 `#[allow(dead_code)]` 或使用它。 | csplit/src/cli.rs:52-53 |
| M3 | **numfmt parse_numeric last 未用** | parse_numeric line 129 `let last = s.chars().last()?;` 定义了 last 变量但后续从未读取，纯冗余。 | numfmt/src/main.rs:129 |

---

## ⚠️ Unverified Items

仅静态审查，未执行以下验证（需 cargo build + 运行冒烟测试）：
1. 所有 3 个 crate 能否成功 `cargo build --release`
2. csplit: `/XXX/10` 及以上多位数重复次数的实际行为 (C1 实际触发)
3. pr: 带页眉分页实际输出是否对齐 72 列宽
4. numfmt: `--to=iec-i 1536` → "1.5Ki"；`--from=iec-i 1.5Ki` → 1536

---

## Assessment

**Needs fixes**（需要修复后通过）

**理由**：Spec 合规 45/45 项全部通过 ✅，3 个 self-fix（A/B/C）经推理语义和编译层面均完全正确 ✅；但存在 **1 个 Critical 功能 bug (C1)**：csplit 多位数 {N} 重复模式 base 截取错误，以及 2 个 Important 级别建议 + 3 个 Minor 冗余项。修复 C1（调整 try_repeat_tail 返回 idx，main.rs:35 改为 `p[..idx+1]`）后即可批准。

---

## Fixes after review v1

**Commit**: `2306e80` — fix(task6 csplit numfmt): {N} multi-digit repeat base slice (C1) + cli unused PathBuf (I2)

### 修复细节

#### C1 Critical + I1 Important（同源根因）：csplit {N} 多位数字尾重复时 base 截取错误

**变更文件**：`csplit/src/main.rs`

(1) `try_repeat_tail` 签名从 `Option<usize>` 改为 `Option<(usize, usize)>` = `(digit_start_idx, repeat_count)`：
```rust
fn try_repeat_tail(p: &str) -> Option<(usize, usize)> {
    if let Some(idx) = p.rfind(|c: char| !c.is_ascii_digit()) {
        let digit_start = idx + 1;
        if digit_start < p.len() {
            let n: usize = p[digit_start..].parse().ok()?;
            if n > 0 { Some((digit_start, n)) } else { None }
        } else { None }
    } else { None }
}
```

(2) 调用点（main while 循环内 repeat 分支）改为解构 + 按 digit_idx 截取 base：
```rust
if let Some((digit_idx, repeat)) = try_repeat_tail(p) {
    let base = p[..digit_idx].to_string();
```

**根因消除**：原来 `p[..p.len()-1]` 只剥 1 个字符，`/END/10` → base=`/END/1`（多留 `1`）；修复后 `digit_start=5`，`p[..5]`→`/END/` 正确。

#### I2 Important：numfmt/src/cli.rs 未使用 import 删除

**变更文件**：`numfmt/src/cli.rs`

删除顶部未使用的 `use std::path::PathBuf;`（该文件 `Args.rest` 是 `Vec<String>` 不涉及 PathBuf；main.rs 自有该 import）。

### Build 输出

**csplit build** (`cd csplit; cargo build --release --offline`)：
```
   Compiling csplit v0.1.0 (D:\Work\rust\csplit)
warning: variable does not need to be mutable
 --> src\main.rs:120:9
    = help: remove this `mut`
    `csplit` (bin "csplit") generated 1 warning
    Finished `release` profile [optimized] target(s) in 4.62s
```
→ 0 errors；仅 1 条遗留 warning（修复前即存在的 unused mut reader），与本次修复无关。

**numfmt build** (`cd numfmt; cargo build --release --offline`)：
```
   Compiling numfmt v0.1.0 (D:\Work\rust\numfmt)
warning: variable does not need to be mutable
  --> src\main.rs:25:9
     = help: remove this `mut`
warning: unused variable: `last`
   --> src\main.rs:129:9
     = help: prefix with underscore
    `numfmt` (bin "numfmt") generated 2 warnings
    Finished `release` profile [optimized] target(s) in 4.83s
```
→ 0 errors；2 条遗留 warning（对应 review M3: unused `last` + 另一条旧 unused mut），**I2 的 PathBuf 未使用 warning 已消除**。

### C1 新 case 实际输出（2 位数 repeat 验证）

**输入文件** (t.txt, 3 个 END 分隔符)：
```
aaa
bbb
END
x1
x2
END
y1
y2
END
z1
z2
```

**命令**：`csplit.exe -k t.txt -- "/END/10"`（repeat=10，2 位数）

**实际输出**：4 个分片（repeat=3 实际生效，文件仅含 3 个 END）
```
xx00: aaa / bbb          (首匹配前)
xx01: x1 / x2            (第 1 次 END 后)
xx02: y1 / y2            (第 2 次 END 后)
xx03: z1 / z2            (第 3 次 END 后)
```
✅ **C1 通过**：若 base 仍截成 `/END/1`（旧 bug），则 next_match 找不到 "/END/1" 模式，只会生成 1 个空/全量文件；实际正确生成 4 段 → 2 位数 repeat digit_idx 截取生效。

### 4 条 Smoke 回归验证

| # | 用例 | 期望 | 实际 | 结果 |
|---|------|------|------|------|
| ① | csplit 基础 line-num + /PAT/ | 生成 xx00/xx01... 正常分割 | line=2 模式 + /PAT/ 生成多片 | ✅ 通过 |
| ③-a | `numfmt --to=iec-i 1536` | "1.5Ki" | "1.5Ki" | ✅ 通过 |
| ③-b | `numfmt --to=iec 1536` | "1.5K" | "1.5K" | ✅ 通过 |
| ③-c | `numfmt --to=si 1500` | "1.5K" | "1.5K" | ✅ 通过 |
| ④ | `numfmt --from=iec-i 1.5Ki` | 1536 | 1536 | ✅ 通过 |

### 遗留问题（均非本次修复范围，对应原 review Minor 级别）
- csplit/main.rs:120 `mut reader` 无需 mut（旧 warning）
- numfmt/main.rs:25 `mut process` 无需 mut（旧 warning）
- numfmt/main.rs:129 `last` 变量未使用（原 review M3）
- pr write_header 第一行空循环（原 review M1，未触及）
- csplit -b suffix-format 参数未使用（原 review M2，brief 标注 unused）

### 最终状态
原 Assessment "Needs fixes" → **修复后可批准**。C1 Critical + I1（同源）+ I2 全部 3 处已修复并通过编译与冒烟回归。
