# RuxCmd

用 Rust 实现的 Linux 风格 Windows 命令集。当前包含 111 个独立 `.exe`，覆盖文件、文本、进程、网络和系统信息工具。

目标平台：Windows x64，MSVC。命令功能和 Windows 差异见 [兼容性说明](docs/compatibility.md)。

## 使用发行包

解压 `ruxcmd-<版本>-x86_64-pc-windows-msvc.zip`，将 `bin` 加入用户 PATH，重新打开终端。也可以直接指定路径：

```powershell
.\bin\cat.exe example.txt
.\bin\ls.exe --help
.\bin\ps.exe --list-columns
```

PowerShell 对 `cat`、`ls`、`cp`、`rm` 等名称有 alias。使用 `cat.exe` 或完整路径可明确调用 RuxCmd；不需要修改全局 alias。

发行包包含 `manifest.json`、`SHA256SUMS` 和使用说明。ZIP 旁的 `.sha256` 文件用于核对整个下载文件：

```powershell
Get-FileHash .\ruxcmd-0.1.0-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

套件版本在根 `Cargo.toml` 的 `workspace.metadata.ruxcmd.version` 中管理。各命令的 `--version` 保留自己的版本，可能与套件版本不同。

## 从源码构建

需要：

- Windows x64。
- Rustup；仓库的 `rust-toolchain.toml` 固定 Rust 1.99.0、rustfmt 和 Clippy。
- Visual Studio 或 Build Tools 的 **Desktop development with C++** 工作负载，含 MSVC x64 工具与 Windows SDK。
- PowerShell 7，用于测试和打包脚本。
- Git；首次构建需要下载 Cargo 依赖。

在仓库根执行：

```powershell
cargo build --workspace --bins --release --target x86_64-pc-windows-msvc --locked
.\target\x86_64-pc-windows-msvc\release\cat.exe --help
```

也可运行 `build-all.bat`。构建输出统一位于根 `target/`，脚本不依赖固定机器路径。

单个命令：

```powershell
cargo run -p cat -- example.txt
cargo test -p cat --locked
cargo run -p ps-bin -- --list-columns
```

## 验证与打包

```powershell
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
pwsh -File scripts/smoke-test.ps1 -BinDir target/x86_64-pc-windows-msvc/release
pwsh -File scripts/tests/package-tests.ps1
pwsh -File scripts/package.ps1
```

打包脚本执行 release 构建、PE DLL 依赖检查、111 个命令冒烟测试、校验和生成及解压后验证。输出在 `dist/`；相同发行文件已存在时拒绝覆盖。再次生成本地包可指定新目录：

```powershell
pwsh -File scripts/package.ps1 -OutputDir dist/local-check
```

本地包在清单记录未提交状态。正式发行从干净 tag 检出构建，使用 `-RequireClean -ExpectedTag v0.1.0`。GitHub Actions 的 Release workflow 手动接收已存在的 tag，生成 workflow artifact；当前不自动创建 GitHub Release。

## 开发

- [开发说明](docs/development.md)
- [贡献规范](CONTRIBUTING.md)
- [变更记录](CHANGELOG.md)
- [ps 文档](docs/ps/README-zh.md)

正式 crate 位于 `crates/`，`ps-sys`、`ps-core`、`ps-bin` 同属根 workspace。独立非命令项目不属于发行包。

## 许可证

整个仓库尚未统一指定许可证；`ps` 的三个 crate 保留已有 MIT 元数据声明。在维护者明确整体授权之前，不将本项目视为已采用统一开源许可证。
