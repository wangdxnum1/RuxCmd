# 开发与构建

[English](development.md) | **简体中文**

## 工程结构

根 virtual workspace 管理 `crates/*`。目前 139 个 package 产出 136 个命令；`ps-sys`、`ruxcmd-core` 是运行时库包，`ruxcmd-build` 是仅构建时使用的库包，`ps-bin` 产出 `ps.exe`，`ls` 同时提供供 `vdir` 复用的库。Cargo metadata 是构建及发布工具清单的唯一来源，冒烟策略必须与其二进制集合一致。

Edition 2021/2024 和各 crate 原版本保留。根 Cargo.lock 固定整个套件的依赖，resolver 为 3。ps 保留原来声明的 Rust 1.75 元数据，但整个仓库当前只用固定的 1.99.0 验证，不承诺整体 MSRV。

Cargo 命令可在根使用 `--workspace`，也可在单个 crate 目录运行；两者使用根锁文件和 target。默认 debug 产物路径可能受用户 Cargo 配置影响，发布命令显式使用 x86_64-pc-windows-msvc。

## 检查与测试

统一入口为 `scripts/build.ps1`；`-Task Verify` 执行与 CI 相同的完整流程，参数见 [脚本说明](../scripts/README-zh.md)。Rust 集成测试覆盖全套件的代表性行为，包括 Round 15 全部 14 个新命令。文件测试使用 tempfile；原生进程、硬链接和 ICMP 测试需要正常 Windows 环境，受限沙箱可能拒绝相关 API。traceroute 使用环回及有界 TEST-NET 探测，不断言多路由器网络行为。`scripts/difftest.ps1` 单独比较 GNU 输出和退出码；会修改文件的命令另比较独立文件系统快照。

发布冒烟只运行经过源码检查的帮助入口，以及 true/false 的退出码。每个进程有超时，异步读取输出和错误，超时终止本次进程树。帮助通过说明启动/参数入口能工作，不证明全部业务行为正确。

`open` 的五项桌面集成检查标记为 `#[ignore]`：它们会启动资源管理器、文件关联程序、命令窗口或浏览器，常规 `cargo test`、Verify 和发布验证均不执行。需要时在测试桌面手动运行 `cargo test -p open --test cli --target x86_64-pc-windows-msvc --locked -- --ignored`。资源管理器可能在临时测试路径删除后才完成导航并弹出“位置不可用”，因此不要将这些交互检查纳入无人值守验证。

路径按 PowerShell 当前目录解析，即使执行过 Set-Location 而 .NET 进程目录未同步，也可正确使用相对 BinDir/OutputDir。进程等待、stdin 写入和输出管道共用截止时间；Windows Job Object 负责本次子进程及其后代的生命周期，包括父进程先退出而后代仍持有输出管道的情况。

`scripts/tests/package-tests.ps1` 的 21 个测试检查含空格路径、Set-Location 后的相对路径、参数、超时、继承输出管道的后代、阻塞 stdin、大输出、ps-bin/ps 名称、清单遗漏、旧产物、校验篡改、发行标签和拒绝覆盖等失败条件。

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

## Windows 可执行文件属性

全部 136 个命令在普通 Cargo Debug/Release 编译时嵌入 VERSIONINFO，需要 Windows SDK 的 RC 编译器；资源编译失败会中止构建。共享入口为 `build/windows-version.rs`，由仅构建时使用的 `ruxcmd-build` 库实现。资源只链接到指定二进制目标，因此 `ps-bin` 正确生成 `ps.exe`，`vdir` 不会继承依赖库 `ls` 的文件身份。

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
