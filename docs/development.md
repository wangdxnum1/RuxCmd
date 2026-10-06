# 开发与构建

## 工程结构

根 virtual workspace 管理 `crates/*`。目前 113 个 package 产出 111 个命令；`ps-sys`、`ps-core` 是库，`ps-bin` 产出 `ps.exe`。Cargo metadata 是构建及发布工具清单的唯一来源，冒烟策略必须与其二进制集合一致。

Edition 2021/2024 和各 crate 原版本保留。根 Cargo.lock 固定整个套件的依赖，resolver 为 3。ps 保留原来声明的 Rust 1.75 元数据，但整个仓库当前只用固定的 1.99.0 验证，不承诺整体 MSRV。

Cargo 命令可在根使用 `--workspace`，也可在单个 crate 目录运行；两者使用根锁文件和 target。默认 debug 产物路径可能受用户 Cargo 配置影响，发布命令显式使用 x86_64-pc-windows-msvc。

## 检查与测试

统一入口为 `scripts/build.ps1`；`-Task Verify` 执行与 CI 相同的完整流程，参数见 [脚本说明](../scripts/README.md)。Rust 集成测试覆盖 cat、echo、wc、grep、cp、mv、rm、true、false 和现有 ps 行为。文件测试使用 tempfile；没有测试依赖联网或终止真实进程。

发布冒烟只运行经过源码检查的帮助入口，以及 true/false 的退出码。每个进程有超时，异步读取输出和错误，超时终止本次进程树。帮助通过说明启动/参数入口能工作，不证明全部业务行为正确。

路径按 PowerShell 当前目录解析，即使执行过 Set-Location 而 .NET 进程目录未同步，也可正确使用相对 BinDir/OutputDir。进程等待、stdin 写入和输出管道共用截止时间；Windows Job Object 负责本次子进程及其后代的生命周期，包括父进程先退出而后代仍持有输出管道的情况。

`scripts/tests/package-tests.ps1` 的 17 个测试检查含空格路径、Set-Location 后的相对路径、参数、超时、继承输出管道的后代、阻塞 stdin、大输出、ps-bin/ps 名称、清单遗漏、旧产物、校验篡改、发行标签和拒绝覆盖等失败条件。

## 打包实现

用户入口为 `scripts/release.ps1`，默认生成本地包，只有 `-Publish` 执行 GitHub 发布。底层 `scripts/package.ps1` 解析 Cargo JSON compiler-artifact 消息，只收集与 workspace binary target 精确对应的 exe。每次使用唯一暂存目录，不从旧 target 或 dist 模糊匹配文件。

通过 MSVC dumpbin 检查 PE 导入依赖，拒绝非系统 DLL 或动态 VC runtime 依赖；每个 exe 的依赖记录写入 manifest。静态 CRT 配置保留在 `.cargo/config.toml`，但实际检查结果才是发行依据。

清单包含 suiteVersion、target、rustcVersion、sourceCommit、dirty、binaries 与 files。SHA256SUMS 覆盖 exe 和随包文档；清单和校验索引不循环校验自身，外部 `.sha256` 校验整个 ZIP。生成后解压到独立临时目录，核对完整文件集合和校验值，再运行全部冒烟及代表性行为检查。

相同版本的包已存在时不会覆盖。脚本默认仅清理本次创建的暂存目录和失败的本次归档，不删除历史发行文件。

## 当前已知问题

- 历史代码含 unused import/variable、dead code 和 Clippy 风格 warning，基线门槛是检查命令成功退出；后续逐步消除。
- 此次行为测试暴露并修复 cp/mv 的源参数必需性声明和 grep 未匹配退出码；csplit 的溢出检查改为 checked_add。
- `wc -l` 对 stdin 会带 `<stdin>` 标签，与常见 GNU 输出不同，目前测试保留该行为。
- `whereis` 仍含历史固定搜索路径；其业务兼容性需另行修正，构建和发行流程不依赖该路径。
- 迁移留下的旧各命令 target 缓存不属于发布输入，且被忽略；需要腾空间时可由维护者另行清理。
- Windows ARM64、x86、最低 Windows 版本和完整 GNU/POSIX 兼容性尚未验证。
- 远程 CI 的真实运行结果以 GitHub Actions 为准，本地命令通过不等于远程 workflow 已运行。
