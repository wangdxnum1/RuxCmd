# Round 16 设计草案：高频命令的 NUL 分隔支持

日期：2026-10-25。基线：fe136eb（Round 15 收尾，125 个命令；853 Rust tests；GNU 差分 362 pass / 0 fail / 31 skip；远端 CI 40162715091 成功）。

状态：维护者于 2026-10-25 确认“按推荐方案直接实现，技术细节由你决定”。采用下述方案 2，执行者 inline 实现、最终一次独立审查。维护者此前确认将 `-z` 独立为下一轮。

## 目标与范围

本轮补齐 15 个已有命令的 NUL 选项，不新增命令或依赖包版本，命令数仍为 125。

| 类别 | 命令 | GNU 选项与语义 |
| --- | --- | --- |
| 记录处理（8 个） | sort、uniq、cut、paste、comm、shuf、head、tail | `-z/--zero-terminated`：输入记录以 NUL 分隔；输出记录以 NUL 分隔。记录内 LF、CR、非 UTF-8 字节保留。 |
| 路径输出（2 个） | basename、dirname | `-z/--zero`：只改变输出终止符，不从 stdin 读取 NUL 记录；操作数规则保持不变。 |
| 校验输出（5 个） | md5sum、sha1sum、sha256sum、sha512sum、cksum | `-z/--zero`：摘要/CRC 内容不变，输出末尾用 NUL；文件名不转义。四种摘要的 `-z` 与 `-c` 互斥，不将 NUL 输出当作可校验清单。 |

边界：本轮不扩展 join、grep、numfmt、readlink、realpath、du 的 NUL 选项；不引入 sort 外排、tail 环形缓冲或新随机算法；不合并 main、发新版或上架 winget。

## 实现方案与取舍

1. 每个命令分别增加 `read_until(0)`：改动容易定位，但容易重复 CRLF、空记录和末记录错误。
2. **推荐：扩展 ruxcmd-core 的 ByteLines 为可选择记录分隔符。** 保留 `ByteLines::new` 的既有换行/CRLF 行为；增加 `with_separator(reader, separator)`，NUL 模式仅删除 NUL，不删除 CR/LF。sort/cut 复用现有依赖，其他需要读取记录的命令按需引用现有 core。head/tail 保留自己的原始字节截取和 follow 路径，仅传入记录分隔符。无全局重写。
3. 全面统一记录读取、比较、输出及校验引擎：长期重复更少，但会扩大默认模式回归面，当前需求不足以支持这一规模。

方案 2 已由维护者确认；具体 API 和测试组织由执行者决定。

## 必须保持的行为

- 未传 `-z` 时保持既有正常行为。记录处理改为字节安全时，非 UTF-8 输入应保留原始字节，不能因 lossy 字符串比较而把不同字节误判相同；本轮验证发现的真实错误（包括默认模式共用的标题分隔符）须修正并记录，锁定错误行为的旧测试更新为已实测的 GNU 行为。
- 空输入不产生输入记录；连续两个 NUL 表示两个空记录；末尾 NUL 不额外制造第三个空记录。输出遵循具体命令语义：paste 串行模式即使输入文件为空，也为该操作数输出一个终止符（`paste -z -s` 空 stdin 输出一个 NUL）。
- sort/uniq/cut/paste/comm/shuf 对最后一个未终止记录按各自 GNU 规则补终止符；head/tail 保留所选数据原有终止状态，不凭空补 NUL。
- `head/tail -z -c N` 仍按字节截取；文件标题和标题间空行仍使用 LF。tail 的 `+N` 以 NUL 记录计数；`-f` 输出原始追加字节，不等待完整 NUL 记录。
- paste 的记录分隔符与 `-d` 字段分隔符独立，`-z -d '\n'` 在 NUL 记录内部插入 LF；默认 TAB 不改成 NUL。
- paste 的 `-d` 支持 GNU `\n`、`\t`、`\\` 和 `\0`（此处表示不插入字段分隔符）；重复 `-` 操作数从同一个 stdin 缓冲区轮流读取，不能由独立 BufReader 各自预读。
- uniq/comm 保留整条原始记录，比较键和输出数据分离；不要通过 UTF-8 有损视图合并不同原始记录。按 GNU 8.32 实测和 field_sep 源码，uniq `-f` 与 sort 默认字段以 space/TAB/LF 划分，CR 不属于字段空白；这些字节仍保留在原始记录输出中。跳过字段后比较原始后缀，不折叠空白。comm 隐藏列后压缩剩余列的 TAB 前缀。
- cut 的 `-b/-c/-f` 均保留原始输出字节；有效 UTF-8 的 `-c` 维持字符选择行为，非法字节不经替换字符重编码。sort 默认比较和唯一性不能以相同有损替换字符合并不同原始字节。
- shuf 保持既有随机源近似语义，验证记录多重集合和固定源可复现性，不要求与 GNU 使用相同随机排列。
- 四种摘要 `--tag -z` 仍用对应 BSD tag，只改变终止符并关闭文件名转义；`-z -c` 必须报错退出 1，不能静默校验零个文件后成功。
- 接触 md5sum/sha256sum 时同步修正与 Round 15 新摘要实现不一致的清单 stdin、精确文件名、读取错误传播及 FAILED 输出流，使用同类回归测试；不增加未声明的 checksum 选项。

## 验证与交付

- 先写字节级行为断言并观察失败，再实现。覆盖短/长选项、空输入、连续 NUL、记录内 CR/LF、非 UTF-8、末记录未终止及既有参数组合；follow 用有界进程测试。
- difftest 保留旧矩阵，新增独立的 NUL 形态，不把 NUL 输入混入所有旧用例；comm 使用真正按 NUL 记录排序的参照输入。
- 摘要 `--tag -z` 使用相对文件名进行实际 GNU 比较，避免把新行为整体纳入既有 MSYS 生成格式豁免。随机 shuf 用行为测试及可确定的 GNU 用例验证。
- 本机 Git GNU cksum 为 coreutils 8.32，不声明 `-z`；对其新增 GNU 用例按具体选项能力跳过并报告，CRC NUL 输出仍用已知向量和行为测试验证。GNU 文档当前支持该公共输出选项。能力探测必须确认参照 `--help` 成功，DLL 初始化/启动失败必须报错，不能当作“不支持选项”而跳过。
- 保持 125 项冒烟策略与清单一致；更新 compatibility 双语、CHANGELOG、计划及进度台账。
- 完整 Verify、完整 GNU 差分、本地候选 ZIP、一次独立审查；修复重要问题后提交并推送新功能分支。远端 CI 成功后交付，不合并 main 或发布版本。

## 参考

- [GNU sort](https://www.gnu.org/software/coreutils/manual/html_node/sort-invocation.html)
- [GNU uniq](https://www.gnu.org/software/coreutils/manual/html_node/uniq-invocation.html)
- [GNU checksum 公共选项](https://www.gnu.org/software/coreutils/manual/html_node/cksum-common-options.html)
- [GNU 8.32 field_sep（isblank 或 LF）](https://github.com/coreutils/coreutils/blob/v8.32/src/system.h#L145-L150)
- 本机 Git for Windows GNU `--help` 及有界字节探测为当前可执行参照，区域设置固定 C；启动参数沿用 Round 15 的逐参数引用规则。

独立 builder check 已完成：无需要维护者另行决定的技术阻塞；paste 转义/共享 stdin、uniq 字段跳过和 cut 非 UTF-8 输出的修正已纳入上述要求。
