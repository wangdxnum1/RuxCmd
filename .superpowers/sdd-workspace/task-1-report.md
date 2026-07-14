# Task 1: factor 任务报告

## 实现内容

按照 task-1-brief.md 的要求，创建了质因数分解工具 `factor` 的独立 Rust crate，包含以下三个文件：

1. **factor/Cargo.toml** - 项目配置，name = "factor"，version = "0.1.0"，edition = "2024"，依赖 clap 4 (derive 特性)
2. **factor/src/cli.rs** - 命令行参数解析，使用 clap Parser，禁用默认 --version 标志，自定义 -v/--version 为 SetTrue 动作
3. **factor/src/main.rs** - 主程序逻辑，包含：
   - 从命令行参数或 stdin（每行一个）读取正整数
   - `factor_one()` 函数实现试除法质因数分解
   - 错误通过 eprintln!("factor: {}", e) 输出
   - 使用 std::process::exit 返回退出码

## 构建输出

执行 `cargo build --release --offline` 结果：

```
   Locking 21 packages to latest Rust 1.97.0 compatible versions
 Compiling windows-link v0.2.1
 Compiling proc-macro2 v1.0.106
 Compiling unicode-ident v1.0.24
 Compiling quote v1.0.46
 Compiling anstyle v1.0.14
 Compiling utf8parse v0.2.2
 Compiling windows-sys v0.61.2
 Compiling once_cell_polyfill v1.70.2
 Compiling anstyle-parse v1.0.0
 Compiling is_terminal_polyfill v1.70.2
 Compiling colorchoice v1.0.5
 Compiling strsim v0.11.1
 Compiling clap_lex v1.1.0
 Compiling heck v0.5.0
 Compiling anstyle-wincon v3.0.11
 Compiling anstyle-query v1.1.5
 Compiling anstream v1.0.0
 Compiling clap_builder v4.6.0
 Compiling syn v2.0.118
 Compiling clap_derive v4.6.1
 Compiling clap v4.6.1
 Compiling factor v0.1.0 (D:\Work\rust\factor)
  Finished `release` profile [optimized] target(s) in 14.87s
```

构建成功，无错误，生成 `target\release\factor.exe`。

## 冒烟测试结果

### 测试 1: factor 60
命令：`.\target\release\factor.exe 60`

实际输出：
```
60: 2 2 3 5
```
预期：`60: 2 2 3 5` ✓ 通过

### 测试 2: factor 1 12 131
命令：`.\target\release\factor.exe 1 12 131`

实际输出：
```
1:
12: 2 2 3
131: 131
```
预期：
```
1:
12: 2 2 3
131: 131
```
✓ 通过

### 测试 3: echo 1234567 | factor
命令：`echo 1234567 | .\target\release\factor.exe`

实际输出：
```
1234567: 127 9721
```
预期：`1234567: 127 9721` ✓ 通过

（验证：127 × 9721 = 1,234,567 ✓）

## 变更文件

- 新建: factor/Cargo.toml
- 新建: factor/src/cli.rs
- 新建: factor/src/main.rs

## Git 提交

```
commit 32e5728
Author: Tim
Date:   2026/7/14

    feat: add factor (prime factorization) - round 7

 3 files changed, 104 insertions(+)
```

## 自审

- [x] Cargo.toml 的 name、version、edition、description、authors、dependencies 全部与 brief 一致
- [x] cli.rs 中 disable_version_flag = true，-v/--version 使用 ArgAction::SetTrue
- [x] main.rs 开头有 `mod cli; use clap::Parser;`
- [x] 错误输出格式 eprintln!("factor: {}", ...)
- [x] 使用 std::process::exit 退出
- [x] 源代码无任何注释
- [x] 未添加 Cargo.lock 或 target/（已 gitignore）
- [x] 3/3 冒烟测试全部通过
- [x] 构建无 error

---

## 代码审查修复：移除死代码

### 变更内容

修复代码审查中 Important 级别发现：`factor/src/main.rs` 第 74-76 行的 `if d < 3 { break; }` 为不可达死代码。原因是 `d` 初始值为 3，而 `saturating_add(2)` 永远不会使值减小，因此 `d < 3` 条件永假。

**修改文件：** factor/src/main.rs
- 删除了 3 行死代码：
  ```rust
  if d < 3 {
      break;
  }
  ```
- 保留了 `d = d.saturating_add(2);` 及其他所有代码不变。

### 重新构建输出

执行 `cargo build --release --offline` 结果：

```
   Compiling factor v0.1.0 (D:\Work\rust\factor)
    Finished `release` profile [optimized] target(s) in 1.38s
```

构建成功，无错误。

### 冒烟测试结果（修复后重新验证）

**测试 1: factor 60**
```
60: 2 2 3 5
```
✓ 通过

**测试 2: factor 1 12 131**
```
1:
12: 2 2 3
131: 131
```
✓ 通过

**测试 3: echo 1234567 | factor**
```
1234567: 127 9721
```
✓ 通过

### 修复提交

```
commit 35f7c7c
Author: Tim
Date:   2026/7/14

    fix(factor): remove unreachable dead-code check (d<3 never true)

 1 file changed, 3 deletions(-)
```
