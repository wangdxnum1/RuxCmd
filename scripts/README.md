# Build and release scripts

**English** | [简体中文](README-zh.md)

All entry points require Windows x64, MSVC, and PowerShell 7. The scripts locate the repository root automatically, and Cargo uses `rust-toolchain.toml` and the root lockfile. Use `Get-Help ./scripts/build.ps1 -Full` or `Get-Help ./scripts/release.ps1 -Full` for built-in help.

## build.ps1

| Parameter | Default | Purpose |
| --- | --- | --- |
| `-Task` | `Build` | `Build` compiles; `Check` runs fmt/check/Clippy; `Test` runs tests; `Verify` runs full verification; `Clean` removes Cargo artifacts for the selected configuration |
| `-Configuration` | `Release` | Build and clean select `Release` or `Debug`; check and tests use Cargo's default test profile |
| `-Package` | whole workspace | A Cargo package name or command name; for example `ps` maps automatically to `ps-bin`. PowerShell invocations accept several names, e.g. `-Package cat,echo` |
| `-Jobs` | Cargo default | Compilation parallelism, 1–256 |
| `-Offline` | off | Cargo metadata, build/check/Clippy/test, and the Cargo operations of verification sub-scripts must not touch the network; dependencies must already be cached |
| `-Verbose` | off | Show verbose Cargo output |
| `-WhatIf` | off | Preview; does not build, test, or clean. Still reads Cargo metadata |

```powershell
pwsh -File scripts/build.ps1
pwsh -File scripts/build.ps1 -Package ps -Configuration Debug
pwsh -File scripts/build.ps1 -Task Check -Offline
pwsh -File scripts/build.ps1 -Task Test -Package cat
pwsh -File scripts/build.ps1 -Task Verify -Jobs 4
pwsh -File scripts/build.ps1 -Task Clean -Configuration Debug -WhatIf
```

`Verify` must cover the whole workspace and does not accept `-Package`. It runs formatting/check/Clippy, the Rust tests, compilation, native version-resource checks, all command smoke checks, representative behavior checks, and all script test suites in order, stopping at the first failure. The actual artifact directory is determined by Cargo metadata, and `CARGO_TARGET_DIR` is supported; the normal configuration is `target/x86_64-pc-windows-msvc/{release|debug}/`.

`Clean` uses `cargo clean`, explicitly selecting the workspace or given packages, and only removes this repository's build artifacts for the selected targets and configuration, keeping third-party dependency caches; it does not clean `dist`, sources, or user files. `Check` inspects formatting only and never modifies sources. `-WhatIf` is not a substitute for real verification.

## release.ps1

| Parameter | Purpose |
| --- | --- |
| `-OutputDir` | Output directory, default repository `dist/`; relative paths resolve against the caller's PowerShell directory |
| `-Tag` | Must be `v<suite version>`. Local builds require an existing tag pointing at HEAD; publish mode derives the tag from the suite version and can create a missing one |
| `-RequireClean` | Requires a clean working directory even for local packaging. `-Publish` always requires clean |
| `-Publish` | After full verification, pushes the version tag, creates the GitHub Release, and uploads the ZIP and checksum files |
| `-Draft` | With `-Publish`, creates a GitHub Release draft |
| `-NotesFile` | With `-Publish`, a non-empty release notes file; defaults to the root CHANGELOG.md |
| `-WhatIf` | Preview; does not build, create tags, or contact GitHub, but still validates local parameters and the source state |

```powershell
# Produce a local package; an uncommitted checkout is allowed, but the manifest records dirty=true.
pwsh -File scripts/release.ps1 -OutputDir dist/local-check

# Formal tag build, identical to the Release workflow.
pwsh -File scripts/release.ps1 -Tag v0.1.0 -RequireClean

# The publish operations below need a clean working directory, gh login, and repository write access.
pwsh -File scripts/release.ps1 -Publish -WhatIf
pwsh -File scripts/release.ps1 -Publish -Draft
```

Local mode reuses `package.ps1`, pushes no tags, and creates no GitHub Release. Existing output files are never overwritten; repackaging the same version requires a different output directory. The package is named `ruxcmd-<version>-x86_64-pc-windows-msvc.zip` and comes with a `.zip.sha256`.

Publish mode supports only a github.com `origin`, over HTTPS or SSH remotes; it acts explicitly on that repository and does not use other repository environment variables. An existing tag must point at HEAD, and a remote annotated tag is validated against the commit it points to; tags are never moved and existing Releases are never overwritten. Suite versions with a pre-release suffix automatically produce a GitHub prerelease. Without `-Draft`, the release is published directly.

The scripts finish local verification and packaging before pushing the tag and uploading. If upload fails, the package and the tag are kept and remote records are not deleted automatically; check the actual state on GitHub first, then recover manually. The working-directory check includes untracked files; personal logs or temporary notes can be moved outside the repository or into an ignored directory — you do not need to commit unrelated files to publish.

## CI

GNU differential checks run separately from Verify:

Round 16 adds dedicated NUL, unterminated NUL, binary NUL and consecutive-empty-record shapes to selected cases; the old LF matrix is unchanged. Hash `--tag -z` and all four normal hash verification modes are compared directly. Option-specific skips require a successful reference `--help`; for example Git's GNU cksum 8.32 lacks `--zero`, while the RuxCmd option is covered by known CRC vectors. Startup failures remain failures.

```powershell
./scripts/difftest.ps1
./scripts/difftest.ps1 -Cases rmdir,link,unlink,mktemp,install,cksum,sha1sum,sha512sum
./scripts/difftest.ps1 -Seed 42 -Variants 3
```

The default uses current release binaries (debug fallback) and Git for Windows GNU tools. Build first, or pass `-BinDir` explicitly. Each selected RuxCmd binary must exist; missing GNU tools and declared platform differences are reported as skips. Comparisons cover raw stdout bytes and exit codes; mutating commands use separate fixtures and compare file contents/directory state. Only mktemp's random name is normalized. Native Windows hardlink support is required. SHA check mode runs against real manifests even though MSYS binary-mode generation is exempted. Filesystem stderr wording, ACL equivalence, WTS/utmp identities and long-list formatting are not bytewise compatibility claims.

The CI workflow calls `build.ps1 -Task Verify` and `release.ps1`. The Release workflow checks out an existing version tag, runs the same Verify entry point, then `release.ps1 -Tag <tag> -RequireClean`; it only uploads Actions artifacts and does not perform `-Publish`. Remote publishing requires the maintainer to explicitly run the local publish command.

Script tests:

```powershell
pwsh -File scripts/tests/entrypoint-tests.ps1
pwsh -File scripts/tests/package-tests.ps1
```

The entry-point tests use isolated temporary Git/Cargo projects and cover parameters, compilation failures, real Debug builds, read-only previews, and tag/repository/manifest validation, without contacting GitHub or publishing anything.

## verify-version-info.ps1

Reads all 125 EXEs without launching them and requires the configured suite version, identity, publisher, description, copyright, numeric versions and build flags. Use -BinDir to choose a directory; use -DebugBuild for debug artifacts. Verify and ZIP packaging invoke this gate automatically. Three real RC build regressions are in scripts/tests/version-resource-tests.ps1.
