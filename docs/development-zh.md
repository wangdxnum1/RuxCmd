# 开发与构建

[English](development.md) | **简体中文**

## 工程结构

根 virtual workspace 管理 `crates/*`。目前 145 个 package 产出 142 个命令；`ps-sys`、`ruxcmd-core` 是运行时库包，`ruxcmd-build` 是仅构建时使用的库包，`ps-bin` 产出 `ps.exe`，`ls` 同时提供供 `vdir` 复用的库。Cargo metadata 是构建及发布工具清单的唯一来源，冒烟策略必须与其二进制集合一致。

Edition 2021/2024 和各 crate 原版本保留。根 Cargo.lock 固定整个套件的依赖，resolver 为 3。ps 保留原来声明的 Rust 1.75 元数据，但整个仓库当前只用固定的 1.99.0 验证，不承诺整体 MSRV。

Cargo 命令可在根使用 `--workspace`，也可在单个 crate 目录运行；两者使用根锁文件和 target。默认 debug 产物路径可能受用户 Cargo 配置影响，发布命令显式使用 x86_64-pc-windows-msvc。

## 检查与测试

统一入口为 `scripts/build.ps1`；`-Task Verify` 执行与 CI 相同的完整流程，参数见 [脚本说明](../scripts/README-zh.md)。Rust 集成测试覆盖全套件的代表性行为，包括 Round 15 全部 14 个新命令。文件测试使用 tempfile；原生进程、硬链接和 ICMP 测试需要正常 Windows 环境，受限沙箱可能拒绝相关 API。traceroute 使用环回及有界 TEST-NET 探测，不断言多路由器网络行为。`scripts/difftest.ps1` 单独比较 GNU 输出和退出码；会修改文件的命令另比较独立文件系统快照。

发布冒烟只运行经过源码检查的帮助入口，以及 true/false 的退出码。每个进程有超时，异步读取输出和错误，超时终止本次进程树。帮助通过说明启动/参数入口能工作，不证明全部业务行为正确。

`open` 的五项桌面集成检查标记为 `#[ignore]`：它们会启动资源管理器、文件关联程序、命令窗口或浏览器，常规 `cargo test`、Verify 和发布验证均不执行。需要时在测试桌面手动运行 `cargo test -p open --test cli --target x86_64-pc-windows-msvc --locked -- --ignored`。资源管理器可能在临时测试路径删除后才完成导航并弹出“位置不可用”，因此不要将这些交互检查纳入无人值守验证。

路径按 PowerShell 当前目录解析，即使执行过 Set-Location 而 .NET 进程目录未同步，也可正确使用相对 BinDir/OutputDir。进程等待、stdin 写入和输出管道共用截止时间；Windows Job Object 负责本次子进程及其后代的生命周期，包括父进程先退出而后代仍持有输出管道的情况。

CI 还会运行 `scripts/compat-check.ps1`：把兼容表与 `cargo metadata`、各命令 `--help` 对照，并校验 README 与 development 页面引用的命令数/包数。表格结构损坏、缺行或重复行、选项只在一侧存在、计数过期都会让 job 失败。help 不能可靠反映选项的命令会显式跳过并打印原因。

`scripts/tests/package-tests.ps1` 的 21 个测试检查含空格路径、Set-Location 后的相对路径、参数、超时、继承输出管道的后代、阻塞 stdin、大输出、ps-bin/ps 名称、清单遗漏、旧产物、校验篡改、发行标签和拒绝覆盖等失败条件。

## 打包实现

用户入口为 `scripts/release.ps1`，默认生成本地包，只有 `-Publish` 执行 GitHub 发布。底层 `scripts/package.ps1` 解析 Cargo JSON compiler-artifact 消息，只收集与 workspace binary target 精确对应的 exe。每次使用唯一暂存目录，不从旧 target 或 dist 模糊匹配文件。

通过 MSVC dumpbin 检查 PE 导入依赖，拒绝非系统 DLL 或动态 VC runtime 依赖；每个 exe 的依赖记录写入 manifest。静态 CRT 配置保留在 `.cargo/config.toml`，但实际检查结果才是发行依据。

清单包含 suiteVersion、target、rustcVersion、sourceCommit、dirty、binaries 与 files。SHA256SUMS 覆盖 exe 和随包文档；清单和校验索引不循环校验自身，外部 `.sha256` 校验整个 ZIP。生成后解压到独立临时目录，核对完整文件集合和校验值，再运行全部冒烟及代表性行为检查。

相同版本的包已存在时不会覆盖。脚本默认仅清理本次创建的暂存目录和失败的本次归档，不删除历史发行文件。

## 发布与安装

`scripts/release.ps1` 产出 `dist/ruxcmd-<版本>-x86_64-pc-windows-msvc.zip` 及内容为 `<sha256>  <文件名>` 的 `.zip.sha256`。安装侧不改变发布纪律：显式 `-Tag` 必须等于套件版本并指向当前 checkout，正式打包要求工作区干净。

`scripts/install.ps1` 是安装经过校验的发布包的用户入口；它加载 `scripts/common.ps1` 中的共享助手，使 shim 策略与安全解压和 EXE 安装包保持一致。输入可以是 ZIP 路径（读取旁边的 `.sha256`，或用 `-Sha256` 显式指定），也可以是 `0.4.3` 这样的版本号（从 GitHub Release 下载）。SHA256 在创建任何目录之前校验；解压到同级的暂存目录，检查路径穿越与重复条目，并验证包内 `SHA256SUMS`，之后才写入目标位置。若检测到独立 EXE 安装包的卸载注册表项，脚本会拒绝执行。

布局：可执行文件直接进入 `-InstallDir`（默认 `%LOCALAPPDATA%\RuxCmd\bin`，相当于归档中的 `bin/`），其余文件进入 `<InstallDir>\share`。`.ruxcmd-install.json` 记录版本、校验和、可执行文件名、shim 名称及旧版本备份，供重复执行与卸载使用。

Windows 先搜索 `C:\Windows\System32`，再搜索用户 `PATH`；PowerShell 别名又会遮蔽许多命令名。因此裸名 `find`、`sort`、`where`、`cat`、`ls`、`cp`、`mv`、`rm`、`echo`、`pwd`、`ps`、`kill`、`sleep`、`tee`、`clear`、`curl`、`wget` 不能可靠地命中 RuxCmd。安装器只为包内实际存在的命令生成 `rux-<命令>.cmd` shim（`@echo off` 与 `"%~dp0<命令>.exe" %*`）；套件不含 `where.exe`，因此不会生成 `rux-where`。`%*` 在常规用法下保留调用方引号；需要让字面引号穿过多层 cmd.exe 的情形仍受 cmd.exe 限制。`rux` 预留给未来的多路复用入口。

升级会先把现有可执行文件、`share` 目录和状态文件复制到 `<InstallDir>.old-<版本>`（无状态文件时使用带时间戳的名称），再替换文件；替换中途失败会从该副本恢复。相同版本与校验和重复执行不会改动任何文件，shim 也只在缺失或内容不同时重写。脚本只通过 `[Environment]::SetEnvironmentVariable(..., 'User')` 修改用户 PATH，并在修改前打印变更前后的完整值；`-Force` 跳过确认。绝不写机器级 PATH 或其他注册表值。测试通过 `RUXCMD_INSTALL_USER_PATH_FILE` 重定向 PATH，不接触真实用户配置。

Scoop 与 winget 源文件位于 `scripts/packaging/`。`scoop/ruxcmd.json` 使用 `extract_dir: "bin"`，并把冲突命令别名到 `rux-*`；只有把它复制到某个 bucket 仓库后，`scoop install ruxcmd` 才能解析。`packaging/winget/` 下的三个文件是 `microsoft/winget-pkgs` 的生成模板；本地 `winget validate --manifest scripts/packaging/winget` 已通过，但正式发布仍需 fork、按版本目录放置并提交 PR。winget 接受未签名的 ZIP 配合 `NestedInstallerType: portable`，首次运行仍可能触发 SmartScreen 警告。由于 winget 对每个可执行文件只能创建一个命令别名，冲突命令只暴露 `rux-*`。

`scripts/tests/install-tests.ps1` 覆盖：校验失败且不写任何文件、首次安装生成 shim 并追加 PATH、幂等（文件时间戳与 PATH 不变）、升级保留旧版本目录，以及带/不带 `-RemoveInstallDir` 的卸载；全部在临时目录中执行。

## 当前已知问题

- 历史代码含 unused import/variable、dead code 和 Clippy 风格 warning，基线门槛是检查命令成功退出；后续逐步消除。
- 此次行为测试暴露并修复 cp/mv 的源参数必需性声明和 grep 未匹配退出码；csplit 的溢出检查改为 checked_add。
- `wc` 对 stdin 输出裸计数、不带名称后缀，与 GNU 一致；`<stdin>` 标签只出现在错误信息中。
- `whereis` 仍含历史固定搜索路径；其业务兼容性需另行修正，构建和发行流程不依赖该路径。
- 迁移留下的旧各命令 target 缓存不属于发布输入，且被忽略；需要腾空间时可由维护者另行清理。
- Windows ARM64、x86、最低 Windows 版本和完整 GNU/POSIX 兼容性尚未验证。
- 远程 CI 的真实运行结果以 GitHub Actions 为准，本地命令通过不等于远程 workflow 已运行。

## Windows 可执行文件属性

全部 142 个命令在普通 Cargo Debug/Release 编译时嵌入 VERSIONINFO，需要 Windows SDK 的 RC 编译器；资源编译失败会中止构建。共享入口为 `build/windows-version.rs`，由仅构建时使用的 `ruxcmd-build` 库实现。资源只链接到指定二进制目标，因此 `ps-bin` 正确生成 `ps.exe`，`vdir` 不会继承依赖库 `ls` 的文件身份。

根 `workspace.metadata.ruxcmd.version` 提供产品版本；文件版本及两组固定数字版本使用 major.minor.patch.0。产品名称、公司名称及版权来自 `workspace.metadata.ruxcmd.windows`；文件说明来自各包 description，内部名称与原始文件名使用可执行目标名称。Debug 和预发布标记随构建设置；根或包清单变化会重新生成资源，不嵌入编译时间或本机源码路径。

Verify 通过 `scripts/verify-version-info.ps1` 读取所有文件属性，不启动程序或资源管理器。打包在计算哈希之前和解压后各校验一次，并将字段记录到 manifest.json 的 `binaries[].versionInfo`。缺失/过期字段、错误命令身份、固定数字版本或构建标记均会被拒绝。四项资源用例使打包脚本测试增至 21 项；另有三项独立工作区的真实 RC 编译测试，覆盖中文/引号/反斜杠、空格路径、二进制别名、仅改根版本号的缓存刷新、带连字符的构建元数据及 Debug/Release 标记。

这些说明字段与 Authenticode 数字签名相互独立。本次不创建证书或修改信任存储，可执行文件仍未签名。无需打开资源管理器即可查看：

```powershell
(Get-Item ./target/x86_64-pc-windows-msvc/release/ls.exe).VersionInfo |
    Format-List FileDescription,ProductName,CompanyName,FileVersion,ProductVersion,OriginalFilename,LegalCopyright
pwsh -File scripts/verify-version-info.ps1
```

## ps 结构与重构验收

ps 专属模型、过滤、排序、进程树及列格式化均位于 `ps-bin` 内部；`ps-sys` 继续由
`ps`、`pgrep`、`pkill` 共享。42 名称注册表改用静态描述和四组格式化函数，不再
逐列创建 trait 对象；c/cp 占位等现有兼容限制保持。单元测试包含旧实现的全列
元数据/输出快照；标准 Verify 覆盖这些测试、全部命令、原生版本资源和打包行为。

需要额外对比新旧程序时，将旧版保存在单独目录并保留 `ps.exe` 文件名，再运行：

```powershell
pwsh -File scripts/tests/ps-refactor-compare.ps1 -BaselineBinary target/ps-before-simplification/ps.exe
```

该对比查询自己的进程，检查 stdout/stderr/退出码与文件输出，不发送进程信号或
打开桌面窗口。`docs/ps/docs/superpowers` 内的原始计划保留为历史资料，不代表
当前包布局。
