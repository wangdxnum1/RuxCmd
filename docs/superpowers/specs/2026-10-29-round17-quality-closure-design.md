# Round 17 质量收口设计(行为测试覆盖 + P2 清偿)

日期:2026-10-29。基线:main(0.3.2 之后)。

## 背景与目标

0.3.2 之后验证基础设施成熟(902 个 Rust 测试、125 冒烟、GNU 差分、版本资源门禁),但覆盖深度不均:

1. 兼容性表中约 31 个命令"仅冒烟、无行为验证"。重新盘点后,真实缺口分两类:
   - `tests/` 完全缺失:clear、cmp、expand、factor、mkdir、mkfifo、reset、rev、shred、uname、whereis、who、whoami、zcat;
   - 仅有 `tests/cli.rs`(参数解析测试,无行为断言):df、free、iostat、kill、mpstat、pgrep、pkill、readlink、realpath、sar、seq、time、w、which。
2. `docs/command-review-2026-10-07.md` 的 P2 项在 Round 12 被明确搁置;经核对仍成立且值得本轮修复的见下文。

目标:为上述命令补代表性行为测试(确定性命令用精确断言,系统类命令用有界断言),并清偿仍然成立的小型 P2 缺陷;同步兼容性表与差分矩阵。命令数保持 125。

## P2 修复范围(经 GNU 实测核实)

以下参照行为均于 2026-10-29 用 Git Bash 附带 GNU coreutils 实测确认:

### printf(最大项)
GNU 行为(实测):
- 宽度/精度/标志:`%5d`→`   42`、`%-6s`→`ab    `、`%05.1f`→`003.1`、`%5.2s`→`   ab`;标志支持 `-`(左对齐)、`0`(补零)、`+`、空格(正数前空格)、`#`(o/x 前缀、f 保留小数点)。
- 转换:`d/i`(i64)、`u`(负数回绕为 u64,`%u -1`→`18446744073709551615`)、`o/x/X`、`f`、`e/E`(指数至少 2 位带符号,`1.234500e+03`)、`g/G`(有效数字式,去尾零,`0.000012`→`1.2e-05`)、`c`(取首字符)、`s`、`%b`(参数内转义展开,`\c` 截断后续输出)、`%q`(POSIX shell 引用扩展)、`%%`。
- 格式复用:格式串含转换指令时,重复消费剩余参数(`printf '%s\n' a b c` → 三行);格式串无转换指令时只输出一次并 stderr 警告 `warning: ignoring excess arguments, starting with '1'`。
- 缺参:`%s`→空、数值→`0`、`%c`→空;非法数值参数 → stderr 报错、该处输出 `0`、最终 exit 1(沿用现有语义)。
- 未知指令(如 `%y`)→ stderr `printf: %y: invalid conversion specification`,exit 1(替换现有"原样输出"行为)。
- 格式串转义补全:`\a \b \f \v \' \? \e`(\e 为 GNU 扩展)。
- 实现:自写格式化器(不映射 Rust format! 语法,空格标志与 %g 需要手工控制);f64 遵循 shortest-repr 精度格式化 + 手工指数补位。

### sleep
GNU 行为(实测):多操作数求和(`sleep 0.5 0.5`);后缀仅小写 `s/m/h/d`,`sleep 1S` → `sleep: invalid time interval '1S'` exit 1(LC_ALL=C 下 ASCII 引号);`sleep -1` 按未知选项报错 exit 1;`sleep 0` 合法。移除现有非 GNU 的 `-v` 短选项,保留 `--version`。

### echo
GNU 行为(实测):未知选项按操作数输出(`echo -x` → `-x`);`--` 不特殊,也按操作数输出;选项簇 `-ne`/`-nE` 生效;选项解析在首个非选项参数处停止,其后 `-x` 一律为操作数。`-e` 转义补全:`\a \b \c`(截断输出且不加换行)`\e \f \v \\ \0NNN \xHH`,保留现有 `\n \r \t \"`。实现改为手工前导选项扫描,不再用 clap(echo 的参数语法 clap 表达不了 GNU 语义)。

### which
- 扩展名探测改用 PATHEXT(缺省 `.com/.exe/.bat/.cmd` 顺序在前,追加用户 PATHEXT 去重);保留无扩展名精确匹配优先。
- 输出改为 `PATH 条目 + 文件名` 的书写形式(不 canonicalize,不解析符号链接),与 GNU which/Windows where 一致;继续用 `ruxcmd_core::paths::display` 净化。`--all` 行为不变。

### whereis
- 删除硬编码 `D:\develop\rust-tools` 搜索路径(兼容性表标注 "a dedicated fix is pending")。
- 输出对齐 GNU:`cmd: path1 path2`(一行、空格分隔);未找到只输出 `cmd:`,不再输出 "not found"。
- 新增 `-b/-m/-s` 分区过滤(二进制/手册/源码;Windows 上 -m/-s 常为空,如实输出空);无匹配时 exit 1,任一命令有匹配则 exit 0(util-linux 语义)。
- 二进制搜索范围:PATH 各条目 + 系统目录(System32 等通过 PATH 已覆盖),不做硬编码盘符路径。

### seq -w 负数(核对后决定)
GNU(实测):`seq -w -3 0` → `-3 -2 -1 00`(负号计入宽度,正数补前导零)。核对现有实现,若一致仅补行为测试,不一致则修。

## 明确不做(记录为遗留)

- 审查报告中的大型重写:sed/awk 正则解释器、find -exec、ls 布局类 P2(total/对齐/-k)、stat -L/--printf、diff c 合并——另行专项。
- printf `%N$` 参数位引用(glibc 扩展,GNU coreutils 不支持)。
- clear/reset 仅做有界断言(ANSI 序列存在性),不做终端行为自动化。

## 行为测试设计

- 确定性命令(精确/字节断言):cmp、expand、factor、hexdump、mkdir、rev、shred、strings、zcat、printf、sleep、echo、which、whereis、seq。
- 有界断言(数值范围/表头/退出码,参照 top 已有做法):df、free、iostat、mpstat、sar、time、uname、whoami、who、clear、reset、mkfifo。
- 进程类(kill/pgrep/pkill):测试内 spawn `cmd /c ping -n N 127.0.0.1 >nul` 作为长命子进程,断言后确保清理(deadline 内 TerminateProcess),不依赖外部窗口(0.3.1 教训:不启动真实窗口)。
- readlink/realpath 符号链接路径:复用 ln 现有测试的创建方式;创建失败(无特权)时打印提示并跳过,不算失败。
- 差分矩阵新增确定性用例:cmp、factor、expand、rev、echo -e、printf 新格式(空/非 UTF-8/无尾换行形态沿用现有 shape 机制)。

## 验收

1. 上述命令均有 `tests/` 行为测试并在 CI Verify 中运行;
2. P2 修复项先失败测试后实现,全部转绿;
3. `pwsh -File scripts/build.ps1 -Task Verify` 全绿(格式/检查/Clippy/测试/发布编译/冒烟/行为/脚本失败路径);
4. GNU 差分新增用例通过或明确豁免;
5. compatibility 双语表逐条更新验证状态与差异说明;CHANGELOG Unreleased 与台账回填。
