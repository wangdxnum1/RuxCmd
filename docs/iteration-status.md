# 迭代状态

项目方向：优先服务 Windows 日常命令行与脚本使用。后续先稳定发布基线，再围绕文件筛选、批量处理、日志分析和结果校验补齐完整流程。

## 定位规则

以套件版本、源码提交和验证结果定位进度。历史台账的 Round 编号存在重复，只作为历史标题；不重写或重新编号已有记录。

- 当前代码与兼容性范围：[CHANGELOG](../CHANGELOG.md)、[兼容性表](compatibility-zh.md)。
- 历史执行记录：[旧台账](../.superpowers/sdd/progress.md)。
- 当前迭代：[设计](superpowers/specs/2026-11-07-release-baseline-design.md)、[实施计划](superpowers/plans/2026-11-07-release-baseline.md)。

## 已核验的发布基线

| 版本 | 源码提交 | 本轮核验（2026-11-07） |
| --- | --- | --- |
| 0.4.9 | `bf50d0e87019354ef9cb91fd153ca0bd0304f3e3` | Release 成功；ZIP SHA256 与清单匹配，manifest 的 sourceCommit 与 tag 一致且 dirty=false；发布包 GNU 差分 942 通过、0 失败、39 跳过 |

基线 main 提交 `2d79cfb9a8cfca050cf872e57a933adab1f23691` 只在上述源码后追加台账。本地全量 Verify 通过：1613 项 Rust 测试通过、19 项忽略，142 项冒烟检查；独立兼容表检查发现 cut/diff/mktemp 三处选项漏列。远端 CI 在兼容表步骤失败，Release 当时没有此检查。

- [0.4.9 Release](https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.4.9)
- [Release 运行](https://github.com/wangdxnum1/RuxCmd/actions/runs/42317376720)
- [基线 CI 运行](https://github.com/wangdxnum1/RuxCmd/actions/runs/42317436193)

## 当前迭代：发布基线稳定化（未发布）

继续沿用套件版本 0.4.9，改动记在 Unreleased。目标是修正文档漏项，让 Verify、CI 和 Release 使用同一兼容性门禁，并明确检查本次构建目录。完成结果见下方验证记录；新的远端结果须在合入后单独核验。

本轮本地验证（2026-11-07）：

- 全量 `build.ps1 -Task Verify -Offline` 通过（180.9 秒）：1613 项 Rust 测试通过、19 项忽略，142 项版本资源与冒烟检查通过。
- 兼容表门禁已在上述全量流程内执行：142 行、127 项选项检查通过、15 项按明确原因跳过。
- 入口测试 21 项通过，包含新增 Release/Debug、自定义输出目录、Offline 传递和兼容表失败即停止的子进程回归。
- PowerShell 与工作流 YAML 解析通过；两份工作流的公共 Verify 入口与依赖缓存策略检查通过。
- 独立只读审查完成，无需修复的问题；审查核对了改动与本地验证日志，未重复执行此前的发布资产核验。
- 当前改动保留在 `codex/stabilize-release-baseline`，尚未提交、推送或发布。远端 CI 的修复结果需在这些改动推送后再核验。
