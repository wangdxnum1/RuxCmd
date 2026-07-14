# 第 7 轮 Linux 工具移植设计：基础文本处理工具集

**日期**：2026-07-14
**作者**：Tim
**背景**：前 6 轮已移植 78 个 Linux 命令行工具到 Windows (Rust, 静态 CRT MT 模式)。本轮移植 10 个基础文本处理类 coreutils 工具。

---

## 1. 范围与目标

### 1.1 本轮移植的 10 个工具

| # | 工具 | 功能简述 | 实现难度 | 依赖策略 |
|---|------|----------|----------|----------|
| 1 | `base64` | Base64 编码 / 解码 | ⭐ | 标准库手写 |
| 2 | `expand` | Tab → 空格转换 | ⭐ | 标准库 |
| 3 | `unexpand` | 空格 → Tab 转换 | ⭐ | 标准库 |
| 4 | `tac` | 反向输出文件（按行/分隔符） | ⭐⭐ | 标准库（正则可选） |
| 5 | `column` | 按列格式化输出 | ⭐⭐ | 标准库 |
| 6 | `shuf` | 随机打乱输入行 | ⭐⭐ | 标准库手写 Fisher–Yates |
| 7 | `csplit` | 按上下文模式分割文件 | ⭐⭐⭐ | 标准库（正则可选） |
| 8 | `pr` | 分页/多列打印格式化 | ⭐⭐ | 标准库 |
| 9 | `numfmt` | 数字格式化（人类可读 SI/IEC） | ⭐⭐ | 标准库 |
| 10 | `factor` | 整数质因数分解 | ⭐ | 标准库 |

### 1.2 不在范围内

- 不引入新 crate 依赖（除 `clap = { version = "4", features = ["derive"] }` 作为既有统一依赖）；
- 不实现 GNU 所有偏门选项（仅实现设计中列出的常用子集）；
- 不处理跨平台文件路径编码（沿用 Rust Path/PathBuf，Windows 原生）。

---

## 2. 总体架构与代码组织

### 2.1 目录结构

每个工具独立 crate：
```
d:\Work\rust\
├── base64/
├── expand/
├── unexpand/
├── tac/
├── column/
├── shuf/
├── csplit/
├── pr/
├── numfmt/
└── factor/
```

每个 crate 包含：
- `Cargo.toml` — 包定义与依赖
- `src/cli.rs` — 命令行参数定义（clap derive）
- `src/main.rs` — 主逻辑

### 2.2 Cargo.toml 统一模板

```toml
[package]
name = "<TOOL_NAME>"
version = "0.1.0"
edition = "2024"
description = "<SHORT_DESCRIPTION>"
authors = ["Tim"]

[dependencies]
clap = { version = "4", features = ["derive"] }
```

### 2.3 src/cli.rs 统一模板

```rust
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "<TOOL_NAME>", version, about, disable_version_flag = true)]
pub struct Args {
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub version: bool,

    // 其他工具专属参数...
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,
}
```

**关键约定**：`disable_version_flag = true` + 自定义 `-v/--version`（不使用 `-V`），与前 78 个工具保持一致。

### 2.4 src/main.rs 统一骨架

```rust
mod cli;

use clap::Parser;

fn main() {
    let args = cli::Args::parse();

    if args.version {
        println!("<TOOL_NAME> 0.1.0");
        return;
    }

    let mut exit_code = 0i32;

    // ... 工具主逻辑 ...

    std::process::exit(exit_code);
}
```

### 2.5 全局配置（已就绪）

- **静态 CRT 链接**：`.cargo/config.toml` 已设置 `target-feature=+crt-static`，对所有 crate 生效；
- **构建流水线**：`build-all.bat` 调用 `cargo build --release --offline`，复制到 `D:\develop\rust-tools`；
- **忽略文件**：`.gitignore` 已忽略 `**/target/`、`**/*.exe`、`**/*.pdb`、`Cargo.lock`。

---

## 3. 各工具详细设计

### 3.1 base64

**CLI**：
```
base64 [OPTIONS] [FILE]...
  -d, --decode       解码模式（默认编码）
  -w, --wrap=COLS    编码后每行 COLS 字符（默认 76，0=不换行）
  -v, --version      版本
  FILE               输入文件（空或 '-' 读 stdin）
```

**实现要点**：
- 编码：读取输入字节流，每 3 字节组 → 4 Base64 字符；末尾不足补 `=`；
- 解码：忽略所有空白字符（\s, \t, \r, \n），每 4 字符组 → 3 字节；遇到非法字符报错退出；
- `-w`：编码时每行累计到 COLS 插入换行；`-w 0` 不插入；
- Base64 字母表：`A–Z a–z 0–9 + /`（标准 MIME 表）。

**输入模式**：支持多个文件（含 `-`）连续处理；解码时多个文件视为一个连续字节流。

---

### 3.2 expand

**CLI**：
```
expand [OPTIONS] [FILE]...
  -t, --tabs=N          每 N 字符一个 tab stop（默认 8）
  -t, --tabs=LIST       自定义 tab stop 列表（逗号/空白分隔），如 1,4,8,20
  -i, --initial         仅转换行首空白中的 Tab
  -v, --version         版本
```

**实现要点**：
- 维护当前列号 `col`（初始 0）；
- 遇到 `\t`：在 LIST 模式下找下一个大于 `col` 的 stop 位置；在 N 模式下跳到 `((col / N) + 1) * N`；输出 `(next_stop - col)` 个空格；
- 遇到 `\n`：输出换行，`col = 0`；
- 其他字符：输出字符，`col += 1`（按字符宽度 1 简化处理，不考虑全角 unicode 宽字符）；
- `-i` 模式：遇到首个非空白/非 Tab 字符后，后续字符直接输出不再转换。

---

### 3.3 unexpand

**CLI**：
```
unexpand [OPTIONS] [FILE]...
  -t, --tabs=N       每 N 字符一个 tab stop（默认 8）
  -a, --all          转换所有能被 Tab 替换的空白（默认仅转换行首连续空白）
  -v, --version      版本
```

**实现要点**：
- 默认模式：每行开头累计空白和 Tab，重算并尽可能替换为 Tab + 剩余空格；行中非行首空白不处理；
- `-a` 模式：全行列扫描，每当连续空白跨越 tab stop 边界时替换为 `\t` + 少量空格；
- 与 `expand` 对称的 tab stop 计算逻辑（共享同一实现思路）。

---

### 3.4 tac

**CLI**：
```
tac [OPTIONS] [FILE]...
  -b, --before          分隔符放在输出块之前（默认之后）
  -r, --regex           将 -s 的分隔符视为正则表达式
  -s, --separator=S     记录分隔符字符串（默认 '\n'）
  -v, --version         版本
```

**实现要点**：
- 默认模式（换行分隔）：逐行读入 `Vec<String>`，反向 `rev().for_each(print)`；
- 自定义 `-s STRING`：按字节扫描输入，找到分隔字符串位置切片成块，反向输出；
- 分隔符归属：默认（无 `-b`）`chunk1 \n chunk2 \n` → 反转后 `chunk2 \n chunk1 \n`；`-b` 则 `\n chunk1 \n chunk2` → 反转后 `\n chunk2 \n chunk1`；
- `-r` 正则模式：**简化实现**，如果离线环境无 `regex` crate，则在文档和帮助信息中声明 "-r 暂未在 Windows 构建中启用" 并以字面分隔符回退；若缓存中有 regex crate 则启用（优先标准库回退方案，保持无新增依赖）。

---

### 3.5 column

**CLI**：
```
column [OPTIONS] [FILE]...
  -t, --table              表格对齐模式（默认开启）
  -s, --separator=STRING   输入列分隔符（默认任意空白）
  -o, --output-separator=S 输出列分隔符（默认两个空格）
  -c, --output-width=N     输出宽度（默认 80）
  -x, --fillrows           先横排再竖排（仅多列模式，默认先竖后横）
  -v, --version            版本
```

**实现要点**：
- 读取所有行（多文件级联，含 stdin）；
- 每行列分割：`-s` 指定分隔符字符串 split；无 `-s` 时按连续空白 split；
- 计算每列最大 `display_width`（按字符数简化）；
- 输出：每行按 `{:<width1}{}{:<width2}{}{:<width3}...` 格式化，插入 `-o` 分隔符；
- 不支持 `-c` / `-x` 的复杂填充布局（仅保留参数接受，在 `-h` 中声明，行为回退到默认表格模式），以降低复杂度。

---

### 3.6 shuf

**CLI**：
```
shuf [OPTIONS] [FILE]
  -e, --echo=ARGS...     将 ARGS 作为输入行（不读文件）
  -n, --head-count=N     仅输出前 N 行（不放回抽样 N 条）
  -o, --output=FILE      输出文件（默认 stdout）
  -r, --repeat           允许重复（有放回抽样）
  -z, --zero-terminated  记录分隔符 '\0' 而非 '\n'
  -v, --version          版本
  FILE                   输入文件（空或 '-' 读 stdin）
```

**实现要点（不引入 rand crate）**：
- 随机源：`std::collections::hash_map::RandomState::new().build_hasher().finish()` 取 u64 种子；
- 标准库 PRNG：实现简单 `xorshift64` 或 `splitmix64` PRNG，一次初始化种子；
- 洗牌算法：Fisher–Yates（Knuth shuffle）原地 `O(n)`；
- `-n`：若 `-r` 则循环生成索引采样；若无 `-r` 则洗牌后取前 N 或全部输出；
- `-z`：分隔符为 `\0`，其他逻辑不变；
- `-o`：写入指定文件（创建或覆盖）；
- `-e`：优先于 FILE，将命令行参数作为输入行集合。

---

### 3.7 csplit

**CLI**：
```
csplit [OPTIONS] FILE PATTERN...
  -f, --prefix=PREFIX   输出文件名前缀（默认 'xx'）
  -n, --digits=N        后缀数字位数（默认 2），生成 aa, ab 或 00, 01
  -k, --keep-files      出错时不删除已生成文件
  -z, --elide-empty-files  不输出空片段
  -s, --quiet / -s      静默（不输出字节数）
  -v, --version         版本
  FILE                  输入文件（必须指定）
  PATTERN               一个或多个分割模式
```

**PATTERN 语法（简化子集）**：
1. `NUMBER`（纯数字行号，1-based）：在第 NUMBER 行之前切；
2. `/REGEX/`：找到下一行匹配正则（简化为 `String::find` 子串匹配）时，在其前切分，副本继续；
3. `{COUNT}`：重复前一个模式 COUNT 次。

**实现要点**：
- 逐行读入到内存（若过大则分块流式，简化实现按行向量处理）；
- 依据 PATTERN 序列计算断点下标 `Vec<usize>`；
- 每个片段写入 `{prefix}{:0digits$}` 文件名；
- 每次写入成功后，向 stdout 输出该文件字节数；
- `-z` 模式：0 字节文件跳过不创建也不输出；
- 出错处理：除非 `-k`，否则 `rm` 已生成文件后退出码 1。

---

### 3.8 pr

**CLI**：
```
pr [OPTIONS] [FILE]...
  -NUM                  输出 NUM 列（简写 --columns=NUM）
  -l, --length=PAGE     每页行数（默认 66）
  -o, --indent=MARGIN   左缩进 MARGIN 个空格
  -w, --width=WIDTH     行宽（默认 72）
  -h, --header=TEXT     页眉文字（默认文件名）
  -t, --omit-header     省略页眉、页脚和页间空白
  -d, --double-space    双倍行距（行间多一空行）
  -J, --join-lines      超长行不截断
  -v, --version         版本
```

**实现要点**：
- 页眉格式（非 `-t` 时每页开头）：
  ```
  2026-07-14 16:00  <HEADER_TEXT or FILENAME>   Page <page_no>
  (空行)
  ```
- 每页行数：header + 空行 + body（length - 5 行左右，简化估算）+ 页脚空行；
- `-NUM` 多列模式：将所有输入行按列轮转分配（默认先竖后横 column-major）；每行按列宽截断或 `-J` 不截断；
- `-d` 双倍行距：正文每行后追加一空行；
- `-o`：所有输出行前置 `MARGIN` 个空格；
- 简单实现不支持复杂两文件合并（`-m`）。

---

### 3.9 numfmt

**CLI**：
```
numfmt [OPTIONS] [NUMBER]...
  --to=UNIT            输出单位：none | auto | si | iec | iec-i（默认 none）
  --from=UNIT          输入单位：同 UNIT（默认 none）
  --to-unit=N          输出时除以 N
  --from-unit=N        输入时先乘以 N
  --field=FIELDS       只处理指定列（逗号分隔，如 1,3-5）
  -d, --delimiter=X    字段分隔符（默认空白）
  --padding=N          输出对齐 padding（正值右对齐，负值左对齐）
  --suffix=SUFFIX      数字后追加字符串
  --header=N           跳过前 N 行原样输出
  -v, --version        版本
  NUMBER               要格式化的数字；无则读 stdin
```

**单位规则**：
- SI（`--to=si`）：K=10³, M=10⁶, G=10⁹, T=10¹², P=10¹⁵, E=10¹⁸, Z=10²¹, Y=10²⁴；
- IEC（`--to=iec`）：K=2¹⁰, M=2²⁰, G=2³⁰, T=2⁴⁰, P=2⁵⁰, E=2⁶⁰（缩写 K,M,G... 无 'i'）；
- IEC-I（`--to=iec-i`）：同 IEC 量级，缩写 Ki, Mi, Gi...；
- `--to=auto`：以 1000 为边界选「最合适」的 SI 量级；

**实现要点**：
- `--from-unit` 乘法 → 核心除法 / 量级匹配 → 格式化为 `"<sign><int>.<frac> <suffix><suffix2>"`；
- 未指定 `NUMBER` 时读 stdin，每行根据 `--field` 挑选字段处理后重组行；
- `--header`：前 N 行整行原样输出，不做格式化。

---

### 3.10 factor

**CLI**：
```
factor [OPTIONS] [NUMBER]...
  -v, --version   版本
  NUMBER          要分解的正整数（空则读 stdin，每行一个）
```

**实现要点**：
- 数据类型：`u64`（最大 2⁶⁴ - 1 = 1.8e19）；
- 特殊输出：`1:` 后空（无质因数）；`0:` 报错提示 invalid；负数提示无效；
- 算法：
  1. 先除尽 2；
  2. 从 `d = 3` 起，`d += 2` 到 `sqrt(n)`，试除累计质因数；
  3. 循环结束 `n > 1` 时，残余 n 本身是质数，输出。
- 命令行有 `NUMBER` 参数：逐个处理；否则读 stdin，每行解析一个。

---

## 4. 错误处理规范

### 4.1 统一错误输出格式

| 场景 | stderr 格式 | 退出码 |
|------|-------------|--------|
| 打开文件失败 | `"<tool>: <path>: <os_error>"` | 1 |
| 读取失败（stdin/文件） | `"<tool>: error reading <stdin|path>: <error>"` | 1 |
| 写入 stdout 失败 | `"<tool>: write error: <error>"` | 1 |
| 无效参数值（非 clap 捕获） | `"<tool>: invalid <param_name> '<value>': <reason>"` | 1 |
| 数值溢出 / 解析失败 | `"<tool>: invalid number: '<raw>'"` | 1 |
| 多文件模式下个别失败 | 报错 continue，整体 exit_code=1 | 累计非零 |

### 4.2 `-` 作为文件

所有工具遵循：FILE 列表中出现 `"-"` 表示读取 stdin；允许与真实文件名混合。

---

## 5. 构建集成

### 5.1 build-all.bat 变更

PROJECTS 变量末尾追加：
```
base64 expand unexpand tac column shuf csplit pr numfmt factor
```

### 5.2 构建命令

```bat
cargo build --release --offline
```

产出路径：`<tool>/target/release/<tool>.exe` → 复制到 `D:\develop\rust-tools\`。

### 5.3 Git 提交

- 提交文件：每个新工具的 `Cargo.toml`、`src/cli.rs`、`src/main.rs`，以及修改后的 `build-all.bat`、新增的 `docs/superpowers/specs/2026-07-14-round7-text-tools-design.md`；
- 不纳入：`target/`、`.exe`、`.pdb`、`Cargo.lock`（已被 .gitignore 过滤）；
- 提交信息：`Add 10 new Linux text-processing tools for Windows (round 7)`。

---

## 6. 验收检查清单

| 项 | 说明 |
|----|------|
| 编译通过 | 10 个工具 `cargo build --release --offline` 全部成功，无 warning 优先 |
| 可执行文件大小合理 | 静态链接 MT，每个 exe 独立可运行 |
| 基本功能冒烟 | 每个工具至少 3 条常见用法验证通过 |
| build-all.bat 集成 | 新 10 个名称已加入 PROJECTS 列表 |
| 产物复制 | `D:\develop\rust-tools` 目录出现 10 个新 exe |
| Git 状态 | `git status` 只显示预期源代码文件，无 target 产物污染 |

---

## 7. 风险与回退

| 风险 | 缓解策略 |
|------|----------|
| `shuf` 无 `rand` crate 导致随机性弱 | 用标准库 RandomState 取 64bit seed + 自研 PRNG，对日常 CLI 使用足够 |
| `csplit` PATTERN 正则/复杂解析 | 仅支持最常用的行号和子串匹配，在帮助信息中标注 |
| `column` / `pr` 多列对齐 unicode 宽度 | 以字符计数代替显示宽度，接受中英文混排轻微错位 |
| 编译时间过长（10 个 crate 并行） | `build-all.bat` 已有逐个串行构建流程 |
