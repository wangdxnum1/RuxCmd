# Development and building

**English** | [简体中文](development-zh.md)

## Project structure

The root virtual workspace manages `crates/*`. Currently 137 packages produce 134 commands; `ps-sys` and `ruxcmd-core` are runtime library packages and `ruxcmd-build` is build-only, `ps-bin` produces `ps.exe`, and `ls` also provides a library reused by `vdir`. Cargo metadata is the single source of truth for the build and release tool inventories; smoke policies must stay consistent with that binary set.

Editions 2021/2024 and each crate's original version are preserved. The root `Cargo.lock` pins the whole suite's dependencies, with resolver 3. The ps packages keep their originally declared Rust 1.75 metadata, but the repository as a whole is only validated against the pinned 1.99.0; no suite-wide MSRV is promised.

Cargo commands can run with `--workspace` at the root or inside a single crate directory; both use the root lockfile and target. The default debug artifact path can be influenced by user Cargo configuration; release commands explicitly use x86_64-pc-windows-msvc.

## Checks and tests

The unified entry point is `scripts/build.ps1`; `-Task Verify` runs the same complete pipeline as CI, and parameters are documented in the [script usage](../scripts/README.md). Rust integration tests cover representative behavior across the suite, including all 14 Round 15 commands. File tests use tempfile; native process, hardlink and ICMP tests need a normal Windows environment (a restricted sandbox can deny these APIs). Traceroute uses loopback and bounded TEST-NET probes; multi-router network behavior is not asserted. `scripts/difftest.ps1` separately compares GNU outputs/exit codes and, for mutating commands, isolated filesystem snapshots.

Release smoke checks only run help entries that were reviewed in source, plus true/false exit codes. Every process has a timeout, reads output and errors asynchronously, and on timeout terminates that process tree. Passing the help check proves the startup/argument entry point works; it does not prove all business behavior.

The five `open` desktop integration checks are marked `#[ignore]`: they launch Explorer, file associations, command windows, or a browser. They do not run during ordinary `cargo test`, Verify, or release validation. To run them manually on a test desktop, use `cargo test -p open --test cli --target x86_64-pc-windows-msvc --locked -- --ignored`. Explorer may navigate after temporary test paths have been deleted and show a location-unavailable dialog; do not include these interactive checks in unattended verification.

Paths are resolved against the PowerShell current directory, so relative BinDir/OutputDir values keep working even after Set-Location when the .NET process directory has not been synced. Process waits, stdin writes, and output pipes share a common deadline; a Windows Job Object owns the lifetime of the spawned child tree, including descendants that outlive a parent already exited while still holding output pipes.

The 21 tests in `scripts/tests/package-tests.ps1` cover failure conditions such as paths containing spaces, relative paths after Set-Location, arguments, timeouts, descendants inheriting output pipes, blocking stdin, large outputs, ps-bin/ps naming, manifest omissions, stale artifacts, checksum tampering, release tags, and refusing overwrites.

## Packaging implementation

The user entry point is `scripts/release.ps1`, which produces a local package by default; only `-Publish` performs the GitHub release. The underlying `scripts/package.ps1` parses Cargo JSON compiler-artifact messages and collects only exes that exactly correspond to workspace binary targets. Every run uses a unique staging directory and never fuzzy-matches files from old target or dist directories.

PE import dependencies are checked with MSVC dumpbin; non-system DLLs and dynamic VC runtime dependencies are rejected, and each exe's dependencies are recorded in the manifest. The static CRT configuration lives in `.cargo/config.toml`, but the actual check results are what a release relies on.

The manifest contains suiteVersion, target, rustcVersion, sourceCommit, dirty, binaries, and files. SHA256SUMS covers the exes and bundled docs; the manifest and the checksum index do not verify themselves recursively, and the external `.sha256` file verifies the whole ZIP. After generation, the ZIP is extracted to a separate temporary directory, the full file set and checksums are compared, and all smoke plus representative behavior checks run again.

Existing packages of the same version are never overwritten. By default the scripts only clean staging directories created by the current run and failed archives of the current run; historical release files are not deleted.

## Known issues

- Historical code contains unused imports/variables, dead code, and Clippy style warnings; the baseline gate is that check commands exit successfully, to be cleaned up incrementally.
- Behavior testing surfaced and fixed the cp/mv source-argument requiredness declarations and grep's no-match exit code; csplit's overflow check was changed to checked_add.
- `wc -l` reading stdin emits a `<stdin>` label, which differs from common GNU output; tests currently preserve that behavior.
- `whereis` still contains historical fixed search paths; its behavioral compatibility needs a dedicated fix, and the build/release pipeline does not depend on them.
- Old per-command target caches left over from the migration are not release inputs and are ignored; maintainers can clean them manually when disk space is needed.
- Windows ARM64, x86, minimum Windows versions, and full GNU/POSIX compatibility remain unverified.
- For remote CI, real results are whatever GitHub Actions reports; local command success does not mean the remote workflow has run.

## Windows executable properties

All 134 commands embed VERSIONINFO during ordinary Cargo debug and release builds. Windows SDK RC is required; resource compilation failures stop the build. The shared build entry is `build/windows-version.rs`, backed by the build-only `ruxcmd-build` library. Resources link only to the named binary target, so `ps-bin` yields `ps.exe` and `vdir` does not inherit the identity of its `ls` library dependency.

The root `workspace.metadata.ruxcmd.version` supplies ProductVersion; FileVersion and both fixed numeric versions use major.minor.patch.0. ProductName, CompanyName, and copyright come from `workspace.metadata.ruxcmd.windows`; FileDescription comes from each package description, while InternalName/OriginalFilename use the executable target. Pre-release and debug flags reflect the build. Changes to the root or package manifest trigger resource regeneration; no timestamp or local source path is embedded.

Verify checks every executable through `scripts/verify-version-info.ps1` without launching it. Packaging checks the fields before hashing and again after extraction, and records them as `binaries[].versionInfo` in manifest.json. Missing or stale fields, mismatched identities, fixed numeric versions, and build flags are rejected. Four resource cases extend package-script tests to 21; three independent-workspace RC compilation cases cover Unicode/quotes/backslashes, spaced paths, binary aliases, root-version-only rebuilds, hyphens in SemVer build metadata, and debug/release flags.

These descriptive properties are independent of Authenticode signing. This implementation does not create certificates or change trust stores; executables remain unsigned. To inspect without opening Explorer:

```powershell
(Get-Item ./target/x86_64-pc-windows-msvc/release/ls.exe).VersionInfo |
    Format-List FileDescription,ProductName,CompanyName,FileVersion,ProductVersion,OriginalFilename,LegalCopyright
pwsh -File scripts/verify-version-info.ps1
```

## ps structure and refactor acceptance

The ps-specific model, filters, sorting, tree and column formatting live inside
`ps-bin`; `ps-sys` remains shared with `pgrep` and `pkill`. Columns use a static
42-name registry and four formatter groups, without per-column trait objects.
The existing limitations (including c/cp placeholders) are preserved. Unit tests
contain the old all-column metadata/value fixture; the standard Verify pipeline
covers these tests, all commands, native version resources and packaged behavior.

For an optional old/new acceptance comparison, retain the old executable in a
separate directory under the same `ps.exe` filename, then run:

```powershell
pwsh -File scripts/tests/ps-refactor-compare.ps1 -BaselineBinary target/ps-before-simplification/ps.exe
```

The comparison queries its own process and checks stdout/stderr/exit codes and
file output; it never signals processes or opens desktop windows. Source plans
under `docs/ps/docs/superpowers` are historical, not the current crate layout.
