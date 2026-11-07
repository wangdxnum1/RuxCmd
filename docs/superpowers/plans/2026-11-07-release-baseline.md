# Release Baseline Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans to implement task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Make compatibility drift block local Verify, CI and Release consistently.

**Architecture:** Reuse build.ps1's post-build validation sequence and explicit binary directory. Reuse existing workflow dependency caching without caching workspace command artifacts.

**Tech Stack:** PowerShell 7, Cargo, GitHub Actions, Markdown.

**Spec:** ../specs/2026-11-07-release-baseline-design.md

## Global Constraints

- Keep suite version 0.4.9 and all command implementations unchanged.
- No tags, pushes or publication; leave changes reviewable on codex/stabilize-release-baseline.
- Retain Debug, Offline, custom Cargo target directories and failure propagation.
- Preserve historical ledger entries; use version and commit identity for new progress.

## Review Focus

- Compatibility failure must prevent smoke checks and success output.
- Debug Verify must use Debug artifacts even when old release artifacts exist.
- Child metadata checks inherit Offline.
- Both workflows must rebuild workspace command artifacts rather than restore them from cache.
- Documentation must describe recursive diff as explicitly unsupported.

### Task 1: Common validation gate

**Files:** scripts/build.ps1, scripts/tests/entrypoint-tests.ps1, docs/compatibility.md, docs/compatibility-zh.md.

**Interfaces:** Verify invokes compat-check.ps1 -BinDir $binaryDirectory after version-resource validation and before smoke-test.ps1.

- [x] Add subprocess cases for Release/Debug binary directory, Offline inheritance and compatibility failure propagation; update the Debug package-check fixture.
- [x] Run entrypoint-tests.ps1 and observe the missing gate failure.
- [x] Add the gate and fix both option inventories plus Chinese diff behavior text.
- [x] Run the entrypoint suite and compat-check.ps1 against current release artifacts; expect zero failures.

### Task 2: Workflow and documentation alignment

**Files:** .github/workflows/{ci,release}.yml, scripts/README{,-zh}.md, README{,-zh}.md, docs/development{,-zh}.md, docs/iteration-status.md, CHANGELOG.md.

**Interfaces:** Both workflows consume Task 1's Verify gate; progress index links the spec and plan.

- [x] Remove redundant standalone CI validation steps; use pinned rust-cache with workspace caching disabled in Release and a shared cache key covering both workflows.
- [x] Document the common gate and current automatic Release behavior; create a version/commit progress index without renumbering historical rounds.
- [x] Parse PowerShell and workflow YAML; inspect cache settings and both Verify call sites.
- [x] Run full build.ps1 -Task Verify -Offline and inspect its final exit status; record results and obtain final code review.

## Execution evidence

- RED: without the gate, the new Release subprocess test reached smoke-test.ps1 instead of the expected compatibility failure.
- GREEN: entrypoint suite 21/21; independent compatibility check passed; full Verify passed in 180.9 seconds, including the new gate and the 21-case entrypoint suite.
- Full Verify: 1613 Rust tests passed, 19 ignored; 142 version-resource and smoke checks; compatibility tables 142 rows, 127 option checks and 15 explicit skips.
- PowerShell/YAML syntax and workflow Verify/cache settings checked; git diff --check passed.
- Command implementations, Cargo manifests and dependency lockfile unchanged.
- Final independent read-only review: no actionable findings. Review inspected the diff, new documents and successful test logs.
- Review scope: future remote workflow/cache behavior remains unverified until changes are pushed; local syntax/settings checks support the configuration change. Published-asset audit evidence comes from the preceding maintainer-requested verification, not a second download by the reviewer.
