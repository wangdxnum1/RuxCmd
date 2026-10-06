# Changelog

## Unreleased

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
