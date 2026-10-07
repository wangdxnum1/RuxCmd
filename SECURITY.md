# Security Policy

**English** | [简体中文](#简体中文)

## Supported versions

RuxCmd is pre-1.0 and under active development. Only the latest tagged release
and the current `main` branch receive security fixes. Suite and command
versions can differ; check `workspace.metadata.ruxcmd.version` in `Cargo.toml`
and the [CHANGELOG](CHANGELOG.md).

## How to report a vulnerability

Please do **not** open a public issue for an exploitable security problem.

1. Prefer GitHub **private vulnerability reporting**:
   <https://github.com/wangdxnum1/RuxCmd/security/advisories/new>
2. If that is unavailable, contact the maintainer directly and ask for a
   private channel before posting details.

Include the command name, exact arguments, expected versus actual behavior, the
suite or commit version, and a minimal reproduction. Do not include private
data in any material you share.

You can expect an initial response within a few days. Once a fix is confirmed,
credit is given by default unless you ask otherwise, and a fix ships in the
next patch release with a CHANGELOG entry.

## Scope

In scope:

- The 111 RuxCmd commands: data loss outside the documented behavior of a
  command (for example `rm` deleting files not matched by its arguments),
  privilege escalation, arbitrary code execution through crafted file names,
  arguments, or file contents.
- The build, verification, and packaging scripts (`scripts/`) and the GitHub
  Actions workflows: anything that would make a released ZIP differ from what
  the committed source and lockfile build.

Out of scope:

- The intentionally documented Windows/POSIX behavioral differences listed in
  the compatibility documentation.
- Social engineering of end users, attacks requiring local write access to the
  repository or the build cache, and reports about automated scanners without a
  demonstrated impact.

## Safe handling of destructive commands

Commands such as `rm`, `shred`, `mv`, and `cp` are destructive by design.
Behavior questions about them are normal bug reports, not security reports —
open a regular issue unless you can cause damage beyond what the arguments
request.

---

## 简体中文

## 支持的版本

RuxCmd 处于 1.0 之前的活跃开发阶段,只有最新 tag 发布版本和当前 `main` 分支会收到安全修复。套件版本与单命令版本可能不同,以 `Cargo.toml` 中 `workspace.metadata.ruxcmd.version` 和 [CHANGELOG](CHANGELOG.md) 为准。

## 漏洞报告方式

**请不要为可利用的安全问题公开提 issue。**

1. 首选 GitHub 私密漏洞报告:<https://github.com/wangdxnum1/RuxCmd/security/advisories/new>
2. 若不可用,先私下联系维护者约定私密渠道,再提供细节。

请包含:命令名、完整参数、预期与实际行为、套件或提交版本,以及最小复现步骤。不要在材料中包含私密数据。确认后默认会在修复说明中署名,除非你要求匿名;修复随下一个补丁版本发布并记录在 CHANGELOG。

## 范围

**在范围内:**

- 111 个命令:超出命令文档化行为之外的数据破坏(例如 `rm` 删除了参数未匹配的文件)、提权、通过构造文件名/参数/文件内容实现的任意代码执行。
- 构建、验证、打包脚本(`scripts/`)与 GitHub Actions workflow:任何会导致发布 ZIP 与已提交源码和 lockfile 构建结果不一致的问题。

**不在范围内:**

- 兼容性文档中已明确记录的 Windows/POSIX 行为差异。
- 针对最终用户的社会工程、需要本地写权限攻击仓库或构建缓存的场景,以及没有演示出实际影响的扫描器批量报告。

## 破坏性命令的安全处理

`rm`、`shred`、`mv`、`cp` 等命令天生具有破坏性。关于它们的行为疑问属于普通 bug 报告——除非你能造成超出参数请求范围的破坏,否则请直接开普通 issue。
