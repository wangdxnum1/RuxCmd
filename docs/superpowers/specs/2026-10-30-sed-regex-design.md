# 方向 3 专项设计:sed 正则引擎(Round 19)与 awk 解释器(Round 20)

日期:2026-10-30。基线:main(Round 18 合并后,134 命令)。

## 背景

Round 12 明确推迟的两项大重写是普通用户感知最强的 GNU 差距:

1. **sed**:`s` 命令与地址是字面子串,无正则、无 `&`/`\1` 引用(`crates/sed/src/main.rs` 现 523 行,`CmdKind::Substitute` 走 `bytes_contain`/`replace_substring`)。
2. **awk**:无正则、无数组、无内建函数、无重定向,for 循环直接报错(`crates/awk`,条件求值已在 0.2.0 重写过)。

本专项分两轮,每轮独立验收。

## Round 19:sed 正则(BRE)

### 范围

- `s/BRE/repl/flags`:BRE 模式(锚定 `^`/`$`、`.`、`*`、`[...]`、`\(...\)` 分组、`\{m,n\}` 区间),替换端支持 `&` 与 `\1`–`\9` 分组引用、`&`/`\` 转义;flags `g`、`p`、`i`、数字 N(仅替换第 N 处)、`w FILE`。
- 地址 `/BRE/`、`/BRE/,/BRE/`、`$`;`\%text%` 自定义分隔符地址与 s 命令。
- 非法 BRE → 启动报错退出 1(GNU 语义)。

### 实现要点

- 把 expr(Round 18)的 `bre_to_regex` 提升为 `ruxcmd-core` 共享函数,expr 与 sed 共用;Rust `regex` crate 从 expr 移到 workspace 依赖,core 可选启用(`feature = "regex"`)或直接引入(锁定 1 版本)。
- BRE 反向引用(`\(a\)\1`)rust regex 不支持:sed 启动时报 `sed: -e expression N: back references are unsupported`(记录为差异),不静默错配。
- 替换执行:逐匹配切片重建字节串(沿用现有字节安全约定,非 UTF-8 透传);`g`/N 计数、`p` 打印、`i` 大小写折叠(regex crate 原生 `(?i)`)。
- 地址匹配输出保持原始字节(匹配只用于选择,不改写)。

### 验收

- 现有 sed 测试(字面子集)全部保持通过;
- 新增 GNU 对齐测试:分组捕获替换、`&`、`g/N/p/i`、区间 `\{2,3\}`、多命令脚本、非 UTF-8 字节透传、`y///` 不变;
- 与 /usr/bin/sed 差分抽样(Git Bash 有 sed,可加入 difftest 形态);
- 兼容性表 sed 行改写;Verify 全绿。

## Round 20:awk 解释器(设计要点,另行 spec 细化)

- 最小可用集:模式-动作对、字段 `$1..$NF`/`NF`/`NR`/`FS`/`OFS`/`RS`/`ORS`、关联数组、`if/while/for`、算术与字符串内建函数(`length/substr/index/split/sub/gsub/sprintf`)、正则模式(与 sed 共用 BRE 转换)、管道外重定向仍不支持(打印差异)。
- 结构:独立 `crates/awk` 内部模块(lexer/parser/eval),不引入外部解释器 crate;
- 复用 ruxcmd-core ByteLines 字节安全读行。
- 该轮工作量 ≈ 2-3 个 Round 15 量级命令,执行前单独出实施计划。

## 明确不做

- ERE(`-E`/`--regexp-extended`)可后续在 BRE 转换器加开关,不进入 Round 19 验收;
- sed 的 `e`、`R`、`W` 等 GNU 扩展命令;
- awk 的用户自定义函数与协程/管道重定向。
