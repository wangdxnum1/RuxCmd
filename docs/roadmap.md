# RuxCmd Direction and Insights

[**English**](roadmap.md) | 简体中文

> Written 2026-11-07. Based on v0.5.10: 142 commands, 1740 tests, 11 releases in one day, all gates green. v0.5.11 is a documentation/progress close-out release (no CLI changes), so the automation and distribution work below shifts to 0.5.12 / 0.6.0.
> This is the maintainer's analysis and mid-term plan; it commits to directions, not dates. Execution log: [iteration status](iteration-status.md); round-by-round ledger: [.superpowers/sdd/progress.md](../.superpowers/sdd/progress.md).

## 1. Positioning: GNU vocabulary, Windows semantics

RuxCmd competes with neither WSL nor Git Bash/MSYS2.

- WSL is a full Linux environment: heavy, isolated, filesystem-crossing. Users who want Linux choose it.
- MSYS2/Git Bash is a POSIX emulation layer: path conversion, line endings and signal semantics pile up drift that makes scripts "look like they work" while behavior shifts at the edges.
- **RuxCmd sits between them**: it brings the GNU command vocabulary (`sort | uniq -c | head`) to native Windows process semantics — real NTFS timestamps, the volume/drive model, Windows' coarse-grained permissions, single-file executables with no runtime dependency. Scripts read like Linux; behavior is faithful to Windows.

Target users: people writing scripts on Windows — CI/CD, operations automation, developers migrating from Linux/macOS, and native users boxed in by PowerShell's thin vocabulary. Their pain is not "missing a shell" but "missing a composable, portable, predictable set of command words."

This positioning drives every trade-off below: **where Windows provides real semantics (timestamps, disk counters, WTS sessions), implement Windows semantics and declare differences honestly; where the vocabulary matters (options, output formats, exit codes), align byte-for-byte with GNU.**

## 2. Where we are: an inflection point

Quality metrics now lead adoption metrics, and the gap keeps growing:

| Dimension | State |
| --- | --- |
| Quality | 1740 tests, byte-identical differential runs, Verify green (tests / clippy -D warnings / compat-check / check-manifests / difftest / installer scripts) |
| Features | 142 commands; P0/P1 fully cleared; remaining P2 items are mostly platform-inherent limits |
| Releases | 11 successful releases in one day; notes auto-generated; assets and manifest hashes synced |
| **Distribution** | **Zero** — no winget package, no public scoop bucket, no docs site; installation is a manual download |

Judgment: **the marginal return of writing more code is now lower than the return of getting one more person installed.** Distribution is the multiplier for everything else, which sets the ordering of the tracks below.

## 3. Insights

### 3.1 Differential testing is the moat

The project's most distinctive asset is not volume of code but methodology: **byte-level differential testing against a reference implementation.** It kept proving itself this session —

- The SHA256 `*` separator, GNU binary-mode NUL segment counting, `yesterday = now minus one day` (not yesterday midnight), the dd conversion tables — all came from probing the MSYS2 reference tools, not from reading docs.
- It also prevents the reverse error: two claimed gaps in the compat table were disproven by probing (sed nested groups and find relative dates were already implemented).

Institutionalize it: **before claiming "X is unsupported", run the reference tool once; before claiming "X is supported", build a byte-identical comparison case.** For commands with no reference tool (zip, iostat, sar), build golden vectors and state the provenance in the docs.

### 3.2 Stability comes from closed loops, not good intentions

The v0.5.5/v0.5.6 manifest SHA256 chain break is the cycle's defining lesson: an update script failed, the failure was misread as success, and two releases shipped with a stale hash — checksum-verified installs necessarily failed, and no gate shouted.

The countermeasure is in place (check-manifests in Verify, mandatory post-release .zip.sha256 download and sync), but the principle deserves recording: **any step that "a human promises to do" must eventually become a gate that "the machine refuses to skip."** Remaining manual steps in the release flow (4.2) all get this treatment.

### 3.3 Gap lists must be probed

The compatibility table is a promise document, but "gaps" rot too — implementation moves ahead of documentation, or the gap never truly existed. Two claimed gaps were disproven this round. Countermeasure: gap lists carry a probe date; unverified gaps older than one release automatically downgrade to "needs verification."

### 3.4 142 standalone executables is a trade, not a compromise

Single-file native executables buy: on-demand distribution, no PATH shadowing (the rux- prefix), no runtime dependencies, crash isolation. The costs: process-startup overhead ×142 and shared code discipline concentrated in ruxcmd-core.

The architecture is not changing, but its cost must be presented **with data** (see 4.3): "are 142 exes slow to start" is the most-asked question about this design, and the answer should be a comparison table, not a feeling.

### 3.5 Windows fidelity is the differentiator, not a defect

Real NTFS timestamp fields for timestamps, IOCTL_DISK_PERFORMANCE for disk counters, WTS enumeration for sessions — the places where this is "less precise than POSIX" are exactly where the project's credibility lives. Every "equivalent" in a POSIX emulation layer is a future behavioral drift. Principle: **use what Windows gives, declare what it cannot, never fabricate.**

## 4. Four development tracks

### 4.1 Distribution and visibility (highest priority)

| Action | Content | Cost |
| --- | --- | --- |
| winget-pkgs submission | Submit the scripts/packaging/winget manifests to the microsoft/winget-pkgs community repo; enables `winget install` | Half a day + PR round-trip |
| scoop bucket | Separate manifest repo; users `scoop bucket add ruxcmd` | 2 hours |
| Docs site | Publish the existing bilingual docs via GitHub Pages; a fixed home for release notes | Half a day |
| 1.0 signal | After distribution lands and the gap list freezes, tag 1.0 — the version number itself becomes an adoption signal | — |

### 4.2 Closing the release automation loop

Mechanize the remaining manual steps:

- After the Release workflow succeeds, run "download .zip.sha256 → sync manifest hashes → check-manifests → commit/push" automatically (a post-release sync job)
- Re-entrant by design: one manifest commit per released version is already the verified convention

### 4.3 Quality deepening (defensive investment)

- **difftest argument-combination expansion**: cross-option cases for high-frequency commands (sort/grep/cut: `-c -z`, `-k2 -r -u`, …) with a fixed seed for determinism
- **Property tests for parsers**: bc/find/awk syntax layers hold the invariant "no input panics or loops forever"
- **Performance baseline**: cold-start table for 142 exes vs GNU/PowerShell; pipeline throughput
- **ARM64 CI** (optional): windows-11-arm runners to verify portability

### 4.4 Infrastructure consolidation (debt)

Three duplicated implementations move into ruxcmd-core — one fix fixes every consumer:

- **Size-suffix parsing**: du / truncate / dd each carry their own K=1024/KB=1000 table
- **Date-time parsing**: touch / find each carry one (find's relative-date grammar should flow back to touch `-d`)
- **Epoch conversion**: zip / touch / timefmt hold three DOS↔Unix conversions

## 5. Non-goals

- **No more blind P2 sweeps**: remaining items wait for real user feedback, or get removed when probing disproves them.
- **No byte-level archive-layout alignment**: zip has no reference tool; the investment is bottomless. Semantic equivalence suffices.
- **No platform-impossible features**: mktemp Unix permissions, persistent mkfifo FIFOs and the like stay declared differences.
- **No POSIX emulation layer**: paths, line endings and signal semantics are not emulated — that is the boundary with MSYS2.
- **No new commands without demand signals**: new commands come from user feedback or scripting necessity (jq is the only candidate on the watch list, but it is a standalone project to be evaluated separately).

## 6. Version roadmap

```
v0.5.11   Documentation/progress close-out (done, no CLI changes)
v0.5.12   Release-automation close-out (4.2) + infrastructure consolidation (4.4) — pure internal quality
v0.6.0    Distribution lands (winget PR, scoop bucket, docs site) — the adoption release
v0.6.x    Quality deepening in batches (difftest expansion → property tests → performance baseline)
1.0-rc    Gap freeze + full regression + docs final pass
1.0       Commitment freeze: CLI behavior, manifest structure, release artifact chain
```

1.0 does not mean "feature-complete"; it means **commitment freeze**: breaking CLI changes thereafter require a major version, and user scripts may rely on the surface.

## 7. Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| Single maintainer (bus factor = 1) | Docs and automation lower maintenance friction; ledger and gates let anyone reproduce the process |
| windows-sys frozen feature set breaks down | New APIs prefer direct windows_link declarations (precedents in touch/dd/diff); feature additions need a full Verify run |
| Commands without reference tools drift | Golden vectors + property tests; provenance declared in docs |
| Upstream dependency breakage (zip/regex/clap) | Version upgrades are their own round; full Verify regression before merge |
| Manifest/hash chain break recurrence | check-manifests gate + automated post-release sync (closed once 4.2 lands) |

## 8. Measuring success

- **Adoption**: winget installs, GitHub stars/issues, download counts
- **Quality**: difftest command and argument-combination coverage, parsers under property tests, gate pass ratio
- **Engineering**: number of manual steps in the release loop (target: 0), Verify duration, first-response time on issues
- **Architecture**: ruxcmd-core module count vs duplicated implementations (target: zero new duplication)

---

*This document updates with major direction changes; round-by-round execution details live in the ledger.*
