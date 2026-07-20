## Review

### Scope and evidence

`plan.md` and `progress.md` were requested but are absent at the repository root (`ENOENT`). This review therefore used the current checkout and live GitHub API state. `gh` is authenticated and the repository is public.

The ten findings are **all currently open Scorecard code-scanning alerts** (#42–#51), created 2026-07-19 16:01 UTC. The scheduled Scorecard run at 2026-07-20 14:59 UTC completed successfully against `main` commit `33e8b9161c0bddc1449f169370f3a8de6cbe48e4`; successful analysis does not mean its SARIF findings are resolved.

### Per-alert classification

| Alert | Scorecard severity | Classification | Verified evidence and disposition |
|---|---:|---|---|
| #42 Branch-Protection | High | **Accepted single-maintainer limitation** | Live API ruleset `mainline` is active on the default branch, prevents deletion and non-fast-forward pushes, requires PRs, two named checks, linear history, and has no bypass actors. It nevertheless has `required_approving_review_count: 0`, `require_last_push_approval: false`, and non-strict status checks. The alert correctly also reports no `CODEOWNERS`; no such file exists. The collaborators API returns exactly one direct collaborator, `Scetrov` (admin), so an independent required approval cannot currently be satisfied. This is not stale or false-positive. Formally record the high-risk exception; when a second maintainer exists, add `CODEOWNERS`, require at least one approving reviewer and latest-push approval, and enable strict/up-to-date status checks. `docs/threat-model.md:45-48` already states this intended control. |
| #43 Pinned-Dependencies | Medium | **Actionable improvement** | The current alert identifies `crates/evefrontier-service-route/Dockerfile:2`: `rust:1.97.0-bookworm` is tag-pinned but mutable, not digest-pinned. The API supplies the current manifest digest `sha256:8fa55b…c3073`. This Dockerfile is live local-build input via `docker-compose.yml:45-47`; the finding is current, not stale. |
| #44 Pinned-Dependencies | Medium | **Actionable improvement** | `crates/evefrontier-service-route/Dockerfile:27` uses mutable `gcr.io/distroless/cc-debian12:nonroot`; Scorecard supplies digest `sha256:66aa87…359faa`. It is also used by Compose. |
| #45 Pinned-Dependencies | Medium | **Actionable improvement** | `crates/evefrontier-service-scout-gates/Dockerfile:2` has the same mutable Rust builder tag. Compose selects this file at `docker-compose.yml:59-61`. |
| #46 Pinned-Dependencies | Medium | **Actionable improvement** | `crates/evefrontier-service-scout-gates/Dockerfile:27` has the same mutable distroless runtime tag. |
| #47 Pinned-Dependencies | Medium | **Actionable improvement** | `crates/evefrontier-service-scout-range/Dockerfile:2` has the same mutable Rust builder tag. Compose selects this file at `docker-compose.yml:73-75`. |
| #48 Pinned-Dependencies | Medium | **Actionable improvement** | `crates/evefrontier-service-scout-range/Dockerfile:27` has the same mutable distroless runtime tag. |
| #49 Code-Review | High | **Accepted single-maintainer limitation** | The alert reports 0/4 approved recent changesets. API review evidence confirms merged PR #197 (supply-chain changes) had no reviews, and recent merged PR listing reports no review decisions. With only the one direct collaborator above, self-approval cannot provide independent review. The finding is accurate, not stale. Record the temporary exception and recruit/assign an independent reviewer before converting the ruleset to require approval. |
| #50 CII-Best-Practices | Low | **Actionable improvement** | The live alert says the Best Practices API found no effort/badge. This measures public registration rather than a code flaw; it is still accurately open. Register the repository with OpenSSF Best Practices, publish the resulting in-progress/passing badge, and maintain its assertions. Existing controls make this a low-priority governance task, not a dismissal candidate. |
| #51 Fuzzing | Medium | **Actionable improvement** | There is no cargo-fuzz/OSS-Fuzz/ClusterFuzzLite configuration or property-testing dependency in tracked source/workflow files (repository-wide search excluding generated/agent files). The alert is accurate that no fuzz integration exists. It has a detection caveat: the alert documentation says Scorecard currently recognizes OSS-Fuzz/ClusterFuzzLite and only selected non-Rust language-native fuzzers, so a standalone Rust `cargo-fuzz` harness may improve assurance yet still not clear this Scorecard item. |

### Correct

- Scorecard is enabled on pushes to `main`, scheduled weekly, and manually dispatchable; its action and SARIF upload are immutable SHA references (`.github/workflows/scorecard.yml:3-33`).
- Third-party Actions are SHA pinned, including dependency review (`.github/workflows/dependency-review.yml:1-17`). The live repository API also reports Dependabot security updates, secret scanning, and secret-scanning push protection enabled.
- The active `mainline` ruleset already blocks deletion and non-fast-forward updates, limits merges to squash, requires PR-thread resolution, requires `Build and test workspace` and `Dependency review`, and has no bypass actors. Thus #42 is a narrow, material approval/CODEOWNERS gap rather than absence of all protection.
- Docker release has provenance enabled (`.github/workflows/docker-release.yml:187-202`), Trivy scanning with documented unfixed-vulnerability policy (`:220-245`), cosign signing (`:248-279`), and SBOM generation (`:281-313`).

### Blocker

- **High — independent review remains impossible today.** #42 and #49 should not be dismissed as false positives: GitHub API proves the single direct collaborator and zero required approvals, while the most recent security/governance PR #197 has no approval. The practical disposition is a documented, time-bounded single-maintainer exception plus recruiting a second reviewer; it is not an equivalent security control.

### Note

- The six Docker findings are genuine but are not exhaustive of mutable base-image use. `docker-release.yml:175-185` generates a release Dockerfile containing the same unpinned distroless `FROM` at line 178. Pin that reference in the same remediation, and use a controlled digest-update process (for example Docker ecosystem Dependabot configuration) so digest pins do not block security updates. This reconciles reproducibility with the existing docs’ update intent at `docs/SECURITY_AUDIT.md:260-280`.
- For #51, prioritize fuzz/property tests at parser/decompression boundaries: `decode_fmap_token` base64-decodes then unboundedly gzip-decompresses caller input (`crates/evefrontier-lib/src/fmap.rs:307-348`); archive extraction processes ZIP entries (`crates/evefrontier-lib/src/github.rs:931-968`); and spatial-index loading reads and zstd-decompresses serialized data (`crates/evefrontier-lib/src/spatial.rs:1114-1235`). Harnesses should assert no panic, bounded resource behavior, and clear `Err` results for malformed input. Cargo-fuzz plus an OSS-Fuzz/ClusterFuzzLite path is needed if clearing Scorecard itself is a goal.
- No project/source files were modified. The only written file is this required review artifact under the untracked `.pi-subagents/` area.

```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "All ten open Scorecard alerts #42-#51 are individually classified with Scorecard severity, exact GitHub API facts, and file/line evidence where applicable."
    }
  ],
  "changedFiles": [],
  "testsAddedOrUpdated": [],
  "commandsRun": [
    {
      "command": "gh api repos/Scetrov/evefrontier-rs/code-scanning/alerts/{42..51}",
      "result": "passed",
      "summary": "Retrieved all ten live Scorecard alerts, severities, messages, and locations."
    },
    {
      "command": "gh api repos/Scetrov/evefrontier-rs/rulesets/9651727 and gh api repos/Scetrov/evefrontier-rs/collaborators?affiliation=direct",
      "result": "passed",
      "summary": "Verified the active main ruleset and its zero required approvals, plus the single direct collaborator."
    },
    {
      "command": "gh run list/view --workflow scorecard.yml",
      "result": "passed",
      "summary": "Verified the latest scheduled Scorecard run completed successfully on main commit 33e8b916."
    },
    {
      "command": "pnpm nx run-many -t test --all --parallel=1",
      "result": "failed",
      "summary": "Could not start: pnpm reports Command \"nx\" not found (dependencies/Nx executable unavailable locally). No code changed, so no replacement direct test run was needed for this governance-only review."
    },
    {
      "command": "git diff --check; git diff --cached --check; git diff --cached --name-only; git status --porcelain=v1",
      "result": "passed",
      "summary": "No staged files and no tracked diff/check errors; only the permitted untracked .pi-subagents/ artifact area is present."
    }
  ],
  "validationOutput": [
    "Live GitHub API confirms alerts #42-#51 remain open.",
    "No CODEOWNERS file or fuzzing/property-test integration was found in tracked repository content.",
    "No source changes or staged files were introduced by this review."
  ],
  "residualRisks": [
    "High: main can merge without independent approval while one administrator is the only direct collaborator.",
    "Medium: all three Compose Docker builds and the generated release Dockerfile can resolve mutable base-image tags.",
    "Medium: no automated fuzzing covers decompression/archive/binary parser boundaries; Scorecard may not detect cargo-fuzz alone."
  ],
  "noStagedFiles": true,
  "diffSummary": "Read-only governance/supply-chain review; no project/source diff. Required report artifact only.",
  "reviewFindings": [
    "blocker: GitHub ruleset mainline has required_approving_review_count 0 and the sole direct collaborator is Scetrov; #42 and #49 are valid high-severity governance gaps under a documented single-maintainer exception.",
    "medium: #43-#48 correctly identify six mutable Docker FROM tags in Compose-used Dockerfiles; the generated release Dockerfile has an additional unpinned distroless FROM at .github/workflows/docker-release.yml:178.",
    "low: #50 is an accurate missing OpenSSF Best Practices registration, suitable for an administrative improvement.",
    "medium: #51 accurately reflects no detected fuzz integration; add Rust fuzzing and a Scorecard-recognized integration if alert closure is required."
  ],
  "manualNotes": "Root plan.md and progress.md were requested but absent (ENOENT). Nx test validation could not start because the local pnpm environment has no nx executable."
}
```