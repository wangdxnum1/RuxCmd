# RuxCmd Infrastructure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans for native execution, or superpowers:subagent-driven-development if the maintainer selects delegation. Steps use checkbox syntax for tracking.

**Goal:** 将 111 个 Windows 命令组织为标准 Cargo workspace，并提供经过验证的测试、CI 和 ZIP 发行流程。

**Architecture:** 根 virtual workspace 管理 crates/*，Cargo 负责构建和依赖锁定。PowerShell 7 脚本负责发布产物验证和归档；GitHub Actions 复用本地命令与脚本。

**Tech Stack:** Rust 1.99.0、Cargo resolver 3、Windows x64 MSVC、PowerShell 7、GitHub Actions。

**Spec:** [基础设施设计](../specs/2026-10-06-project-infrastructure-design.md)

## Global Constraints

- 正式发行包含 111 个独立命令、113 个 package；ps 的产物名为 ps.exe。
- Rust 工具链固定 1.99.0，target 为 x86_64-pc-windows-msvc。
- 套件版本初始为 0.1.0，存储于 workspace.metadata.ruxcmd.version；发行 tag 为 v<版本>。
- 保留现有源码和未提交修改，迁移期间不批量升级依赖、不改变命令调用方式。
- 不恢复或纳入 skill-manager、haozip-ad-killer、rust_test_project。
- 根 Cargo.lock 由 Cargo 生成并提交；验证和发布构建使用 --locked。
- 只保留 ps 现有 MIT 声明，不为整个仓库选择新许可证。
- 远程 CI 未执行时明确报告；第一阶段不自动创建 GitHub Release。

## Review Focus

- 仓库路径含空格或从其他工作目录启动：脚本仍正确定位根目录，产物路径不得靠 shell 字符串拼接。
- ps-bin 与 ps.exe 名称不同：按 Cargo target/artifact 而不是目录名取产物。
- 旧 target/dist 含额外 exe：归档只能包含本次构建确认的 111 个命令。
- 编译、子进程或归档验证失败：退出非零，不遗留被误认为成功的发行包。
- 持续运行或具有系统副作用的命令：使用确认安全的参数、有超时的进程和隔离临时目录。

## 文件职责

- Cargo.toml / Cargo.lock / rust-toolchain.toml：workspace、依赖锁定及工具链。
- crates/<command>/：现有命令源码、测试与本命令样例；ps 三个 crate 独立并列。
- scripts/smoke-test.ps1：从明确二进制目录运行有界策略，缺少策略或失败时退出非零。
- scripts/package.ps1：构建、校验版本与产物、暂存、调用 smoke、生成归档与校验并解压验证。
- scripts/tests/package-tests.ps1：使用临时目录验证打包校验和冒烟错误路径，不修改真实系统。
- .github/workflows/ci.yml / release.yml：常规验证与指定 tag 的手动构建发行 artifact。
- README.md / CONTRIBUTING.md / CHANGELOG.md / docs/development.md / docs/compatibility.md：使用、贡献、开发和真实兼容性信息。

### Task 1: 迁移为可解析的根 workspace

**Files:** Create Cargo.toml、rust-toolchain.toml、crates/；Modify .gitignore、build-all.bat、ps 的三个 manifest；Move 全部正式命令目录及 ps 文档。

**Interfaces:** Produces Cargo metadata：113 个 workspace package、111 个唯一 binary target；根 metadata.ruxcmd.version = "0.1.0"。

- [ ] 记录 git status、当前命令清单及源码/样例文件 SHA-256，保存在本次临时目录，供迁移后比对；确认全部目标路径在仓库内且不存在。旧构建缓存不进入发布源。
- [ ] 将 110 个单 crate 命令整体迁移到 crates/<原名>；将 ps-sys、ps-core、ps-bin 移到 crates/ 下，ps 文档移到 docs/ps/。使用 PowerShell 原生 Move-Item -LiteralPath，逐一验证源和目标绝对路径。
- [ ] 创建 virtual workspace：members = ["crates/*"]、resolver = "3"；转移 ps workspace.dependencies，成员显式保留原 version/edition/rust-version/license。
- [ ] 创建固定工具链文件，取消 Cargo.lock 忽略；保留静态 CRT 配置。build-all.bat 用 %~dp0 定位仓库并运行 locked release 构建，直接传播退出码。
- [ ] 运行 cargo metadata --no-deps --format-version 1，断言 package = 113、binary = 111 且没有重复 bin 名；逐一比较迁移前后源码及样例 SHA-256，允许列出的 manifest 修改。
- [ ] 将本任务所需的迁移及配置变更单独提交；不包含无关日志、历史进度修改或其他新文档。

### Task 2: 建立真实的全量编译基线

**Files:** Create Cargo.lock；Format crates/**/*.rs；必要时修改导致基线失败的具体源码，逐项记录于 docs/development.md。

**Interfaces:** Consumes Task 1 metadata；Produces 可通过 locked check/test/build 的 workspace 和真实 warning 清单。

- [ ] 运行 cargo generate-lockfile，再运行 cargo check --workspace --all-targets --locked，记录实际失败命令或依赖问题。
- [ ] 对编译失败先定位根因。只修复构建必需问题；保留依赖版本范围，并为涉及行为的必要修复补最小复现测试。
- [ ] 运行 cargo fmt --all，复核只改变格式；格式化与必要功能修复分别提交。
- [ ] 运行 cargo fmt --all --check、cargo clippy --workspace --all-targets --locked、cargo test --workspace --all-targets --locked，退出码均为 0；记录现存 warning，不批量添加 allow。
- [ ] 运行 cargo build --workspace --bins --release --target x86_64-pc-windows-msvc --locked，确认 111 个二进制存在；使用 MSVC 的 dumpbin /DEPENDENTS 或可用 PE 工具核查非系统 DLL 依赖。
- [ ] 提交根锁文件与本任务改动。

### Task 3: 行为测试与发布冒烟策略

**Files:** Create crates/{cat,echo,wc,grep,cp,mv,rm,true,false}/tests/cli.rs；Modify 对应 Cargo.toml dev-dependencies；Create scripts/smoke-test.ps1、scripts/tests/package-tests.ps1；保留 crates/ps-bin/tests/cli.rs。

**Interfaces:** smoke-test.ps1 -BinDir <绝对或相对目录> -TimeoutSeconds 10；成功返回 0，失败抛错并返回非零。测试策略集合与 Cargo metadata 的 binary target 集合完全一致。

- [ ] 先添加有实际断言的集成测试：cat 文件/stdin/缺失文件、echo 文本、wc 行数、grep 匹配/未匹配、cp/mv/rm 临时文件变化、true=0/false=1。使用 assert_cmd、predicates 和 tempfile；复用仓库已用的测试依赖版本范围。
- [ ] 逐个阅读所有发布命令的启动与参数解析路径，列出不会修改系统或无限运行的冒烟参数；false/true 采用对应退出码。无法安全运行的例外必须记录原因且仍核对 exe 存在。
- [ ] 实现 smoke-test.ps1：验证覆盖集合、以 ProcessStartInfo 的参数数组启动显式 exe、异步读取 stdout/stderr、等待超时并终止本次子进程；包含命令名、参数及错误输出的失败报告。
- [ ] 在 package-tests.ps1 增加断言：目录含空格时成功；缺失 ps.exe 或策略缺失时失败；false 的预期退出码 1 通过；传入超时测试进程时失败且结束进程。
- [ ] 运行 cargo test --workspace --all-targets --locked 和 pwsh -File scripts/tests/package-tests.ps1，并对当前 release 目录运行 smoke-test.ps1；所有预期正常检查退出码为 0。
- [ ] 提交测试和冒烟入口。

### Task 4: 可追溯且可验证的 ZIP 发行包

**Files:** Create scripts/package.ps1；Extend scripts/tests/package-tests.ps1；Create README.md、CHANGELOG.md、docs/compatibility.md 的发行所需内容。

**Interfaces:** package.ps1 -OutputDir <默认根 dist> [-RequireClean] [-ExpectedTag v0.1.0]。清单 schemaVersion=1、suiteVersion、target、rustcVersion、sourceCommit、dirty、binaries；binaries 项为 name、packageVersion、file、sha256。

- [ ] 实现版本检查：读取 suiteVersion，ExpectedTag 必须等于 v<suiteVersion>；RequireClean 下 git 工作目录有修改则拒绝。常规本地构建必须标记 dirty。
- [ ] 执行固定 target 的 locked release build，使用 --message-format=json-render-diagnostics，将 compiler-artifact 中的 executable 与 workspace binary target 精确对应；检查退出码、缺失、重复和未知产物。
- [ ] 在 OutputDir 下创建唯一暂存目录，复制仅本次确认的 111 个 exe；不递归删除外部目录，不从 dist 或 glob 查找可执行文件。
- [ ] 调用 Task 3 冒烟入口，创建 manifest.json、SHA256SUMS，加入 README、CHANGELOG、compatibility.md。内部校验覆盖 exe 和随包文档，不循环包含校验文件自身。
- [ ] 生成 ruxcmd-0.1.0-x86_64-pc-windows-msvc.zip 和外部 .sha256，文件已存在时明确失败，避免覆盖历史包；失败时清理由本次创建且已经验证属于 OutputDir 的暂存或不完整归档。
- [ ] 解压到独立临时目录，比对完整文件集合、manifest、所有校验值，执行 cat/echo/wc/grep/true/false/ps 代表性验证后报告成功。
- [ ] 扩展 package-tests.ps1：伪造旧额外 exe 不进入 archive；篡改文件校验失败；标签不一致失败；缺失 artifact 失败；已存在 ZIP 不被覆盖；异目录启动和含空格路径通过。
- [ ] 运行脚本测试和一次实际全量打包，核查解压包确实有 111 个 exe；记录 DLL 核查结果和包 SHA-256。
- [ ] 提交脚本与发行所需文档。

### Task 5: CI、贡献流程与收尾验证

**Files:** Create .github/workflows/{ci,release}.yml、.github/ISSUE_TEMPLATE/{bug_report,feature_request}.yml、.github/pull_request_template.md、CONTRIBUTING.md、docs/development.md；Finish README.md、CHANGELOG.md、docs/compatibility.md；Update 迁移后命令 README 中有效构建路径。

**Interfaces:** ci 触发 push/pull_request/workflow_dispatch；release 只接受 workflow_dispatch 的既有 tag，成功上传 ZIP 与 .sha256 artifact，不创建 GitHub Release。

- [ ] 核查 GitHub Actions 官方来源，固定 checkout、缓存和 artifact action 到真实 commit SHA，权限默认 contents: read；所有 run 步骤传播 Cargo/脚本失败。
- [ ] ci Windows job 安装仓库固定工具链，缓存 Cargo，依次 fmt/check/clippy/test、release build、smoke 和脚本测试。release job 校验 tag 并检出干净源码，运行 package.ps1 -RequireClean -ExpectedTag <输入>，上传产物。
- [ ] 写明 MSVC/SDK/PowerShell 7 前置要求、单工具与全量命令、ZIP 使用与 PowerShell alias、如何新增 crate 和冒烟策略、套件与命令版本区别、整体许可证未统一状态。
- [ ] 兼容性表从源码检查生成初始状态，对已自动验证的行为单独标记；Issue/PR 模板包含参数、输入、环境、预期/实际结果与验证证据。
- [ ] 使用可用 YAML parser 解析 workflow/template；静态核对 workflow 调用和脚本参数一致。检查变更和迁移文件集合，确认原有未提交工作保留。
- [ ] 仅在最终源码或配置发生变化时重跑受影响验证，发布涉及变化时重新完整打包；不重复无变化的全量检查。
- [ ] 审阅最终 diff 的迁移正确性、失败路径、测试隔离和包清单；记录已通过步骤与未在 GitHub 执行的限制，提交仅本任务文件。

## 执行交接

建议在本会话原生执行：各任务共享同一 workspace 和构建产物，顺序执行能更容易处理初次全量构建问题。维护者审阅此计划并选择原生或分工执行后开始实现。执行时按任务更新上述复选框，出现影响设计的变化先明确说明并更新设计。
