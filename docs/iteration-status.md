# 迭代状态

项目方向：优先服务 Windows 日常命令行与脚本使用。当前阶段已从「功能补齐」转入「分发与闭环」——先把发布基线稳定化，再围绕分发、反馈回流与承诺冻结补齐闭环。中期方向见[发展方向与见解](roadmap-zh.md)（[English](roadmap.md)）。

## 定位规则

以套件版本、源码提交和验证结果定位进度。历史台账的 Round 编号存在重复，只作为历史标题；不重写或重新编号已有记录。

- 当前代码与兼容性范围：[CHANGELOG](../CHANGELOG.md)、[兼容性表](compatibility-zh.md)。
- 逐轮执行记录：[旧台账](../.superpowers/sdd/progress.md)（Round 26–34 覆盖 0.5.2–0.5.10）。
- 发布基线稳定化的原始设计与计划：[设计](superpowers/specs/2026-11-07-release-baseline-design.md)、[实施计划](superpowers/plans/2026-11-07-release-baseline.md)。

## 当前状态（2026-11-07）

| 项目 | 值 |
| --- | --- |
| 套件版本 | **0.5.11**（[Cargo.toml](../Cargo.toml) `workspace.metadata.ruxcmd.version`） |
| 最新发布 | **v0.5.11** 已成功发布（Release run `42384943635`），文档与进度闭环收口版本，四资产（ZIP、setup EXE 及两个 `.sha256`）；发布说明由 CHANGELOG 对应节自动生成 |
| HEAD | `a8fb923c6c7725a3829cb1b9f0a572d081a49aaa` = `main` = `origin/main` = tag `v0.5.11`（2026-11-07 22:52）；发布后另有清单哈希同步提交 |
| 命令 / 包 | 145 个 crate → 142 个独立可执行文件 |
| 质量 | 约 1740 项 Rust 测试；两个 push CI（42383742043 / 42383815911）与 Release 全量 Verify 全绿（fmt / clippy -D warnings / 测试 / release 编译 / compat-check / check-manifests / 冒烟 / 行为检查 / 脚本测试 / difftest） |
| 发布产物 | ZIP SHA256 `e90ab189…42226c7045`、setup EXE SHA256 `a7b30d64…1d7965eeb17`；`.zip.sha256` sidecar 与 GitHub asset digest 已独立比对一致，清单哈希同步至 `e90ab189…` |

当日（2026-11-07）连续发布 **11 个版本**（v0.5.0 → v0.5.10），随后以 0.5.11 收口文档与进度。主要增量均为 GNU 逐字节对齐与基建：

- 0.5.1–0.5.2：bc `scale`/`ibase`/`obase`、cut `-n`/`--complement`、校验和 `-b/-t`、uniq 输出操作数、truncate 全部调整符与 `--io-blocks`。
- 0.5.3–0.5.4：cp 硬链接/备份、diff `-r` 与 GNU 时间戳；du `--time`/`--time-style`/`--files0-from`/块缩放，抽出 `ruxcmd-core::timefmt`。
- 0.5.5–0.5.7：grep 二进制文件模式、mv 提示流；zip 追加更新语义与百分比、dd EBCDIC 转换；`ruxcmd-core::disk` 基建带动 iostat/sar `-d` 设备报表、nslookup SRV。
- 0.5.8：发布基础设施加固——[scripts/check-manifests.ps1](../scripts/check-manifests.ps1) 纳入 Verify，修复 v0.5.5/v0.5.6 清单 SHA256 断链，并固化「发布后下载 .zip.sha256 核对清单哈希」收尾步骤。
- 0.5.9–0.5.10：find 相对日期与时区后缀（GNU `parse_datetime` 子集）；cp `VERSION_CONTROL` 环境变量、grep 目录操作数计数行。
- 0.5.11：纯文档收口——兼容表 head/mv 过期描述纠偏、README 版本残留修正、迭代状态页重写、新增双语 roadmap、归档 0.4.8 缺口快照。无 CLI 行为变更。

环境与验证约束：本地全量 `build.ps1 -Task Verify` 需要管理员权限、允许子进程脱离 Job 且允许 ICMP 的环境。受限会话中三类测试会失败，均与代码无关：`chmod` 的 junction 测试（`mklink /J` 被拒）、`chown` 的 `SetNamedSecurityInfoW`（缺 `SeTakeOwnershipPrivilege`/`SeRestorePrivilege`，error 5）、`nohup` 的 detached child（Job 禁止脱离，60s 超时）、`traceroute` 的 loopback（`IcmpSendEcho` Request failed）。以 unrestricted / 管理员会话重跑可通过（历史记录 243.1s）；本轮排除这四个包后 `cargo test --workspace --exclude ...` 346 项结果全绿、退出 0。正式发布由 GitHub Actions 在 `windows-2025` runner 上执行同款全量 Verify。逐轮验证与发布记录以[旧台账](../.superpowers/sdd/progress.md)为准。

## 已核验的历史发布基线

| 版本 | 源码提交 | 核验结果 |
| --- | --- | --- |
| 0.4.9 | `bf50d0e87019354ef9cb91fd153ca0bd0304f3e3` | Release 成功；ZIP SHA256 与清单匹配，manifest 的 sourceCommit 与 tag 一致且 dirty=false；发布包 GNU 差分 942 通过、0 失败、39 跳过 |

基线 main 提交 `2d79cfb9a8cfca050cf872e57a933adab1f23691` 只在上述源码后追加台账。本地全量 Verify 通过：1613 项 Rust 测试通过、19 项忽略，142 项冒烟检查；独立兼容表检查发现 cut/diff/mktemp 三处选项漏列。远端 CI 在兼容表步骤失败，Release 当时没有此检查——该缺口已由 0.5.0 的「兼容表校验纳入 Verify」门禁关闭。

- [0.4.9 Release](https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.4.9)
- [Release 运行](https://github.com/wangdxnum1/RuxCmd/actions/runs/42317376720)
- [基线 CI 运行](https://github.com/wangdxnum1/RuxCmd/actions/runs/42317436193)

## 下一阶段（未开工）

来自 [roadmap-zh.md](roadmap-zh.md) 的取舍：**分发是下一阶段的乘数，不是继续堆功能**。0.5.11 已用于文档收口，下列自动化与分发工作顺延：

1. **文档与进度闭环**：0.5.11 完成——兼容表过期描述纠偏、本页重写、方向文档纳入索引、历史缺口快照归档。
2. **v0.5.12 —— 发布自动化收尾**：Release 成功后自动「下载 .zip.sha256 → 回填清单 → check-manifests → 提交推送」；并补上「已发布 asset digest ↔ 清单哈希」的独立比对（当前 check-manifests 只校验哈希格式，结构上抓不到哈希合法但停留在旧版本的事故形态）。基建整合（尺寸后缀/日期/纪元换算统一进 ruxcmd-core）随后按批次推进。
3. **v0.6.0 —— 分发落地**：winget-pkgs PR、公开 scoop bucket、GitHub Pages 文档站；同步补「行为差异报告模板」（把用户反馈直接转成 difftest 用例）与代码签名方案评估（winget 清单已声明安装包未签名、SmartScreen 会警告）。
4. **1.0 —— 承诺冻结**：需配套机器可校验的 CLI 契约快照（`--help`/退出码/关键输出格式的黄金文件进 Verify），让「冻结」成为门禁而非口头承诺。

未开工部分的执行细节以 roadmap 与逐轮台账为准；本文只记录已发生的事实与当前锚点。
