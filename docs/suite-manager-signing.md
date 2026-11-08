# RuxCmd Suite Manager Code Signing Plan

[**English**](suite-manager-signing.md) | 简体中文

> Status: recommended plan (pending maintainer confirmation). Written 2026-11-08.
> Applies to: the Suite Manager Tauri installer/executable; also covers existing RuxCmd release assets (setup.exe and the 142 exes) where relevant.
> Context: [Design](suite-manager-design.md) §7 "Security and trust chain".

## 0. Correcting a common misconception

**Signing does not immediately remove the first-run SmartScreen warning.**

Microsoft's [SmartScreen documentation](https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/) is explicit: it is **reputation-based** — it checks whether the file / signing certificate / URL has established reputation; without reputation it warns. Therefore:

| Approach | Effect |
| --- | --- |
| Unsigned | Always warns, and the user sees "Unknown publisher" |
| OV standard / free OSS certificate | Correct publisher shown, but SmartScreen may still warn initially; fades as download volume and time build reputation |
| EV certificate | Usually establishes reputation faster (historically near-instant), but higher cost and eligibility bar |
| Submit file to Microsoft for review | Remediation channel for false positives, [submission portal](https://www.microsoft.com/wdsi/filesubmission/) |

The real benefits of signing are: **verifiable publisher identity + integrity + timestamp + a starting point for reputation**, plus making winget / enterprise environments / users willing to install. The design document's claim that "an unsigned UI breaks the trust chain" holds — but the plan must not promise "no more warnings after signing".

## 1. Recommended: SignPath Foundation (free for OSS) + SignPath.io CI integration

### 1.1 Why

RuxCmd matches SignPath's conditions closely:

- MIT license (OSI-approved), public GitHub repo, continuously released (v0.5.11, 11 releases in one day) → satisfies the [SignPath Foundation terms](https://signpath.org/terms) for OSS License / Released / Maintained / Documented.
- **No proprietary components**, no hacking tools, no user-data collection, installer provides uninstall → satisfies the remaining constraints.
- **Free** ([signpath.org](https://signpath.org): "For OSS projects, our services are free of charge"), with no certificate purchase or renewal cost.
- The private key is generated and held in an **HSM** by SignPath, never on a developer machine or CI runner — solving the classic "you can't plug a USB token into a cloud build" problem.
- Signing **verifies the binary was automatically built from the repository source**, which aligns naturally with the project's provenance / per-file checksum / gate methodology: it upgrades "manifest `sourceCommit`" into a cryptographically verifiable origin.

### 1.2 Costs and constraints (must be accepted)

| Constraint | Impact on RuxCmd |
| --- | --- |
| Every release needs **manual approval** of the signing request | Matches the existing manually-triggered Release workflow; adds one human step to the automation loop |
| The certificate is issued to **SignPath Foundation**, not Tim / RuxCmd | Users see SignPath Foundation as publisher; the project homepage must state a Code signing policy |
| The homepage and download/release pages must use the term **"Code signing policy"** and list: the free signing provider, team roles (Authors / Reviewers / Approvers), and a privacy statement | One new doc section plus release-page wording; one-time work |
| All signed binaries must have **product name / product version metadata set and enforced** | RuxCmd already has version-resource infrastructure (`ruxcmd-build`); Tauri side needs equivalents in `tauri.conf.json` / Windows resources |
| MFA for all team members (SignPath and GitHub) | Sole maintainer: enable GitHub MFA |
| Only artifacts built from your own source may be signed | Sign only Suite Manager and RuxCmd's own exes; no third-party binaries (none here) |
| Terms are a Draft; SignPath may suspend/revoke | External dependency risk — see §4 fallback |

### 1.3 Implementation steps

1. **Apply** at [signpath.org/apply](https://signpath.org/apply), pointing to `github.com/wangdxnum1/RuxCmd`; state that the project is released, MIT, and has no proprietary components.
2. **Write the Code signing policy**: add `docs/code-signing-policy.md` / `code-signing-policy-zh.md`, and link them from README and release pages using the term "Code signing policy". Include:
   - "Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org)"
   - Team roles (sole maintainer: Authors / Reviewers / Approvers are all the repo owner; may reference GitHub permission groups)
   - Privacy statement: the project **does not transfer information to networked systems unless explicitly requested by the user** — note that Suite Manager's "check for updates" calls `api.github.com`, so this must be listed honestly rather than copying the template's "no transfer" wording.
3. **Configure SignPath.io**: create project, artifact configuration (enforce product name / version metadata), signing policy; connect GitHub Actions so CI build artifacts initiate signing requests.
4. **CI integration**: add a signing job to the Release workflow — upload build artifacts to SignPath → manual approval → download signed artifacts → regenerate `.sha256` → publish.
   - **Order matters**: **sign first, then hash**. The current flow is "package → generate `.sha256` → upload"; inserting signing changes the ZIP / setup.exe bytes, so hashes and `check-manifests` values must be recomputed after signing.
   - Make signing a separate optional job, initially signing **only the Suite Manager installer** so the verified 142-exe packaging chain stays untouched; extend to setup.exe once proven.
5. **Timestamping**: use an RFC 3161 timestamp service so signatures remain valid after certificate expiry. SignPath.io supports external timestamping — enable it explicitly.
6. **Verification**: after release, follow the project's existing closing gate — `Get-AuthenticodeSignature` (Valid status, signer, timestamp) and `Get-FileHash` vs `.sha256` vs GitHub asset digest, three-way.

## 2. Alternatives compared

| Option | Cost | Key management | Eligibility | Verdict |
| --- | --- | --- | --- | --- |
| **SignPath Foundation (recommended)** | Free | Their HSM, zero-touch | OSS / released / no proprietary / manual approval | **Best fit** |
| Azure Artifact Signing (formerly Trusted Signing) | Pay-as-you-go, Basic / Premium SKU; [pricing page](https://azure.microsoft.com/en-us/pricing/details/artifact-signing/) requires a quote | Azure-managed, FIPS 140-3 Level 3 HSM | Needs Azure subscription + identity validation; awkward for individuals | Fallback if SignPath declines, or future enterprise SLA / private-trust needs |
| Self-purchased OV code signing certificate | Annual fee (hundreds of USD, varies by CA) | Since 2023 CAs mandate HSM/token storage; CI still needs a cloud signing service | Individuals can buy with identity validation | When you want the publisher to show your own name; annual renewal + management burden |
| Self-purchased EV certificate | Higher annual fee | Hardware token | Organization validation, hard for individuals | Fastest SmartScreen reputation; poor value for a solo maintainer |
| Stay unsigned + document it | 0 | — | — | Current state; winget manifest already declares unsigned. **Not a final answer** — Suite Manager amplifies the gap |

## 3. Phasing

| Phase | Action | Risk |
| --- | --- | --- |
| Now (P0/P1) | Submit SignPath application, draft Code signing policy, confirm the publisher identity is acceptable | Low; does not block the prototype or read-only build |
| Before P2 (update feature) | Land the CI signing job, initially for the Suite Manager installer only; encode "sign then hash" into the release flow | Medium; changes the Release workflow |
| P3 | Extend signing to RuxCmd `setup.exe`; remove "unsigned" wording from winget / scoop manifests; verify hash chain | Medium |
| Later | If SmartScreen still warns at scale, submit files to Microsoft for review; or evaluate EV | Low |

**Key ordering**: signing need not block the P0 prototype or P1 read-only build, but **must be complete before P2 "update" ships** — for a program that downloads and executes new binaries, signing is a launch prerequisite, not a retrofit.

## 4. Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| SignPath declines (terms include a soft "verifiable reputation" judgement) | Fallback to Azure Artifact Signing or a purchased OV. Emphasize release count, reproducible CI builds, per-file SHA256 and manifest provenance in the application |
| Certificate subject is SignPath Foundation, confusing users | State it clearly in the About "publisher" field and the Code signing policy |
| Manual approval slows high-frequency releases | Releases are already manually triggered; if automated later, evaluate the Azure option |
| Signing changes ZIP/setup bytes, breaking the hash chain | Enforce "sign → recompute `.sha256` → sync manifests → `check-manifests`"; fold this into roadmap 4.2's post-release sync job |
| Terms are a Draft, service may change | Keep the CI signing job pluggable; the hash/verification chain does not depend on a specific provider |
| SmartScreen still warns | Tell users honestly that "signed ≠ no warning"; provide verification steps; submit to Microsoft if needed |

## 5. Acceptance checklist

- [ ] `Get-AuthenticodeSignature <setup.exe>` returns Valid with an RFC 3161 timestamp
- [ ] Post-signing ZIP / setup.exe `.sha256` matches the GitHub asset digest and `check-manifests` values, three-way
- [ ] Homepage and every release page link a "Code signing policy"
- [ ] About page shows publisher and signature status
- [ ] winget / scoop manifests no longer declare "unsigned" (if applicable)
- [ ] Docs state that signing does not guarantee an immediate end to SmartScreen warnings

---

*References: [SignPath Foundation](https://signpath.org/) · [SignPath terms](https://signpath.org/terms) · [Azure Artifact Signing overview](https://learn.microsoft.com/en-us/azure/artifact-signing/overview) · [Pricing](https://azure.microsoft.com/en-us/pricing/details/artifact-signing/) · [Microsoft Defender SmartScreen](https://learn.microsoft.com/en-us/windows/security/operating-system-security/virus-and-threat-protection/microsoft-defender-smartscreen/)*
