# Release baseline stabilization

The approved next iteration prioritizes reliable Windows daily command-line and scripting use. The 0.4.9 audit found that CI rejects three missing compatibility-table options while Release publishes without that gate. A local check of older release binaries hid those differences.

## Scope and approach

Keep the existing PowerShell build entry point. `build.ps1 -Task Verify` must run compatibility checks immediately after building and checking native version resources, passing its exact binary directory. Both CI and Release already call Verify, so this makes the gate common to both and to local publishing. Failure must stop subsequent checks and packaging. Debug and custom Cargo output directories must remain supported; Offline must reach the child check.

Update both option inventories for cut, diff and mktemp, and synchronize the Chinese diff behavior description. Use the existing pinned rust-cache action in both workflows with workspace-crate caching disabled. Add a version-and-commit progress index, preserving historical round labels as archival references.

## Alternatives

- Add a separate compatibility step to Release: smallest workflow edit, but leaves local Verify and publishing inconsistent.
- Put the gate in Verify (selected): one ordered validation path, with the configuration and binary directory already available.
- Introduce another release coordinator: unnecessary duplication for this change.

## Constraints and validation

No command implementation, dependency version, suite version, tag or published asset changes. Keep current command behavior, including diff rejecting recursive comparison. Work on a dedicated branch in the current checkout and leave changes available for review.

Add subprocess regression cases proving Release and Debug directory forwarding, Offline forwarding, and fail-fast behavior. Update the existing Debug package-check fixture to account for the new gate. Run compatibility checks against fresh release binaries, the entrypoint suite and full Verify. GNU differential checks remain separate, since reference tools are optional and platform-dependent.
