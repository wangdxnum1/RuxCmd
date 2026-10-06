# Infrastructure execution ledger

Plan: superpowers/plans/2026-10-06-project-infrastructure.md

- Start: 2026-10-07, base 3934fdd; native execution selected by maintainer.
- Ruling: Work in the current checkout on codex/project-infrastructure rather than copying to a worktree. The approved migration includes uncommitted command implementations; a new worktree would omit them. Existing changes are retained. Cost: the current checkout changes structure, isolated by branch rather than by directory.
- Ruling: Migration leaves ignored old target caches in place; they are not published. Moving only project content avoids moving large caches and preserves rollback evidence. Cost: old caches consume disk until explicitly cleaned later.
- Pre-flight: Tasks 2/3 consume Task 1 Cargo metadata; Task 4 consumes Task 3 smoke entry; Task 5 reuses Task 4 package parameters. Interfaces are consistent.
- Ruling: Initial migration, formatting, and the first behavior regressions share one baseline commit. Files were hash-verified before formatting, and regression tests observed failures before fixes. Cost: reviewers should use rename-aware/whitespace-insensitive diffs to inspect the four functional changes.
- Task 1: metadata verified, 113 packages and 111 unique binaries; 463 migrated files hash-verified.
- Task 2: locked check, full release build, Clippy and fmt pass. Clippy exposed csplit arithmetic overflow-check syntax, corrected with checked_add. Historical warnings remain visible.
- Task 3: full Rust tests pass after cp/mv required-source declarations and grep no-match exit-code fixes (failures recorded before fixes). All 111 release smoke checks pass.
- Task 4: 15 packaging failure-path cases pass. Actual ZIP built and extracted; full file checksums, 111 smoke checks and 8 representative behaviors pass. All 111 executables passed PE DLL dependency inspection.
- Task 5: workflows and contribution docs written; validation and fresh review pending. Remote GitHub Actions have not been run.
