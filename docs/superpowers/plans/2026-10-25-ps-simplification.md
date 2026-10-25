# ps Simplification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Reduce ps package/module/column scaffolding while preserving existing observable behavior.
**Architecture:** Keep shared ps-sys; internalize ps-core modules in ps-bin; static column metadata plus four formatter groups.
**Tech Stack:** Rust, existing clap/bitflags/Win32 dependencies, PowerShell acceptance checks.
**Spec:** ../specs/2026-10-25-ps-simplification-design.md

## Global Constraints
- Preserve all 42 registry names/order, column metadata and existing CLI outputs/exit codes.
- Preserve ps-sys and ps/pgrep/pkill consumers; no new features or unrelated bug fixes.
- Preserve Windows resource/packaging gates; no GUI tests or signing.

## Review Focus
- Alias headers and sort matching must preserve canonical ColumnKind matching.
- c/cp currently have no previous sample and must continue returning '?'.
- Missing data and SID/SYSTEM user formatting must match the baseline.
- Move existing tests rather than losing their coverage with ps-core deletion.
- Package naming, current docs and locked dependency inventory must match the new structure.

### Task 1: Characterize and refactor
- [x] Preserve old ps executable; capture deterministic all-column fixture and registry metadata.
- [x] Write/run the behavior contract against old ps-core, then migrate it with the code.
- [x] Move core modules into ps-bin; replace trait factories with static column definitions and grouped formatters.
- [x] Remove unused APIs and ps-core crate/dependency; update Cargo.lock.
- [x] Run focused ps/ps-sys/pgrep/pkill tests and old/new CLI acceptance comparisons.
- [x] Commit the tested refactor.

### Task 2: Docs, review, acceptance
- [x] Update current bilingual structure docs and progress ledger (128 packages, 125 commands).
- [ ] Request one independent whole-change review and resolve important findings.
- [ ] Run full Verify, all version resource checks and clean local ZIP validation.
- [ ] Record acceptance evidence and finish on a clean feature branch.
