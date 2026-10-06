# 构建与发布脚本

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

`Verify` 必须覆盖整个 workspace，不接受 `-Package`。依次执行格式/check/Clippy、Rust 测试、编译、全部命令冒烟、代表性行为及两组脚本测试。任何一步失败均立即停止。实际产物目录由 Cargo metadata 确定，也支持 `CARGO_TARGET_DIR`；普通配置为 `target/x86_64-pc-windows-msvc/{release|debug}/`。

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

CI workflow 调用 `build.ps1 -Task Verify` 和 `release.ps1`。Release workflow 检出已存在的版本 tag，调用相同 Verify 入口，再运行 `release.ps1 -Tag <tag> -RequireClean`；它仅上传 Actions artifact，不执行 `-Publish`。远程发布需要维护者明确运行本地发布命令。

脚本测试：

```powershell
pwsh -File scripts/tests/entrypoint-tests.ps1
pwsh -File scripts/tests/package-tests.ps1
```

入口测试使用隔离的临时 Git/Cargo 工程，覆盖参数、编译失败、实际 Debug 编译、只读预览、tag/仓库/清单校验等，不联系 GitHub 也不实际发布。
