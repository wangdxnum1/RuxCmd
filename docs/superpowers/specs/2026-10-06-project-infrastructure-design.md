# RuxCmd 基础设施设计

日期：2026-10-06。状态：待维护者审阅。

## 目标与已确认约束

RuxCmd 将 Linux 风格命令移植为原生 Windows 可执行程序。维护者希望按常见的 Rust 项目方式建立项目管理、编译、测试和发布基础设施，允许在有必要时重整现有结构。

第一版发布范围仅包含 Linux 风格命令。skill-manager、haozip-ad-killer 独立维护，rust_test_project 不纳入。重新核查时上述三个目录已不在当前工作目录；不重新创建或恢复它们。

成功标准是：新机器安装文档列出的前置依赖后，可以从任意路径构建全部命令、运行测试，并生成带版本及校验信息的 Windows ZIP 发行包。后续增加一个命令不需要维护多份手工构建列表。

保留命令行为和独立 exe 的调用方式。迁移现有源码、测试样例和文档，不删除已有未提交工作。第一阶段只支持 Windows x64 MSVC；其他架构在实际验证后再加入。

## 方案选择

采用一个根 Cargo virtual workspace，以 Cargo 为构建入口。正式 crate 统一置于 crates/，打包使用 PowerShell 7，CI 使用 GitHub Actions Windows runner。

相比保留 111 个独立项目，根 workspace 统一锁文件、构建目录和标准检查入口，减少依赖重复编译和清单漂移。相比单程序分发多个命令，本方案保留现有调用方式，迁移不需要重写业务代码。

## 目录与 workspace

目标结构：

```text
Cargo.toml
Cargo.lock
rust-toolchain.toml
.cargo/config.toml
.github/workflows/ci.yml
.github/workflows/release.yml
.github/ISSUE_TEMPLATE/
.github/pull_request_template.md
crates/
  cat/
  ls/
  ...
  ps-sys/
  ps-core/
  ps-bin/
scripts/
  package.ps1
  smoke-test.ps1
docs/
  development.md
  compatibility.md
  superpowers/
README.md
CONTRIBUTING.md
CHANGELOG.md
build-all.bat
```

111 个命令对应 113 个正式 package：110 个单 crate 命令，加上 ps 的三个 crate。根 workspace 使用 members = ["crates/*"]，resolver = "3"。根下的独立或实验项目不会被通配符自动纳入。

普通命令整体移动到 crates/<原目录名>/，包括现有样例和 README。ps 的三个 crate 移到 crates/ps-sys、crates/ps-core、crates/ps-bin，其相对 path dependency 保持相邻关系。ps 的命令文档及设计历史归档到 docs/ps/，更新有效开发说明中的构建路径；历史设计文件保留其原有时间与历史描述。

将 ps 根 manifest 的 workspace dependency 定义转移到根 manifest，移除嵌套 workspace。ps 原先继承的 version、edition、rust-version、license 在成员 manifest 显式保留，避免给其他命令强加原来只属于 ps 的元数据。

移动之前记录当前 Git 状态，并核对目标路径不存在。只移动明确的项目内容；旧 target 目录不进入版本控制，也不作为新发布包的产物来源。既有修改随源文件一起保留。改造后的 build-all.bat 为基于自身路径的兼容入口，调用标准 Cargo release 构建并检查退出码，不再维护命令列表或向机器固定目录复制。

## 工具链、依赖与版本

rust-toolchain.toml 固定当前已安装的 Rust 1.99.0，包含 rustfmt、clippy 和 x86_64-pc-windows-msvc target。README 明确 Visual Studio Build Tools 的 MSVC C++ 工具及 Windows SDK 前置要求。第一阶段不宣称已经验证项目的最低 Rust 版本。

提交一个由 Cargo 生成的根 Cargo.lock。取消对 Cargo.lock 的全局忽略；根 workspace 的旧成员锁文件不再作为有效锁文件维护。所有验证和发布构建使用 --locked。

迁移保留各命令已有 edition、package version 和依赖约束。先完成可构建的基线，再逐步统一共享依赖；此次不批量升级 windows/windows-sys/winapi 或改写 Win32 调用。

套件版本独立记录在根 workspace.metadata.ruxcmd.version，初始为 0.1.0。发行标签为 v<套件版本>。现有命令 --version 继续表达各自版本，README 解释两者区别。CHANGELOG 按套件版本记录用户可见变更。

保留既有 MSVC 静态 CRT 配置。发布验证须检查动态依赖；如果仍有非系统 DLL 依赖，应随包附带并记录，或明确构建失败原因，不能仅依据 crt-static 声称便携。

## 标准开发入口

本地和 CI 共用以下命令，均从根目录执行：

```powershell
cargo metadata --no-deps --format-version 1 --locked
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo build --workspace --bins --release --target x86_64-pc-windows-msvc --locked
pwsh -File scripts/package.ps1
```

单命令开发使用 cargo run -p cat -- <参数> 或 cargo test -p cat --locked；ps 使用 -p ps-bin。工作区不需要另造一套构建任务系统。

首次迁移使用 cargo fmt --all 整理现有 Rust 代码，格式化改动与结构迁移、功能修复分别组织，便于审阅。Clippy 以正常退出为第一阶段门槛，保留已有 warning 并记录；不为凑通过批量添加 allow，也不立即将所有历史 warning 升为错误。后续再集中消除并提高门槛。

## 测试与兼容性

运行所有现有 ps 测试，并为代表性命令增加有实际行为断言的集成测试，至少覆盖：

- cat：文件输入、stdin 和缺失文件退出码。
- echo、wc、grep：文本输出、计数、匹配与未匹配退出码。
- cp、mv、rm：仅在临时目录中验证文件变化和失败路径。
- true、false：退出码分别为 0 和 1。
- ps：保留已有命令行与底层测试。

所有新增测试均隔离临时文件，不操作真实系统进程或用户文件，不依赖外网。

scripts/smoke-test.ps1 对发布目录运行验证。为每个发布命令记录有界的验证策略：确认参数解析路径后可以使用 --help；true/false 使用明确退出码断言；对于不支持安全 help 的命令，采用专门参数或记录未运行原因。每个子进程有超时，超时或非预期退出码返回非零状态。命令覆盖策略与 Cargo metadata 的实际二进制集合核对，新增工具缺少策略时检查失败。

docs/compatibility.md 由当前源码证据记录支持的常用参数和 Windows 差异，区分实现、测试通过和未验证状态。不把已有工具列表等同于完整 GNU/POSIX 兼容性承诺。

## 打包与发布

scripts/package.ps1 从自身路径定位根目录，读取根 manifest 的套件版本和 Cargo metadata，执行明确 target 的 locked release 构建。通过 Cargo JSON compiler-artifact 消息获得实际 executable 路径，并核对 workspace 的全部 binary target；不从旧 target 或旧 dist 中模糊匹配 exe。

只打包 111 个发布命令。库 crate 不成为发行文件；ps-bin 的产物名按 target.name 得到 ps.exe。未知产物、重名、缺失二进制或任意构建失败均中止打包。

使用 dist 下本次唯一的暂存目录，避免旧文件残留污染本次发布。默认不删除历史发行包。包结构为：

```text
bin/*.exe
README.md
CHANGELOG.md
compatibility.md
manifest.json
SHA256SUMS
```

manifest.json 记录套件版本、target triple、rustc 版本、源码 commit、工作目录是否有未提交修改，以及每个二进制的名称、package 版本和 SHA-256。清单与内部校验文件不对自身做循环校验；外部 .sha256 文件记录 ZIP 的 SHA-256。

打包前对暂存目录运行冒烟测试，生成 ruxcmd-<版本>-x86_64-pc-windows-msvc.zip 及其 .sha256。归档后重新解压到独立临时目录，核对文件集合与校验值，运行代表性行为验证。PATH 使用说明提示 PowerShell 同名 alias 的优先级，并提供显式 exe 调用示例。

本地包允许有未提交修改，但清单必须标记。正式发布要求来自指定 tag 的干净检出，标签版本与 manifest 版本一致。

## CI 与项目管理

ci.yml 在 push、pull_request 和 workflow_dispatch 上运行。Windows job 安装固定工具链，缓存 Cargo 下载和构建结果，执行上面的格式、check、Clippy、test 步骤。CI 中的冒烟测试遵循相同的有界策略；失败必须使 job 失败。

release.yml 第一阶段只支持 workflow_dispatch，输入已存在的 v<版本> 标签。在干净检出中复用 package.ps1，并上传 ZIP 与校验文件为 workflow artifact。此次不自动创建 GitHub Release、不自动发布包管理器，也不部署任何外部服务。所有第三方 action 固定到经核查的 commit SHA。

README 包含项目定位、Windows 前置依赖、构建与运行、ZIP 使用方式、命令概览和开发文档入口。CONTRIBUTING 描述增加新 crate、增加冒烟策略、测试标准与 PR 验证要求。Issue 模板收集命令参数、输入、预期/实际结果及环境；PR 模板简明记录变化和验证。

许可证保持当前真实状态：保留 ps 已有 MIT 声明，不替整个仓库擅自选择许可证；README 明确整体许可证尚未统一。选择整体许可证属于独立的维护者决策，不阻塞本地基础设施验证。

## 实施顺序与完成条件

1. 迁移正式 crate，合并 workspace，确认 metadata 中有 113 个 package 和 111 个命令。
2. 固定工具链、生成根锁文件，格式化并完成全量 check/build 基线；只修复迁移和基线验证暴露的必要问题。
3. 建立代表性行为测试及完整发布冒烟策略，完成 workspace 测试。
4. 实现并验证打包脚本，解压验证文件集合、校验值和代表性行为。
5. 添加 CI、手动发行 workflow 和项目文档，检查配置语法及本地等价步骤。

完成必须有当前执行证据：metadata 数量、全量构建与测试结果、包内 111 个命令和校验验证。无法运行的远程 CI 明确标为尚未在 GitHub 执行，不将静态检查描述为远程成功。

## 已知边界

Windows shell alias、Win32 权限、文件编码和 GNU 兼容性可能暴露历史行为差异。基础设施首次验证发现的问题应给出复现与影响范围，必要修复保持独立，不能靠跳过失败命令伪造完整发行包。

可复现目标是固定源码、工具链和依赖，并准确记录产物，不在第一阶段承诺不同机器构建 ZIP 字节完全相同。

参考：Cargo workspace https://doc.rust-lang.org/cargo/reference/workspaces.html；Cargo.lock https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html。
