# RuxCmd Suite Manager Design

[**English**](suite-manager-design.md) | 简体中文

> Status: **design finalized (v1, not started)**. Written 2026-11-08, based on suite v0.5.11.
> Stack: **Tauri** (Rust backend + web frontend), Windows x64 / MSVC first.
> This document defines positioning, boundaries and contracts, not dates. For round-by-round execution see [iteration status](iteration-status.md).

## 0. Confirmed decisions (2026-11-08, maintainer)

| # | Decision | Detail |
| --- | --- | --- |
| 1 | **Tauri 2.x stable line** | Pin major 2, do not chase 3.0 alpha; do not rely on `releases/latest` (mixes prereleases) |
| 2 | **No code signing for now** (maintainer decision, 2026-11-08) | Keep the [Code signing plan](suite-manager-signing.md) as **future reference only**: no application, no CI integration, not a P2 prerequisite. Current state matches the winget manifest: unsigned, possible first-run SmartScreen warning; disclose this honestly to users |
| 3 | **Code lives at the repository root as `suite-manager/`** | Own `Cargo.toml`, not under `crates/*`; keeps the 142-command Verify / packaging gates untouched. See §3.1 and §8 |
| 4 | **Current prototype is UI v1; no further iteration for now** | [suite-manager-prototype.html](suite-manager-prototype.html) is the frozen v1: About / Check update / Settings, three update states, progress + rollback guidance; later changes get a new version |

Companion docs: [Code signing plan (not adopted, reference only)](suite-manager-signing.md) ([简体中文](suite-manager-signing-zh.md)) · [UI prototype](suite-manager-prototype.html).

## 1. Positioning

**RuxCmd Suite Manager is the one graphical entry point a user needs to remember: what I installed, whether it is current, and how to move to a new version safely.**

It serves the *installation lifecycle*, not the commands themselves:

- The suite is 142 standalone executables that are nearly invisible once installed — the script installer leaves only `%LOCALAPPDATA%\RuxCmd\bin\.ruxcmd-install.json` plus one user-PATH entry; the Inno installer leaves only one "Settings > Apps" uninstall record (whose icon is `find.exe`). There is no Start-menu entry and no unified `--version` entry point.
- [Direction and insights](roadmap.md) identifies distribution and visibility as the current gap. winget/scoop are borrowed entry points; Suite Manager is the project's own landing surface.
- [scripts/install.ps1](../scripts/install.ps1) already implements a reliable update chain: SHA256 verification → extract → back up to `<InstallDir>.old-<version>` → atomic replacement → PATH/shim repair → rollback on failure. **But there is no way to discover the latest version**; users must open GitHub Releases manually.

Suite Manager therefore adds two things: **presence** (About) and a **version loop** (check + update).

### 1.1 Explicit non-goals

| Not doing | Why |
| --- | --- |
| Command launcher / embedded terminal | 142 CLI tools; clicking buttons to run them adds no value and implies the GUI is the primary interface |
| Options / alias / config editor | The project states differences rather than emulating POSIX; there is no configurable runtime state. Adding config means committing to maintain it |
| Embedded docs browser | Bilingual docs already exist; GitHub Pages is a separate roadmap 4.1 item |
| Reimplementing download / verify / replace | `install.ps1` already does this and is covered by script tests; a second implementation would split the logic |
| Managing start/stop of the 142 commands | Standalone exes: no services, no daemons, nothing to manage |
| Silent background auto-update | Updating replaces executables on the user's PATH; it must be informed and explicit |

In one sentence: **it is the frontend to the installation, not a control panel for the suite.**

## 2. Users and scenarios

| User | Scenario |
| --- | --- |
| Manual ZIP extraction | Wants to know which version is installed, whether it is behind, and how to upgrade |
| `install.ps1` user | Wants to upgrade without typing a command |
| Inno EXE user | Wants a findable RuxCmd presence in Windows |
| Developer migrating scripts/CI | Wants to verify provenance (source commit, hashes, command count) |

## 3. Architecture

```
┌─────────────────────────────────────────────────────────┐
│  WebView (HTML/CSS/JS)                                  │
│  About · Check update · Apply update · Settings          │
│  Renders and interacts only; no direct FS/net/registry   │
└───────────────────────┬─────────────────────────────────┘
                        │ Tauri IPC (typed commands + events)
┌───────────────────────▼─────────────────────────────────┐
│  Rust backend (Tauri commands)                          │
│  ├─ detect.rs   installation probe (read-only)          │
│  ├─ release.rs  GitHub Releases query (read-only)       │
│  ├─ integrity.rs SHA256 / manifest verification         │
│  ├─ update.rs   update orchestration (only write path)  │
│  └─ model.rs    shared serde data structures            │
└───────────────────────┬─────────────────────────────────┘
                        │ reuse existing artifacts, no reimplementation
        ┌───────────────┼────────────────┬──────────────────┐
        ▼               ▼                ▼                  ▼
  .ruxcmd-install   Inno uninstall    GitHub Releases   install.ps1 /
  .json (share)     key (HKCU/HKLM)  REST API          Inno setup.exe
```

**Key principle**: the Rust backend is a thin orchestration layer. Every write ultimately lands on existing scripts/installers; the backend does not unzip archives, edit PATH, or generate shims itself.

### 3.1 Stack decisions

- **Tauri 2.x stable line** (pin the major, do not chase alpha; Tauri 3 is currently alpha and GitHub `releases/latest` mixes prereleases). Rationale: same language as the suite, single-file distribution, no Node/.NET runtime dependency, reuses the project's existing `windows` crate experience.
- Frontend: plain HTML/CSS/JS or a very light framework; no heavy frontend build. This is a CLI tool suite — the UI is not the main battlefield.
- Target matches the suite: `x86_64-pc-windows-msvc`. ARM64 deferred.
- **Outside the suite workspace** (decision 3): at the **repository root as `suite-manager/`**, with its own `Cargo.toml`, not under `crates/*`, so the 142-command Verify / packaging gates stay untouched.
- **Independent version**: the Manager has its own version (e.g. `0.1.0`), decoupled from suite 0.5.x → 0.6.0. About shows both "Manager version" and "Suite version" explicitly to avoid confusion.

## 4. Data model

### 4.1 Installation probe (both install methods must converge)

`detect_installation()` returns one unified structure. Sources are probed in priority order; conflicts are reported, never guessed away:

| Source | Location | Key fields |
| --- | --- | --- |
| Script state file | `<InstallDir>\.ruxcmd-install.json` | `schemaVersion`, `product`, `version`, `target`, `source`, `sha256`, `installedAt`, `installDir`, `binaries[]`, `shims[]`, `backup` |
| Inno uninstall registry | `HKCU` / `HKLM` `Software\Microsoft\Windows\CurrentVersion\Uninstall\{06BD23F3-4290-4F15-AEC6-55FD9DC91B9F}_is1` | `DisplayVersion`, `InstallLocation`, `DisplayName`, `Publisher`, `UninstallString` |
| Release manifest | `<InstallDir>\share\manifest.json` | `schemaVersion`, `suiteVersion`, `target`, `rustcVersion`, `sourceCommit`, `dirty`, `binaries[]`, `files[]` |
| Per-file checksums | `<InstallDir>\share\SHA256SUMS` | `sha256  path` lines |

Measured on this machine (2026-11-08): Inno install, `DisplayVersion=0.5.11`, `InstallLocation=C:\Users\Tim\AppData\Local\RuxCmd\bin\` — the prototype uses this as its real data source.

```rust
// Illustrative, not final types
struct InstallInfo {
    method: InstallMethod,          // Script | Inno | Portable | NotInstalled | Unknown
    suite_version: Option<String>,  // "0.5.11"
    install_dir: Option<PathBuf>,
    source_commit: Option<String>,  // from share/manifest.json
    dirty: Option<bool>,
    target: Option<String>,         // "x86_64-pc-windows-msvc"
    binary_count: Option<usize>,    // 142
    shim_count: Option<usize>,      // 16
    checksum_status: ChecksumStatus,// Verified | Mismatch | Missing | NotChecked
    path_on_user_path: Option<bool>,
    conflicts: Vec<String>,         // both states present, version mismatch, etc.
}
```

`NotInstalled` is not an error: About, update checking, and first-install guidance remain available.

### 4.2 Release info (GitHub REST)

`GET https://api.github.com/repos/wangdxnum1/RuxCmd/releases/latest`

Real fields (v0.5.11, measured):

```json
{
  "tag_name": "v0.5.11",
  "name": "RuxCmd v0.5.11",
  "prerelease": false,
  "draft": false,
  "published_at": "2026-11-07T15:21:54Z",
  "html_url": "https://github.com/wangdxnum1/RuxCmd/releases/tag/v0.5.11",
  "body": "Documentation/progress close-out……",
  "assets": [
    { "name": "ruxcmd-0.5.11-x86_64-pc-windows-msvc.zip",
      "size": 56136068,
      "digest": "sha256:e90ab189…42226c7045",
      "state": "uploaded" },
    { "name": "ruxcmd-0.5.11-x86_64-pc-windows-msvc.zip.sha256", "size": 108, "digest": "…" },
    { "name": "ruxcmd-0.5.11-x86_64-pc-windows-msvc-setup.exe", "size": 16732287, "digest": "…" },
    { "name": "ruxcmd-0.5.11-x86_64-pc-windows-msvc-setup.exe.sha256", "size": 114, "digest": "…" }
  ]
}
```

Notes:

- **Four assets are the release baseline** (README / CHANGELOG promise them). The UI should require all four with `state=uploaded` for the target version, and refuse an update otherwise.
- The `digest` field (`sha256:…`) enables an **independent cross-check** against the `.zip.sha256` sidecar — exactly the post-release gate standardized in 0.5.8, now automated inside the updater.
- Version comparison must use semver, not string comparison; normalize the `v` prefix.
- `prerelease` / `draft`: stable only by default; a setting can opt into prereleases, always labelled explicitly.
- Unauthenticated GitHub API is limited to 60 requests/hour: cache, use `ETag` / `If-None-Match` (reuse on 304), and enforce a minimum manual-refresh interval. When offline show "Could not check", **never "Up to date"**.

### 4.3 Update state machine

```
Idle ──check──> Checking ──┬──> UpToDate
                           ├──> UpdateAvailable(version, assets, notes)
                           ├──> Offline / RateLimited (retryable)
                           └──> Error(reason)
UpdateAvailable ──confirm──> Downloading(progress)
    ──> Verifying(SHA256 + sidecar + asset digest, three-way)
    ──> Installing(invoke install.ps1 / Inno silent upgrade)
    ──> Done(version) / RolledBack(reason)
```

Every failure must state: the reason, whether the existing version was preserved, and the rollback path (`<InstallDir>.old-<version>`).

## 5. Feature specification

### 5.1 About (read-only, zero risk)

- Suite version, Manager version, target platform, Rust toolchain version
- Install method, install directory, PATH status, shim count, command count
- Release provenance: `sourceCommit`, `dirty`, per-file SHA256 self-check result
- License (MIT), homepage, docs, changelog, compatibility table links
- "Copy diagnostics" button: export the above as text for issue reports

### 5.2 Check update (read-only + one network request)

- Current version / latest version / publish time / release notes (matching CHANGELOG section)
- Target asset list and sizes (ZIP ~56.1 MB, setup ~16.7 MB)
- Clearly distinguish: up to date / update available / could not check (offline or rate-limited) / prerelease (opt-in required)
- "Check for updates" is explicit; no automatic network call at startup, or at least disableable

### 5.3 Update (the only write operation)

- Second confirmation listing: target version, files to download, backup location, terminals to close
- Choose the update path by install method:
  - Script install → `pwsh -File install.ps1 -Source <version> -Force`
  - Inno install → download `setup.exe` + its `.sha256`, verify, then silent upgrade (`/SILENT` or hand off to the user)
  - Portable extraction → only "open download page"; never rewrite a user-managed directory automatically
- Download progress, verification result and install progress shown in stages
- On success re-probe installation and refresh About; on failure show rollback guidance

### 5.4 Settings (minimal)

- Receive prereleases (default off)
- Check on startup (default off)
- Check interval / cache policy
- Language (简体中文 / English, matching the project's bilingual convention)

## 6. Tauri IPC contract (commands and events)

| Command | Direction | Purpose |
| --- | --- | --- |
| `get_install_info` | frontend→backend | read-only probe, returns `InstallInfo` |
| `get_manager_info` | frontend→backend | Manager's own version and build info |
| `check_update(include_prerelease)` | frontend→backend | read-only, returns `ReleaseInfo` or a typed error |
| `verify_installation` | frontend→backend | per-file check against `SHA256SUMS`, returns `ChecksumStatus` |
| `prepare_update(version)` | frontend→backend | download to temp dir and verify, returns `PreparedUpdate` |
| `apply_update(id)` | frontend→backend | **the only write command**, called after frontend confirmation |
| `cancel_update(id)` | frontend→backend | cancel download / clean temp files |
| `open_external(url)` | frontend→backend | open in system browser (minimal shell scope) |
| `copy_diagnostics` | frontend→backend | copy diagnostics text |

Events (backend→frontend): `update-progress` (stage + bytes), `update-stage`, `update-log`, `update-finished`.

**Permission constraints** (`tauri.conf.json` capabilities):

- No general `fs` write permission; all file access goes through backend commands with backend-decided allowlisted paths.
- `shell` allows only fixed `pwsh` / `install.ps1` / `setup.exe` invocations with backend-constructed arguments — **never concatenate frontend-supplied strings**.
- `http` restricted to `api.github.com`, `github.com`, `objects.githubusercontent.com`.
- CSP enabled; no remote scripts, no `eval`.

## 7. Security and trust chain (critical)

1. **Unsigned trust-chain risk (known, accepted)**: a program that downloads and executes new binaries shows a SmartScreen warning and "Unknown publisher" on first run when unsigned. [scripts/packaging/winget/wangdxnum1.RuxCmd.installer.yaml](../scripts/packaging/winget/wangdxnum1.RuxCmd.installer.yaml) already declares the installer is unsigned. **Decision 2: no signing for now** — the [Code signing plan](suite-manager-signing.md) is future reference only. P2 is therefore not blocked by signing, but the UI must disclose "unsigned / warning possible" and offer hash verification so users can check integrity themselves; the three-way hash check (next item) stays mandatory regardless of signing. Note signing would not immediately eliminate SmartScreen warnings anyway (reputation-based); see signing plan §0.
2. **Three-way hash verification**: after download, compare ① the `.zip.sha256` sidecar content, ② the locally computed SHA256, and ③ the GitHub asset `digest`. Only install when all three agree. This embeds the "published asset digest ↔ manifest hash" independent check that roadmap 4.2 wants.
3. **TLS only** for HTTP; validate target domains; never downgrade to http when update checking fails.
4. **Never execute downloaded content**: the downloads are a ZIP / setup.exe, and the update is handed to existing scripts; the UI does not parse or load resources from inside the archive.
5. **Least privilege**: default user-level install (matching `PrivilegesRequired=lowest`), request elevation only when the user explicitly chooses "all users".
6. **Failure preserves the status quo**: all writes happen after a backup; on verification failure delete temp files and leave the install directory untouched.

## 8. Relationship to the existing release chain

| Item | Decision |
| --- | --- |
| Include in the 142-exe ZIP | **Not yet**. Re-evaluate once the shape is stable; avoid changing the verified package inventory and the `check-manifests` gate |
| Include in scoop / winget manifests | **Not yet**. Ship as a separate optional release asset |
| Include in `build.ps1 -Task Verify` | **Not yet**. Independent CI for the root-level `suite-manager/`, so the 142-command Verify stays fast |
| Code location | **Repository root `suite-manager/`** (decision 3), own `Cargo.toml` |
| Versioning | Independent (e.g. `0.1.0`), decoupled from the suite version |
| Signing | **Not adopted**; [Code signing plan](suite-manager-signing.md) is reference only. The Release workflow keeps "package → compute `.sha256` → upload" unchanged |
| Provenance | `sourceCommit` shown in About refers to the **suite** source commit, not the Manager's own; the UI must label it clearly |

## 9. Suggested phasing

| Phase | Content | Risk |
| --- | --- | --- |
| P0 Prototype | ✅ **v1 frozen**: [suite-manager-prototype.html](suite-manager-prototype.html), three views + three update states + progress; maintainer approved, no iteration for now | none |
| P1 Read-only | Tauri 2.x skeleton (root `suite-manager/`) + `detect` + `release` + `integrity`; About / Check update usable, **no writes at all** | low |
| P2 Update | `prepare_update` / `apply_update` + progress + rollback guidance; signing does not block, but the UI must show an "unsigned" notice and hash verification results | medium (the only write path) |
| P3 Distribution | Independent packaging, independent CI, possible winget/scoop entry, docs-site link | medium |
| P4 Extras | Tray icon, update notifications, diagnostics export, ARM64 | low |

## 10. Acceptance criteria

- Both install methods (script / Inno) are detected and produce a consistent version; not-installed is not an error.
- Offline, API rate limit, 404, missing assets, hash mismatch, interrupted download, installer failure — each has explicit wording, never a false "up to date", and never damages the existing install.
- After a successful update, re-probe and show the new version in About; on failure the old version still works.
- All network domains are auditable in config; all writes trace back to a single backend command.

---

*This document evolves with the design; implementation details and round-by-round records live in code and the ledger. The companion UI prototype is [suite-manager-prototype.html](suite-manager-prototype.html).*
