# RuxCmd

[![CI](https://github.com/wangdxnum1/RuxCmd/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/wangdxnum1/RuxCmd/actions/workflows/ci.yml)

[English](README.md) | **简体中文**

用 Rust 实现的 Linux 风格 Windows 命令集。目前生成 **111 个独立可执行文件**，面向 Windows x64 / MSVC，覆盖文件、文本、进程、网络及系统信息工具。

项目仍在开发中。命令名称和参数沿用 Linux 风格，但不保证完整 GNU/POSIX 兼容性。支持的选项及已知差异见[命令兼容性表](docs/compatibility.md)。

## 使用命令

打开一次成功的 [CI 运行](https://github.com/wangdxnum1/RuxCmd/actions/workflows/ci.yml)，下载 `ruxcmd-windows-x64-ci` artifact。先解压 Actions 下载的文件，再解压其中的 `ruxcmd-<版本>-x86_64-pc-windows-msvc.zip`。

直接调用可执行文件，或将解压后的 `bin` 目录加入用户 `PATH`，再重新打开终端：

```powershell
.\bin\cat.exe example.txt
.\bin\ls.exe --help
.\bin\ps.exe --list-columns
```

PowerShell 为 `cat`、`ls`、`cp`、`rm` 等名称定义了别名。使用 `.exe` 后缀或完整路径，可明确调用 RuxCmd。

包内包含源码与构建清单、逐文件 SHA256 校验和及说明文档。将旁边 `.zip.sha256` 文件中的 ZIP 校验和与以下结果比较：

```powershell
Get-FileHash .\ruxcmd-0.1.0-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

CI artifact 保留 7 天，按 tag 构建的 Release workflow artifact 保留 30 天。目前 workflow 生成构建产物，不自动发布 GitHub Release。

## 本地编译

### 环境准备

- Windows x64。
- Git 和 [Rustup](https://rustup.rs/)。`rust-toolchain.toml` 固定 Rust **1.99.0**、rustfmt、Clippy 及 `x86_64-pc-windows-msvc` 目标。
- Visual Studio 或 Build Tools，安装 **Desktop development with C++（使用 C++ 的桌面开发）**，包含 MSVC x64 工具和 Windows SDK。
- PowerShell 7（`pwsh`），用于验证与打包脚本。
- 首次下载工具链及依赖时需要联网。

以下命令在 **PowerShell 中、仓库根目录下**执行。Rustup 会自动选择固定工具链。

### 克隆与编译

```powershell
git clone https://github.com/wangdxnum1/RuxCmd.git
cd RuxCmd
pwsh -File scripts/build.ps1
.\target\x86_64-pc-windows-msvc\release\cat.exe --help
```

所有可执行文件输出到 `target/x86_64-pc-windows-msvc/release/`。脚本始终使用已提交的依赖锁文件，支持任务、配置、命令/包、并行数及离线模式：

```powershell
pwsh -File scripts/build.ps1 -Package cat -Configuration Debug -Jobs 4
pwsh -File scripts/build.ps1 -Package ps -Offline
pwsh -File scripts/build.ps1 -Task Check
pwsh -File scripts/build.ps1 -Task Test -Package cat
pwsh -File scripts/build.ps1 -Task Clean -Configuration Debug -WhatIf
```

开发或测试单个命令：

```powershell
cargo run -p cat -- example.txt
cargo test -p cat --locked
cargo run -p ps-bin -- --list-columns
```

### 验证

执行与 CI 相同的完整验证入口：

```powershell
pwsh -File scripts/build.ps1 -Task Verify
```

冒烟检查以有超时限制的方式验证所有命令的启动或帮助入口。行为检查覆盖部分代表性命令，不代表每个命令都已实现完整兼容。历史 Clippy warning 会显示，当前只有 Clippy 返回错误时才使构建失败。

### 打包

```powershell
pwsh -File scripts/release.ps1
```

脚本编译 release 可执行文件，检查 DLL 依赖，执行冒烟检查，生成清单与校验和，再解压 ZIP 核对文件并重新运行冒烟和行为检查。验证完成的产物写入：

```text
dist/ruxcmd-0.1.0-x86_64-pc-windows-msvc.zip
dist/ruxcmd-0.1.0-x86_64-pc-windows-msvc.zip.sha256
```

套件版本取自 `Cargo.toml` 的 `workspace.metadata.ruxcmd.version`，各命令自身版本可以不同。本地包记录源码提交及是否存在未提交文件（`dirty`）。脚本不覆盖已有文件；再次生成相同版本时指定新目录：

```powershell
pwsh -File scripts/release.ps1 -OutputDir dist/local-check
```

## GitHub Actions 编译

### CI：push 和 Pull Request

[CI](.github/workflows/ci.yml) 在 push、Pull Request 及手动触发时运行，使用 `windows-2025` 和 PowerShell：

1. 检出指定提交，安装固定 Rust 工具链。
2. 恢复 Cargo 依赖及编译产物缓存。
3. 执行 `scripts/build.ps1 -Task Verify`：格式、check、Clippy、测试、release 编译、冒烟/行为检查及脚本失败路径测试。
4. 执行 `scripts/release.ps1`：生成并验证 ZIP、校验和、DLL 依赖及解压后的可执行文件。
5. 上传 ZIP 和校验文件，artifact 名为 `ruxcmd-windows-x64-ci`，保留 7 天。

手动运行：打开 **Actions → CI → Run workflow**，选择分支后启动。进入运行详情查看各步骤日志；成功后下载 artifact。

### Release：已存在的版本 tag

[Release](.github/workflows/release.yml) 需要手动启动，构建一个**已存在的 tag**，例如 `v0.1.0`：

1. 更新 `Cargo.toml` 中的套件版本和 [CHANGELOG](CHANGELOG.md)，提交并推送修改。
2. CI 成功后，在该提交上创建并推送与版本一致的 tag。以 `0.1.0` 为例：

   ```powershell
   git tag -a v0.1.0 -m "RuxCmd 0.1.0"
   git push origin v0.1.0
   ```

3. 打开 **Actions → Release → Run workflow**，选择 `main`，在 `tag` 输入框填入 `v0.1.0` 并启动。仅推送 tag 不会触发 Release。
4. workflow 检出 tag，安装 Rust，执行 `scripts/build.ps1 -Task Verify`，再用 `scripts/release.ps1 -RequireClean -Tag <tag>` 打包。
5. 成功后下载 `ruxcmd-v0.1.0-windows-x64` artifact，保留 30 天。将文件发布到 GitHub Release，需要维护者另行操作。

tag 必须与套件版本一致，并指向当前检出的提交。正式打包拒绝含未提交文件的工作目录。若需本地执行相同检查，在干净克隆中检出 tag 后运行：

```powershell
pwsh -File scripts/release.ps1 -RequireClean -Tag v0.1.0
```

### 从本机发布 GitHub Release

安装 GitHub CLI 并执行 `gh auth login`。发布前提交或移开所有本地修改（包括未跟踪文件），并更新套件版本和变更记录：

```powershell
pwsh -File scripts/release.ps1 -Publish -WhatIf
pwsh -File scripts/release.ps1 -Publish
```

只有 `-Publish` 才会联系 GitHub 执行发布。它从 `origin` 确定仓库，执行完整验证，在 HEAD 创建缺失的对应版本 tag（或使用指向该提交的已有 tag），生成干净的发布包，推送 tag 并上传 ZIP 和校验文件。已存在的 GitHub Release 和冲突 tag 都会被拒绝。使用 `-Draft` 创建草稿，或通过 `-NotesFile` 指定发布说明；默认使用已提交的 CHANGELOG。上传失败时保留本地文件和已推送的 tag，供排查及手动恢复。

完整参数、示例及输出路径见[脚本使用说明](scripts/README.md)。可以从其他目录通过完整路径运行脚本；相对 `-OutputDir` 和 `-NotesFile` 按当前 PowerShell 目录解析。

## 项目结构

```text
crates/                  命令及库包
scripts/                 冒烟、行为和打包检查
.github/workflows/       CI 及按 tag 构建的发布流程
docs/                    开发与兼容性文档
Cargo.toml               根 workspace 及套件版本
Cargo.lock               统一提交的依赖锁文件
rust-toolchain.toml      固定 Rust 工具链
```

**113 个包生成 111 个命令**：110 个命令各占一个包；`ps` 由两个库包（`ps-sys`、`ps-core`）和一个可执行包（`ps-bin`，生成 `ps.exe`）组成。独立非命令项目不纳入套件。

## 贡献与文档

- [贡献规范](CONTRIBUTING.md)
- [开发说明](docs/development.md)
- [命令兼容性](docs/compatibility.md)
- [变更记录](CHANGELOG.md)
- [ps 文档](docs/ps/README-zh.md)

## 许可证

整个仓库尚未统一指定许可证。`ps` 的三个包保留已有 MIT 元数据声明。仓库公开不代表整个项目已采用统一的开源许可证。
