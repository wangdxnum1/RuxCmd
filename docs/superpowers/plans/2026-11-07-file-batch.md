# File Batch Reliability Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans task-by-task; implement inline, then request one independent final review.

**Goal:** Complete the approved combined xargs/find design for reliable Windows file batching.
**Architecture:** A pure shared UTF-16 command-budget helper in ruxcmd-core; a streaming xargs tokenizer and serial executor; independent bounded buffers for find -exec +. A real native pipe/copy/hash scenario runs in Verify and after ZIP extraction.
**Tech Stack:** Rust, std::process::Command, PowerShell 7, existing GNU differential runner.
**Spec:** ../specs/2026-11-07-file-batch-design.md

## Global Constraints

- Preserve all uncommitted stable-baseline work. No push, publication, merge, tag or version bump.
- Keep 142 published commands and 145 packages; add no third-party dependency versions.
- Use 8000 UTF-16 code units including quoting, fixed arguments and NUL.
- Explicitly preserve GNU input semantics and the documented Windows/UTF-8 limitations.
- No unrelated find traversal or semicolon-action changes.

## Review Focus

- Quoted/empty/Unicode arguments and backslash expansion must survive actual child argv.
- -n must never bypass length limits; overlong items cannot be truncated or executed.
- Child stdin must not consume the streaming input; early execution and failure must not hang.
- Each find action remains independent and processes all valid paths once after failures.
- Real binary pipelines and extracted ZIPs must be checked, not only mocked functions.

### Task 1: Shared command budget and test probe

**Files:** crates/ruxcmd-core/src/{command_batch,lib}.rs; scripts/tests/support/{batch_probe,argv_probe}.rs.
**Interfaces:** command_batch exports MAX_COMMAND_UNITS, command_units and CommandBudget::{new,can_fit_item,try_add,reset}. The test-only Probe compiles a temporary native executable and reads exact invocation records.
- [x] Add boundary/UTF-16/escaping/prefix tests; observe RED before implementation.
- [x] Implement the helper and run cargo test -p ruxcmd-core --locked --offline.

### Task 2: xargs streaming execution

**Files:** crates/xargs/src/{cli,main,input}.rs, crates/xargs/tests/batching.rs, crates/xargs/Cargo.toml.
**Interfaces:** Task 1 budget governs normal batches and post-replacement commands. Tokenizer reads BufRead into one bounded item; CLI leaves command arguments untouched.
- [x] Add real argv tests for quotes, NUL empties, replacements, child options, empty input, UTF-8, length, statuses and streaming stdin; observe old behavior fail.
- [x] Implement CLI compatibility, tokenizer, batching, default echo and serial child-status handling.
- [x] Run the entire xargs suite and compare relevant GNU cases.

### Task 3: find bounded + actions

**Files:** crates/find/src/main.rs, crates/find/Cargo.toml, crates/find/tests/batching.rs.
**Interfaces:** Task 1 budgets attached to each existing ExecAction id; existing AST controls evaluation; flush batches when full and at finish.
- [x] Add native argv tests for multiple batches/actions, failures, fixed/individual oversize and zero matches; observe failures before changing find.
- [x] Implement independent budgets and invalid-action suppression, preserving semicolon execution and AST semantics.
- [x] Run all find and shared-helper tests.

### Task 4: Scenario, packaging, documentation and verification

**Files:** scripts/tests/file-batch-tests.ps1, scripts/{build,package,difftest}.ps1, scripts/tests/entrypoint-tests.ps1, docs/{compatibility,compatibility-zh,iteration-status,file-batch-usage}.md, CHANGELOG.md, scripts/README{,-zh}.md.
**Interfaces:** file-batch-tests.ps1 -BinDir validates the actual selected binaries and can be called on extracted ZIP contents.
- [x] Add byte-pipe find→xargs→cp and find -exec cp + fixtures with exact content/hash, long batches, Unicode/metacharacter paths and empty matches.
- [x] Wire Verify and ZIP verification, update fixture scripts and GNU matrix; document semantics and limitations.
- [ ] Run fmt/check/Clippy, focused suites, GNU differential, full Verify and actual ZIP extraction checks; inspect results.
- [ ] Obtain independent code review, fix important findings with regression tests, and audit every spec requirement before marking the goal complete.

## Execution record

Implementation runs on the existing codex/stabilize-release-baseline branch to preserve accepted but uncommitted baseline changes. No remote mutation is part of this goal.

### Evidence and decisions

- Core command-budget API: missing-module RED, then boundary/UTF-16/escaping tests GREEN.
- xargs old behavior: all nine initial argv cases RED; find old behavior: three batching cases RED. Final xargs suite now includes 13 integration cases plus unit and legacy cases.
- Oversized replacement template that shrinks after substitution: RED, then post-substitution sizing GREEN. Construction also stops oversized expansion before allocating the full result.
- Verify scenario hook: missing-hook RED, then entrypoint suite 22/22 GREEN.
- Decision: repair common.ps1's stdin encoding, because the real Chinese SHA256-manifest scenario exposed console-codepage conversion. Exact UTF-8 roundtrip RED→GREEN; all child stdin text now explicitly uses UTF-8 without BOM.
- Decision: retain single-command differential directories. An unconditional printf lookup introduced by the new xargs matrix failed a cat-only directory; conditional dependency handling is RED→GREEN (15/15). Empty case selection now fails instead of reporting a zero-case pass.
- Full Verify passed in 201.7s before the final supplementary alias case; final-source Verify is running separately. That supplementary xargs case and the entire 13-case integration suite have passed.
- Complete GNU matrix: 968 pass, 0 fail, 39 explicit skips; rerunning against final differential-runner state.
- Actual local ZIP built and extracted: 142 binaries, manifest/file hashes and current xargs/find binary equality checked; 81-file byte-pipe/copy/SHA256 scenarios passed from extracted binaries. Candidate is version 0.4.9 with dirty=true, not a published release.
- Independent final read-only review: no actionable defects. Reviewer inspected focused/full logs and source; coordinator completes the remaining full-matrix/package audit.
- Review scope decisions: pre-existing semicolon/traversal/output-error and non-UTF-8 Windows pathname behavior remain unchanged, as the design requires; explicit shell expansion and POSIX signal equivalence remain documented limitations. Previously accepted baseline behavior is retained, with its new scenario integration verified.

### Requirement audit

| Requirement | Authoritative evidence |
| --- | --- |
| Quotes, empty/NUL items, Unicode, literal braces, all replacement markers, alias/option precedence | Native probe assertions in crates/xargs/tests/batching.rs; GNU xargs matrix |
| Streaming, bounded memory, child stdin isolation | Early-before-EOF native child test, bounded item reader and bounded substitution unit test |
| UTF-16 sizing including fixed prefix, quote/backslash overhead, -n and overflow | command_batch integration tests; xargs length tests inspect every child argument; find records assert each full batch <=8000 |
| xargs status continuation, 255 stop, input/launch failures | Native by-item child and non-executable/missing-program tests |
| Independent find buffers, zero matches, failed batches, oversized prefix/item | All four find native batching tests plus original 55 tests and complete copy scenario |
| Real pipelines and content integrity, including empty selection and failed consumer | scripts/tests/file-batch-tests.ps1, executed on build output and actual extracted ZIP |
| Gate propagation and UTF-8 harness correctness | 22 entrypoint cases; 22 package-script cases, including Chinese stdin roundtrip |
| 142 published commands, existing version, no new dependency versions | Cargo.lock diff only adds existing workspace/path dependencies; compatibility gate and ZIP manifest report 142 bins/version0.4.9 |
| No pushes, tags, publication, or baseline loss | Existing branch/worktree preserved; only local ZIP generation performed |
