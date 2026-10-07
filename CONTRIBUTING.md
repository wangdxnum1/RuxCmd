# Contributing

**English** | [简体中文](CONTRIBUTING-zh.md)

Start with the [development notes](docs/development.md) and the [command compatibility notes](docs/compatibility.md).

## Adding a command

1. Create a Rust package under `crates/<command>/`. The root workspace's `crates/*` glob picks it up automatically; binary names must be unique.
2. Define the arguments and behavior, and document the Windows/Linux differences. Do not depend on personal machine paths.
3. Add behavior tests in the crate's own `tests/`; for file-related tests use isolated temporary directories and never touch real user files, system processes, or the network.
4. Add a bounded smoke policy in `scripts/smoke-cases.json`. `--help` is the usual entry, but confirm that parsing exits before any side effect. `true` and `false` assert exit codes directly.
5. Update the compatibility table and the CHANGELOG. Never mark a feature as verified merely because it compiles.
6. Update the root `Cargo.lock` and commit it; CI and releases use `--locked`.

## Commits and pull requests

Organize commits by purpose. Bug fixes come with a reproducing input and a regression test; do not mix pure formatting with functional rewrites. Keep the existing dependency version ranges; explain dependency upgrades separately.

Run `pwsh -File scripts/build.ps1 -Task Verify` before submitting. When release scripts change, run a full local release package and inspect the extracted artifacts. Historical Clippy warnings are not currently promoted to errors; do not hide problems with bulk `allow` attributes.

The PR description states the concrete behavior change, the verification commands and their results, and any unverified platforms or conditions. Unrelated logs and personal configuration do not go into PRs.

## Releases

Maintainers update `workspace.metadata.ruxcmd.version` and the CHANGELOG. From a clean checkout, run `pwsh -File scripts/release.ps1 -Publish`, which verifies everything, creates/validates the matching tag, pushes it, and uploads the GitHub Release; `-Draft` creates a draft first. Alternatively push a tag manually and run Release in GitHub Actions, which only produces the ZIP and `.sha256` artifacts. See the [script usage](scripts/README.md) for parameters and recovery instructions.
