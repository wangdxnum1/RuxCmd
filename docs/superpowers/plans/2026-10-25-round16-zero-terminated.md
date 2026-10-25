# Round 16 NUL 分隔支持实施计划（草案）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans for inline execution, or superpowers:subagent-driven-development if the maintainer explicitly selects delegation. Steps use checkbox syntax for tracking.

**Goal:** 在 15 个现有命令中补齐 GNU 风格 NUL 记录/输出支持，保持 125 个命令的默认行为和发布验证。

**Architecture:** 扩展现有 ByteLines 的可选分隔符；head/tail 保留原始字节区间和 follow 实现；路径与校验工具仅调整输出终止符。避免全面改造命令框架。

**Tech Stack:** Rust 1.99.0、clap、既有 ruxcmd-core、assert_cmd/tempfile、PowerShell GNU difftest。

**Spec:** `docs/superpowers/specs/2026-10-25-round16-zero-terminated-design.md`。此草案须经维护者确认或明确要求直接执行后实施。

## Global Constraints

- 保留现有 checkout，在 codex/round16-zero-terminated 分支执行，基线 fe136eb。
- 不新增命令或依赖包版本；命令数 125；先行为测试失败、后实现；Cargo 检查和测试使用 --locked。新增现有 core 引用时仅显式刷新依赖边，再恢复 --locked。
- 默认换行模式通过既有测试；NUL 模式不剥 CR/LF，不经 UTF-8 有损重编码输出，不把不同非法 UTF-8 记录误判相同。
- 不合并 main 或发布新版本；本地候选包使用新目录，禁止覆盖旧包。

## Review Focus

- 记录内 CR/LF 与非 UTF-8：每个记录处理任务提供字节断言；uniq/comm 另外验证不同非法字节不合并。
- 末记录与空记录：core、head/tail、paste 分别验证是否补终止符，不能全局统一成一种规则。
- tail follow 分段追加：追加时先有内容后有 NUL，必须原样及时输出；有界测试并清理本次子进程。
- 校验 `-z -c` 与文件名转义：每种摘要测试错误退出及 NUL 下原始反斜杠/空格文件名，确保不修改被校验内容。
- GNU 参照能力和随机性：只跳过不支持具体选项的参照用例；摘要 tag-zero 实际比对；shuf 用多重集合及确定用例，不能豁免全部新增行为。

## Task 1：共享记录分隔符

Files：`crates/ruxcmd-core/src/lines.rs`。

- [ ] 添加 with_separator(reader, u8) 的单元测试：空流、连续 NUL、含 CR/LF、未终止末记录、不同非法字节及读取错误。
- [ ] `cargo test -p ruxcmd-core --locked`，确认新增 API/行为测试先失败。
- [ ] 保留 new 的 LF/CRLF 规则，NUL 路径仅剥终止 NUL；迭代器遇错后结束。
- [ ] 单测与 doc tests 通过，提交 core 改动。

## Task 2：sort 与 cut

Files：两 crate 的 cli/main、现有 tests 或新增 tests/zero_terminated.rs。

- [ ] 写 -z 和 --zero-terminated 字节测试，覆盖 sort -u/-r/-c/-n、cut -b/-c/-f/-s/-d、记录内换行与空记录，观察失败。
- [ ] 通过 core 选择输入分隔符并使用对应输出终止符；sort 字段按 space/TAB 空白段定位，LF/CR 是记录内容；默认比较/去重保留原始字节区分。cut -b/-c/-f 不以有损字符串重建输出，有效 UTF-8 -c 仍按字符选择。
- [ ] 运行两包全部测试；新增 GNU 确定用例先验证，再提交。

## Task 3：uniq、comm、paste、shuf

Files：四 crate 的 cli/main/tests/Cargo.toml；Cargo.lock 仅现有 core 引用。

- [ ] 先增加 NUL 行为失败测试；uniq -c/-d/-u/-i/-f，comm -1/-2/-3，paste 并行/串行/自定义分隔符/重复 stdin（串行空操作数仍输出一个 NUL），shuf -n/-i/-o/固定随机源。
- [ ] 记录保留原始字节，必要时使用 core；uniq -f 比较跳过 ASCII space/TAB 字段后的原始后缀，comm 按可见列重建 TAB 前缀；paste 解码 \n/\t/\\/\0，重复 - 操作数共享单个 stdin 缓冲。非法 UTF-8 不被有损转换吞掉，错误退出而非静默截断。
- [ ] shuf -z 输出记录多重集合一致，-i 的数值项以 NUL 结尾；不要求 GNU 同排列。
- [ ] 运行四包全测与各自确定 GNU 用例，再按逻辑提交。

## Task 4：head 与 tail

Files：两 crate 的 cli/main/tests。

- [ ] 先写 -z 默认/-n/+N、-c 忽略分隔符、未终止末记录、多文件标题和有界 tail -f 分段追加测试，观察失败。
- [ ] 将 NUL 分隔符传入现有读取/偏移函数；字节路径不变；标题仍 LF，输出截取数据不补 NUL。
- [ ] 运行两包全部测试及 GNU 对照，再提交。

## Task 5：basename 与 dirname

Files：两 crate 的 cli/main/tests。

- [ ] 先写 -z/--zero、多操作数、basename 后缀移除的输出字节测试，观察失败。
- [ ] 输出每个结果时选择 LF/NUL；不更改路径解析和操作数规则。
- [ ] 两包全测与 GNU 确定对照通过后提交。

## Task 6：五种校验输出

Files：md5sum、sha1sum、sha256sum、sha512sum、cksum 的 cli/main/tests。

- [ ] 先写五包 -z/--zero 输出字节测试：已知摘要/CRC、stdin/显式 -/文件、多文件、NUL 不转义；四种摘要另测 --tag -z 和 -z -c 拒绝。
- [ ] md5sum/sha256sum 先补裸 -c 与 -c -、空清单、精确空格文件名、读取错误传播与失败 stdout 的回归测试，复现与新摘要实现的差异。
- [ ] 仅改变输出终止符与 zero 模式的文件名转义；同步修复旧摘要同类错误，不新增其他 GNU 选项。
- [ ] 五包全测；四种摘要用 tag-zero GNU 对照；cksum 若参照不支持该选项，单列具体能力跳过并保留已知 CRC 向量测试。通过后提交。

## Task 7：集成、文档与交付

Files：scripts/difftest.ps1、docs/compatibility 双语、CHANGELOG、`.superpowers/sdd/progress.md`、本 spec/plan。

- [ ] 新增 nul/nul-notrail/nul-binary 形态和专属矩阵；不要改变旧用例的形态集合；comm 输入按 NUL 记录排序；旧豁免不得覆盖新 tag-zero 用例。
- [ ] 补具体 GNU 选项能力探测，先确认参照 --help 成功，仅对确实缺少选项的参照计 skip；启动失败和缺失 RuxCmd 二进制仍报错。
- [ ] 双语表同步 long 名、NUL 语义与默认模式限制；更新台账和 CHANGELOG。
- [ ] 完整 `scripts/build.ps1 -Task Verify`、完整 GNU 差分；独立审查核心记录边界和跨选项行为，修复重要问题并补失败先行的测试。
- [ ] 在新目录本地打包，验证 DLL/manifest/checksum/解压冒烟；记录候选包状态和校验和。
- [ ] 提交并推送当前功能分支，确认 CI 通过，交付结果；不合并 main 或发布版本。
