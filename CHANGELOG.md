# Changelog

## Unreleased

- 完成全部 111 个命令的源码级代码审查(报告 docs/command-review-2026-10-07.md):50 个命令存在 P0 级缺陷,报告给出逐命令的功能/移植完整性/技术评估与 P0-P2 分级。
- 制定第 11 轮 P0 缺陷修复设计与计划(specs/2026-10-07-p0-remediation-design.md、plans/2026-10-07-p0-remediation.md):安全(zip slip/命令注入/自递归)、数据损坏、panic、伪造数据清除四大类,48 个任务。
- 新增全仓库 MIT 许可证（LICENSE）、贡献者行为准则（CODE_OF_CONDUCT.md）与安全策略（SECURITY.md）。
- 根 Cargo.toml 增加 `[workspace.package]`，113 个 crate 统一声明 license/repository 元数据。
- 贡献规范、开发说明、命令兼容性表及脚本使用说明提供中英双语版本，中文版使用 `-zh.md` 后缀；两份 README 更新徽章、文档链接与许可证章节。
- 清理迁移遗留的根目录构建产物（游离 `ls/` 目录、`build-output.log`、`rust.html`）。
- 用 PowerShell 构建入口替代 build-all.bat，支持任务、单包、Debug/Release、离线、并行数及只读预览。
- 增加本地与 GitHub 发布入口，复用清单、校验和及解压验证，拒绝冲突 tag 和覆盖已有 Release。
- CI 与 Release workflow 复用统一入口，补充入口失败路径测试及中英文使用说明。

## 0.1.0 — 基础设施基线

- 将 111 个 Windows 命令归入根 Cargo workspace，固定工具链并统一依赖锁定。
- 增加代表性行为测试及每个命令的发布冒烟策略。
- 增加带源码信息、PE 依赖记录、SHA-256 及解压验证的 ZIP 打包流程。
- 增加 Windows CI、手动发行 artifact workflow 及贡献文档。
- 修复 cp/mv 源文件参数声明导致的 debug 启动断言，以及 grep 普通模式未匹配时的退出码。
- 将 csplit 的溢出边界检查改为 checked_add，消除 Clippy 阻断。

此版本建立发布基线；不代表所有命令已经完整兼容 GNU/POSIX 行为。
