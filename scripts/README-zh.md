# 构建与发布脚本

[English](README.md) | **简体中文**

所有入口需要 Windows x64、MSVC 和 PowerShell 7。脚本自动定位仓库根目录，Cargo 使用 `rust-toolchain.toml` 和根锁文件。使用 `Get-Help ./scripts/build.ps1 -Full` 或 `Get-Help ./scripts/release.ps1 -Full` 查看入口帮助。

## build.ps1

| 参数 | 默认值 | 功能 |
| --- | --- | --- |
| `-Task` | `Build` | `Build` 编译；`Check` 格式/check/Clippy；`Test` 测试；`Verify` 完整验证；`Clean` 清理指定配置的 Cargo 产物 |
| `-Configuration` | `Release` | 编译及清理选择 `Release` 或 `Debug`；check 和测试使用 Cargo 默认测试配置 |
| `-Package` | 全 workspace | Cargo 包名或命令名；例如 `ps` 自动映射 `ps-bin`。PowerShell 调用可传多个名称，如 `-Package cat,echo` |
| `-Jobs` | Cargo 默认值 | 编译并行数，1–256 |
| `-Offline` | 关闭 | Cargo 元数据、编译/check/Clippy/测试及验证子脚本的 Cargo 操作禁止联网；依赖须已缓存 |
| `-Verbose` | 关闭 | 输出 Cargo 详细信息 |
| `-WhatIf` | 关闭 | 预览操作；不编译、测试或清理。仍会读取 Cargo 元数据 |

```powershell
pwsh -File scripts/build.ps1
pwsh -File scripts/build.ps1 -Package ps -Configuration Debug
pwsh -File scripts/build.ps1 -Task Check -Offline
pwsh -File scripts/build.ps1 -Task Test -Package cat
pwsh -File scripts/build.ps1 -Task Verify -Jobs 4
pwsh -File scripts/build.ps1 -Task Clean -Configuration Debug -WhatIf
```

`Verify` 必须覆盖整个 workspace，不接受 `-Package`。依次执行格式/check/Clippy、Rust 测试、编译、原生版本资源检查、全部命令冒烟、代表性行为及全部脚本测试。任何一步失败均立即停止。实际产物目录由 Cargo metadata 确定，也支持 `CARGO_TARGET_DIR`；普通配置为 `target/x86_64-pc-windows-msvc/{release|debug}/`。

`Clean` 使用 `cargo clean`，显式选择 workspace 或指定包，只处理选定目标和配置的项目编译产物，保留第三方依赖缓存；不清理 `dist`、源码或用户文件。`Check` 只检查格式，不修改源码。`-WhatIf` 不能替代实际验证。

## release.ps1

| 参数 | 功能 |
| --- | --- |
| `-OutputDir` | 输出目录，默认仓库 `dist/`；相对路径以调用者的 PowerShell 目录为基准 |
| `-Tag` | 必须为 `v<套件版本>`。本地构建需已有 tag 且指向 HEAD；发布模式默认按套件版本推导 tag，可创建缺失的 tag |
| `-RequireClean` | 本地打包也要求工作目录干净。`-Publish` 始终要求干净 |
| `-Publish` | 完整验证后推送版本 tag，创建 GitHub Release 并上传 ZIP 和校验文件 |
| `-Draft` | 与 `-Publish` 一起使用，创建 GitHub Release 草稿 |
| `-NotesFile` | 与 `-Publish` 一起使用，指定非空发布说明文件；默认根 CHANGELOG.md |
| `-WhatIf` | 预览，不构建、创建 tag 或联系 GitHub发布；仍验证本地参数及源码状态 |

```powershell
# 生成本地包；允许未提交状态，但 manifest 会记录 dirty=true。
pwsh -File scripts/release.ps1 -OutputDir dist/local-check

# 正式 tag 构建，与 Release workflow 一致。
pwsh -File scripts/release.ps1 -Tag v0.1.0 -RequireClean

# 以下发布操作需要干净的工作目录、gh 登录及仓库写权限。
pwsh -File scripts/release.ps1 -Publish -WhatIf
pwsh -File scripts/release.ps1 -Publish -Draft
```

本地模式复用 `package.ps1`，不推送 tag、不创建 GitHub Release。已有输出文件拒绝覆盖，同版本重复打包需换输出目录。包名为 `ruxcmd-<版本>-x86_64-pc-windows-msvc.zip`，并生成 `.zip.sha256`。

发布模式仅支持 github.com 的 `origin`，支持 HTTPS/SSH 地址；显式向该仓库操作，不使用其他仓库环境变量。已有 tag 必须指向 HEAD，远程 annotated tag 使用其指向的提交校验；不移动 tag、不覆盖已有 Release。带预发布后缀的套件版本自动使用 GitHub prerelease。没有指定 `-Draft` 时直接发布。

脚本先完成本地验证和打包，再推送 tag 及上传。上传失败保留包和 tag，不自动删除远程记录；先查看 GitHub 的实际状态，再手动恢复。工作目录检查包含未跟踪文件；个人日志或临时说明可移到仓库外或已忽略目录，不需要为了发布提交无关文件。

## CI

GNU 差分检查独立于 Verify 执行：

Round 16 为专属用例增加 NUL、未终止 NUL、二进制 NUL 和连续空记录形态，不改变旧 LF 矩阵。摘要 `--tag -z` 和四种普通摘要校验均实际比较；仅在参照 `--help` 成功且确实不含选项时跳过该用例。例如 Git 的 GNU cksum 8.32 不支持 `--zero`，RuxCmd 仍有已知 CRC 向量测试；启动失败不会被记为“不支持”。

```powershell
./scripts/difftest.ps1
./scripts/difftest.ps1 -Cases rmdir,link,unlink,mktemp,install,cksum,sha1sum,sha512sum
./scripts/difftest.ps1 -Seed 42 -Variants 3
```

默认使用当前 release 二进制（缺失时回退 debug）和 Git for Windows 自带 GNU 工具；请先构建或显式指定 `-BinDir`。所选 RuxCmd 二进制必须存在；缺失 GNU 参照及声明的平台差异计为跳过。比较原始 stdout 和退出码；修改文件的命令使用两份独立 fixture，另比较文件内容与目录状态。仅 mktemp 随机名作规范化，需要正常 Windows 硬链接支持。SHA 校验模式会实际验证清单，不受 MSYS 二进制生成格式豁免影响。不声明文件系统 stderr 文案、ACL、WTS/utmp 身份及长列表格式逐字节等价。

CI workflow 调用 `build.ps1 -Task Verify` 和 `release.ps1`。Release workflow 检出已存在的版本 tag，调用相同 Verify 入口，再运行 `release.ps1 -Tag <tag> -RequireClean`；它仅上传 Actions artifact，不执行 `-Publish`。远程发布需要维护者明确运行本地发布命令。

脚本测试：

```powershell
pwsh -File scripts/tests/entrypoint-tests.ps1
pwsh -File scripts/tests/package-tests.ps1
```

入口测试使用隔离的临时 Git/Cargo 工程，覆盖参数、编译失败、实际 Debug 编译、只读预览、tag/仓库/清单校验等，不联系 GitHub 也不实际发布。

## verify-version-info.ps1

只读检查全部 125 个 EXE，不启动程序；要求套件版本、命令身份、发布者、功能说明、版权、数字版本和构建标记均正确。-BinDir 指定目录，Debug 产物另加 -DebugBuild。Verify 和 ZIP 打包会自动检查；scripts/tests/version-resource-tests.ps1 提供三项真实 RC 编译回归。
