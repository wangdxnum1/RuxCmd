# 贡献规范

请先阅读 [开发说明](docs/development.md) 和 [兼容性说明](docs/compatibility.md)。

## 新增命令

1. 在 `crates/<命令名>/` 创建 Rust package。根 workspace 的 `crates/*` 会自动纳入；二进制名称必须唯一。
2. 定义参数与行为，记录 Windows 与 Linux 的差异。不要依赖个人机器路径。
3. 在本 crate 的 `tests/` 中添加行为测试；涉及文件时使用独立临时目录，避免真实用户文件、系统进程或外网依赖。
4. 在 `scripts/smoke-cases.json` 增加有界策略。通常使用 `--help`，但须确认解析会在任何副作用之前退出。`true`、`false` 直接断言退出码。
5. 更新兼容性表和 CHANGELOG。不要仅凭能编译将功能标记为已验证。
6. 更新根 Cargo.lock 并提交；CI 和发布使用 `--locked`。

## 提交与 PR

按变化目的组织提交。功能修复附复现输入与回归测试，纯格式化避免与功能重写混合。保持现有依赖版本范围；升级依赖单独说明理由。

提交前执行 README 的 fmt、check、Clippy 和 test 命令。涉及发行脚本时运行脚本测试、完整打包并检查解压后的产物。Clippy 历史 warning 暂不统一提升为错误；不要批量用 allow 隐藏问题。

PR 说明具体行为变化、验证命令与结果、未验证的平台或条件。无关日志和个人配置不进入 PR。

## 发行

维护者更新 `workspace.metadata.ruxcmd.version` 和 CHANGELOG，创建对应 `v<版本>` tag 后，在 GitHub Actions 手动运行 Release。workflow 输出 ZIP 和 `.sha256` artifact，审核后由维护者决定发布渠道。
