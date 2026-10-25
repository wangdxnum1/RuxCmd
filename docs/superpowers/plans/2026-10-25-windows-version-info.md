# Windows Version Information Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Populate Windows Details for all 125 commands and reject incomplete release binaries.
**Architecture:** Shared build-only helper renders RC and links only named binaries; PowerShell reads and validates native version resources at Verify and package boundaries.
**Tech Stack:** Rust, embed-resource, toml, semver, Windows SDK RC, PowerShell FileVersionInfo.
**Spec:** ../specs/2026-10-25-windows-version-info-design.md

## Global Constraints
- Suite version is the source of truth; no signing or certificate installation.
- All 125 executable targets, including ps and vdir; libraries do not carry executable resources.
- No desktop app or Explorer invocation during verification.
- No new release or merge to main in this implementation task.

## Review Focus
- ps-bin package versus ps binary; vdir must never inherit ls resource identity.
- Cargo cache invalidation when only root suite version changes.
- Unicode, quotes and backslashes in descriptions and paths.
- Pre-release SemVer and Windows 16-bit component bounds.
- Resources must be checked on the extracted archive, not only source binaries.

### Task 1: Shared compiler and all binary hooks
Files: crates/ruxcmd-build/{Cargo.toml,src/lib.rs}, build/windows-version.rs, Cargo.toml, crates/*/Cargo.toml.
- [ ] Write metadata/version/escaping tests and observe failure before implementation.
- [ ] Implement validated resource rendering and named-binary linking; fill 30 missing package descriptions.
- [ ] Add shared build hooks to 125 packages; leave library-only packages unhooked.
- [ ] Test helper and build ls/vdir/ps in debug and release; inspect resources with FileVersionInfo.
- [ ] Verify root-version-only rebuild and restore metadata.
- [ ] Commit the tested build feature.

### Task 2: Verification and release gates
Files: scripts/common.ps1, scripts/verify-version-info.ps1, scripts/build.ps1, scripts/package.ps1, scripts/tests/package-tests.ps1.
- [ ] Add version-field validation and regression cases; observe rejection of existing unresourced binaries.
- [ ] Wire validation into Verify and package staging/extraction; add manifest version fields.
- [ ] Run positive inventory validation and missing/wrong-version/wrong-command negatives.
- [ ] Commit the tested gate.

### Task 3: Documentation, acceptance, review
Files: CHANGELOG.md, docs/development{,-zh}.md, scripts/README{,-zh}.md, .superpowers/sdd/progress.md.
- [ ] Document fields, suite-version mapping, SDK requirement, unsigned status and 129-package inventory.
- [ ] Run full Verify and package ZIP; inspect all 125 extracted EXEs without opening any desktop windows.
- [ ] Request one independent whole-change review; resolve actionable findings and rerun affected checks.
- [ ] Record evidence and finish with a clean feature branch.