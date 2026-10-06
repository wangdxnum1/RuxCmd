# RuxCmd

[![CI](https://github.com/wangdxnum1/RuxCmd/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/wangdxnum1/RuxCmd/actions/workflows/ci.yml)

**English** | [简体中文](README-zh.md)

Linux-style command-line tools for Windows, written in Rust. RuxCmd currently builds **111 standalone executables** for Windows x64 with MSVC, covering files, text, processes, networking, and system information.

The project is under active development. Familiar command names and options do not imply full GNU/POSIX compatibility. See the [command compatibility table](docs/compatibility.md) for supported options and known differences.

## Use the tools

Open a successful [CI run](https://github.com/wangdxnum1/RuxCmd/actions/workflows/ci.yml) and download the `ruxcmd-windows-x64-ci` artifact. Extract the Actions download, then extract the `ruxcmd-<version>-x86_64-pc-windows-msvc.zip` inside it.

Run the executables directly, or add the extracted `bin` directory to your user `PATH` and reopen your terminal:

```powershell
.\bin\cat.exe example.txt
.\bin\ls.exe --help
.\bin\ps.exe --list-columns
```

PowerShell defines aliases such as `cat`, `ls`, `cp`, and `rm`. Use the `.exe` suffix or a full path to call RuxCmd explicitly.

Packages contain a source/build manifest, per-file SHA256 checksums, and documentation. Compare the ZIP checksum in the companion `.zip.sha256` file with:

```powershell
Get-FileHash .\ruxcmd-0.1.0-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

CI artifacts are retained for 7 days; tagged Release workflow artifacts for 30 days. The workflows currently generate artifacts without publishing a GitHub Release.

## Build locally

### Prerequisites

- Windows x64.
- Git and [Rustup](https://rustup.rs/). `rust-toolchain.toml` pins Rust **1.99.0**, rustfmt, Clippy, and the `x86_64-pc-windows-msvc` target.
- Visual Studio or Build Tools with **Desktop development with C++**, including MSVC x64 tools and the Windows SDK.
- PowerShell 7 (`pwsh`) for verification and packaging scripts.
- Network access for the initial toolchain and dependency downloads.

Run the commands below in **PowerShell from the repository root**. Rustup automatically selects the pinned toolchain.

### Clone and compile

```powershell
git clone https://github.com/wangdxnum1/RuxCmd.git
cd RuxCmd
pwsh -File scripts/build.ps1
.\target\x86_64-pc-windows-msvc\release\cat.exe --help
```

Executables are written to `target/x86_64-pc-windows-msvc/release/`. The script always uses the committed lockfile. Select a task, configuration, command/package, concurrency or offline mode:

```powershell
pwsh -File scripts/build.ps1 -Package cat -Configuration Debug -Jobs 4
pwsh -File scripts/build.ps1 -Package ps -Offline
pwsh -File scripts/build.ps1 -Task Check
pwsh -File scripts/build.ps1 -Task Test -Package cat
pwsh -File scripts/build.ps1 -Task Clean -Configuration Debug -WhatIf
```

Develop or test one command:

```powershell
cargo run -p cat -- example.txt
cargo test -p cat --locked
cargo run -p ps-bin -- --list-columns
```

### Verify

Run the same complete verification entry point used by CI:

```powershell
pwsh -File scripts/build.ps1 -Task Verify
```

Smoke checks verify every command's startup/help entry point with bounded execution. Behavior checks cover selected commands, not complete compatibility. Historical Clippy warnings are reported; the current gate fails when Clippy returns an error.

### Package

```powershell
pwsh -File scripts/release.ps1
```

The script builds release binaries, checks DLL dependencies, runs smoke checks, generates the manifest and checksums, then extracts and verifies the ZIP and reruns smoke/behavior checks. Verified output is written to:

```text
dist/ruxcmd-0.1.0-x86_64-pc-windows-msvc.zip
dist/ruxcmd-0.1.0-x86_64-pc-windows-msvc.zip.sha256
```

The suite version comes from `workspace.metadata.ruxcmd.version` in `Cargo.toml`; individual command versions may differ. Local packages record the source commit and whether the checkout has uncommitted files (`dirty`). Existing files are never overwritten; choose a new directory for another build of the same version:

```powershell
pwsh -File scripts/release.ps1 -OutputDir dist/local-check
```

## Build in GitHub Actions

### CI: pushes and pull requests

[CI](.github/workflows/ci.yml) runs on pushes, pull requests, and manual dispatch, using `windows-2025` and PowerShell:

1. Check out the requested commit and install the pinned Rust toolchain.
2. Restore cached Cargo dependencies and build outputs.
3. Run `scripts/build.ps1 -Task Verify`: formatting, check, Clippy, tests, release compilation, smoke/behavior checks and script failure-path tests.
4. Run `scripts/release.ps1`: build and verify the ZIP, checksums, DLL dependencies, and extracted executables.
5. Upload ZIP and checksum files as `ruxcmd-windows-x64-ci` (7-day retention).

To start CI manually, open **Actions → CI → Run workflow**, select a branch, and run it. Inspect individual step logs in the run and download its artifact after success.

### Release: an existing version tag

[Release](.github/workflows/release.yml) is started manually and builds an **existing tag**, such as `v0.1.0`:

1. Update the suite version in `Cargo.toml` and [CHANGELOG](CHANGELOG.md), then commit and push.
2. After CI succeeds, create and push a matching tag at that commit. For version `0.1.0`:

   ```powershell
   git tag -a v0.1.0 -m "RuxCmd 0.1.0"
   git push origin v0.1.0
   ```

3. Open **Actions → Release → Run workflow** on `main`, enter `v0.1.0` in the `tag` input, and start it. Pushing a tag alone does not trigger Release.
4. The workflow checks out the tag, installs Rust, runs `scripts/build.ps1 -Task Verify`, and packages with `scripts/release.ps1 -RequireClean -Tag <tag>`.
5. Download `ruxcmd-v0.1.0-windows-x64` after success (30-day retention). Publishing these files as a GitHub Release is a separate maintainer action.

The tag must match the suite version and point to the checked-out commit. Formal packaging rejects a dirty checkout. To package locally with the same checks, check out the tag in a clean clone and run:

```powershell
pwsh -File scripts/release.ps1 -RequireClean -Tag v0.1.0
```

### Publish a GitHub Release from your computer

Install GitHub CLI and run `gh auth login`. Commit or set aside all local changes, including untracked files, and update the suite version and changelog before publishing:

```powershell
pwsh -File scripts/release.ps1 -Publish -WhatIf
pwsh -File scripts/release.ps1 -Publish
```

Only `-Publish` contacts GitHub to publish. It derives the repository from `origin`, runs full verification, creates a missing matching tag at HEAD (or uses an existing tag at that commit), packages a clean build, pushes the tag and uploads the ZIP and checksum. Existing GitHub releases and conflicting tags are refused. Use `-Draft` to create a draft or `-NotesFile` for custom release notes; by default the committed changelog supplies the notes. If upload fails, the local files and any pushed tag are retained for diagnosis and manual recovery.

See [script usage](scripts/README.md) for all parameters, examples, and output paths. Commands can be invoked from another directory using the script's full path; relative `-OutputDir` and `-NotesFile` paths follow your current PowerShell directory.

## Project layout

```text
crates/                  Command and library packages
scripts/                 Smoke, behavior, and packaging checks
.github/workflows/       CI and tagged release builds
docs/                    Development and compatibility documentation
Cargo.toml               Root workspace and suite version
Cargo.lock               Shared, committed dependency lockfile
rust-toolchain.toml      Pinned Rust toolchain
```

**113 packages produce 111 commands**: 110 commands each have one package; `ps` uses two libraries (`ps-sys`, `ps-core`) and one executable package (`ps-bin`, producing `ps.exe`). Independent non-command projects are outside the suite.

## Contributing and documentation

- [Contribution guidelines](CONTRIBUTING.md) (Chinese)
- [Development notes](docs/development.md) (Chinese)
- [Command compatibility](docs/compatibility.md) (Chinese)
- [Changelog](CHANGELOG.md)
- [ps documentation](docs/ps/README.md)

## License

A repository-wide license has not yet been selected. The three `ps` packages retain their existing MIT metadata declarations. Public repository visibility does not establish a uniform open-source license for the whole project.
