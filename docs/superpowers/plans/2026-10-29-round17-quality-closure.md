# Round 17 质量收口执行计划

> **For agentic workers:** 按 superpowers:executing-plans 逐任务执行。步骤用 checkbox 跟踪。

**Goal:** 为仅冒烟命令补行为测试,清偿经核实的 P2 缺陷,同步文档与差分矩阵;命令数保持 125。
**Spec:** `docs/superpowers/specs/2026-10-29-round17-quality-closure-design.md`。
**约束:** 分支 codex/round17-quality-closure;cargo 命令 --locked;P2 修复先失败测试后实现;一命令一提交;错误英文 stderr;不新增命令;不动大型重写项(见 spec 遗留清单)。

## Task 1:确定性命令行为测试(批 1)

Files:`crates/{cmp,expand,factor,rev,strings,zcat}/tests/*.rs`(新增)。

- [ ] cmp:相同/不同(报 `differ: byte N, line M`)/EOF 短文件/-s 静默/-l 八进制列;退出码 0/1/2。
- [ ] expand:默认 tab8、-t N、-t 列表、-i 仅行首、无尾换行、CRLF 保留。
- [ ] factor:12→`12: 2 2 3`、素数、1、0、大数;错误参数 exit 1。
- [ ] rev:基本反转、UTF-8、空行、无尾换行保留、多文件。
- [ ] strings:默认 -n 4、-n N、-t d 偏移、非打印过滤、二进制混合。
- [ ] zcat:内嵌最小 gzip 字节解压正确;损坏流报错退出非零。
- [ ] `cargo test -p <pkg> --locked` 逐包通过,提交。

## Task 2:确定性命令行为测试(批 2)

Files:`crates/{mkdir,shred,realpath,readlink,uname,whoami}/tests/*.rs`(新增/扩充)。

- [ ] mkdir:基础、已存在报错 exit 1、-p 幂等、-v 消息 `mkdir: created directory 'x'`、-m 只读位映射。
- [ ] shred:文件内容覆写且大小不变(默认保留)、-u 删除、不存在报错 exit 1;仅对临时文件操作。
- [ ] realpath:相对/绝对/`..` 归一、--strip-components、不存在文件行为(按现有 -e/-m 语义锁定)。
- [ ] readlink:链接目标输出;创建符号链接无特权时跳过(打印提示不算失败)。
- [ ] uname:-s 非空、-m 架构形态、-a 单行、--kernel-release 非空。
- [ ] whoami:非空、--version 路径。
- [ ] 逐包测试通过,提交。

## Task 3:系统类命令有界行为测试

Files:`crates/{df,free,iostat,mpstat,sar,time,kill,pgrep,pkill,who,clear,reset,mkfifo}/tests/*.rs`。

- [ ] df/free:exit 0、表头存在、-h/-m/-g 缩放后仍为数值、free -t 出现 Total 行。
- [ ] iostat/mpstat/sar:exit 0、表头与数值列范围(参照 top 断言方式)。
- [ ] time:子进程退出码透传(如 `cmd /c exit 7` → 7)、输出含 real/user/sys。
- [ ] kill:kill -l 非空;杀 spawn 的长命子进程成功;不存在 PID 报错 exit 1。
- [ ] pgrep/pkill:spawn `cmd /c ping -n 30 127.0.0.1 >nul`,pgrep 命中子进程 PID/名称;pkill 使子进程在 deadline 内退出;无匹配 exit 1;测试保证子进程清理。
- [ ] who/whoami:exit 0 与有界输出;-b 有 boot 行。
- [ ] clear/reset:输出为 ANSI 形态(非空,含 ESC 或清屏序列)或按现有 TTY 行为锁定。
- [ ] mkfifo:创建成功且不可按普通文件读写语义锁定(按现有实现)。
- [ ] 逐包测试通过,提交。

## Task 4:printf 全格式化器(TDD)

Files:`crates/printf/{cli.rs,main.rs,tests/format.rs}`。

- [ ] 先写失败测试:宽度/精度/标志(- 0 + 空格 #)、d/i/u/o/x/X、f/e/E/g/G(指数 2 位带符号、去尾零)、c/s/%b(\c 截断)/%q/%%、格式复用、无转换指令多余参数警告、未知指令报错 exit 1、缺参语义、新转义 \a\b\f\v\'\?\e。
- [ ] 实现自写格式化器;非法数值 stderr 报错该处输出 0、最终 exit 1;LC_ALL=C 下 ASCII 引号。
- [ ] 全包测试 + 与 /usr/bin/printf 抽样核对;提交。

## Task 5:sleep 多操作数与后缀(TDD)

Files:`crates/sleep/{cli.rs,main.rs,tests/behavior.rs}`。

- [ ] 失败测试:多操作数求和(0.1+0.1≈0.2s)、d 后缀、`1S` 拒绝、负数/未知选项拒绝 exit 1、`sleep 0` 合法、--version。
- [ ] 实现数值+单位解析求和;报错消息 `sleep: invalid time interval 'xx'`;移除非 GNU -v。
- [ ] 全包测试;提交。

## Task 6:echo 操作数解析与转义补全(TDD)

Files:`crates/echo/{cli.rs,main.rs,tests/behavior.rs}`。

- [ ] 失败测试:`echo -x`→输出 `-x`;`echo --`→输出 `--`;`-ne`/`-nE` 簇;首非选项后 `-x` 为操作数;-e 的 \a\b\c\e\f\v\\ \0NNN\xHH;\c 截断且无换行;默认 -E 不展开。
- [ ] 实现前导选项手工扫描(弃用 clap);逐字节对齐 GNU。
- [ ] 全包测试;提交。

## Task 7:which PATHEXT 与净路径(TDD)

Files:`crates/which/{main.rs,tests/behavior.rs}`。

- [ ] 失败测试:临时目录注入 PATH,精确名优先、.exe/.cmd/.bat/.com 顺序、PATHEXT 自定义项、--all 全列、未找到 exit 1、输出为书写形式(不解析符号链接)。
- [ ] 实现;`cargo test -p which --locked`;提交。

## Task 8:whereis GNU 格式与去硬编码(TDD)

Files:`crates/whereis/{cli.rs,main.rs,tests/behavior.rs}`。

- [ ] 失败测试:输出 `cmd: path...`、未找到仅 `cmd:`、-b/-m/-s 过滤、命中 exit 0 / 全未命中 exit 1、不再含 D:\develop\rust-tools。
- [ ] 实现;`cargo test -p whereis --locked`;提交。

## Task 9:seq -w 负数核对(视结果)

Files:`crates/seq/tests/cli.rs`。

- [ ] 对照 GNU `seq -w -3 0`(-3/-2/-1/00)补行为测试;不一致才改实现。
- [ ] `cargo test -p seq --locked`;提交。

## Task 10:差分矩阵与文档收尾

Files:`scripts/difftest.ps1`、`docs/compatibility{,-zh}.md`、`docs/development{,-zh}.md`、`CHANGELOG.md`、`.superpowers/sdd/progress.md`。

- [ ] difftest 新增:echo -e 形态、printf 格式(宽度/精度/复用/%b)、cmp、factor、expand、rev 用例;跑一轮确认 pass/已知豁免。
- [ ] compatibility 双语:已覆盖命令状态改"Smoke + behavior tests";printf/sleep/echo/which/whereis 行更新差异说明。
- [ ] CHANGELOG Unreleased 汇总;台账 Round 17 节回填。
- [ ] `pwsh -File scripts/build.ps1 -Task Verify` 全绿;提交;合并 main(本地)。
