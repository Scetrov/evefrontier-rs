# OpenSSF Scorecard Alert Dispositions

**Date**: 2026-07-20 run (recorded 2026-07-20)  
**Run score**: 7.6 / 10  
**Baseline run ID**: See `openspec/changes/harden-scorecard-supply-chain-and-input-boundaries/` change record

This document classifies alerts #42-#51 from the 2026-07-20 OpenSSF Scorecard run.
Scorecard alerts are **posture signals that require verification, not confirmed exploitable vulnerabilities**. Each alert below is classified as actionable, accepted risk, detector limitation, or resolved-by-this-change.

## Alert classifications

| Alert # | Check | Classification | Remediation | Residual risk |
|---------|-------|----------------|-------------|---------------|
| #42 | `Token-Permissions` / `Pinned-Dependencies` (mutable Rust builder in `service-route/Dockerfile`) | **Resolved by this change** | Pin `rust:1.97.0-bookworm` by multi-arch manifest-list digest | None — digest pins are immutable and refreshable via Dependabot |
| #43 | Same as #42 for `service-scout-gates/Dockerfile` | **Resolved by this change** | Pin `rust:1.97.0-bookworm` by multi-arch manifest-list digest | None |
| #44 | Same as #42 for `service-scout-range/Dockerfile` | **Resolved by this change** | Pin `rust:1.97.0-bookworm` by multi-arch manifest-list digest | None |
| #45 | Mutable distroless runtime in `service-route/Dockerfile` | **Resolved by this change** | Pin `gcr.io/distroless/cc-debian12:nonroot` by manifest-list digest | None |
| #46 | Mutable distroless runtime in `service-scout-gates/Dockerfile` | **Resolved by this change** | Pin `gcr.io/distroless/cc-debian12:nonroot` by manifest-list digest | None |
| #47 | Mutable distroless runtime in `service-scout-range/Dockerfile` | **Resolved by this change** | Pin `gcr.io/distroless/cc-debian12:nonroot` by manifest-list digest | None |
| #48 | `Branch-Protection` (partial 3/10 — no strict required checks) | **Resolved by this change** | Require up-to-date branches; require stable `Security audit` context; preserve PR-only, no-bypass, no-deletion, no-force-push | None — once ruleset change lands |
| #49 | `Code-Review` (no independent recent review; single maintainer) | **Accepted single-maintainer risk** | Cannot be remediated while only one independent human maintainer exists | Account-compromise residual risk; see `docs/threat-model.md § Change governance and review`. Activated when a second maintainer joins or higher-impact deployment adopted |
| #50 | `Best-Practices` (no OpenSSF Best Practices registration) | **Resolved by this change** | Register, complete questionnaire, publish honest badge | Ongoing evidence maintenance |
| #51 | `Fuzzing` (no detected custom fuzzer integration) | **Detector limitation / accepted risk** | Adds native Rust cargo-fuzz setup via Nx and scheduled workflow; Scorecard's custom-function detector does not support Rust natively | OSS-Fuzz/ClusterFuzzLite onboarding is a separate future operational decision; cargo-fuzz setup is recorded as evidence |

## Evidence references

Each disposition links to a concrete repository artifact:

| Category | Artifact |
|----------|----------|
| Container digest pins | `crates/evefrontier-service-*/Dockerfile`, `Dockerfile.release`, `docs/RELEASE.md § Container base-image digest pinning` |
| Dependabot refresh | `.github/dependabot.yml` (docker ecosystem, `docker-base-images` group) |
| Ruleset hardening | See the `mainline` ruleset change in GitHub settings; evidence captured in the PR that lands this change |
| Threat-model governance exception | `docs/threat-model.md § Change governance and review` |
| Fuzz target setup | `fuzz/Cargo.toml`, `fuzz/fuzz_targets/`, `fuzz/project.json`, `.github/workflows/fuzz.yml`, `docs/TESTING.md § Fuzz Testing` |
| Input-boundary fixes | `crates/evefrontier-lib/src/fmap.rs`, `crates/evefrontier-lib/src/spatial.rs`, `crates/evefrontier-lib/src/graph.rs`, `crates/evefrontier-lambda-shared/src/requests.rs` |

## Re-evaluation triggers

- **Scorecard #49 (Code-Review)** — re-evaluate when a second independent maintainer joins or a materially higher-impact deployment is adopted. At that point, activate CODEOWNERS, required approvals, and latest-push approval.
- **Scorecard #51 (Fuzzing)** — re-evaluate if OSS-Fuzz/ClusterFuzzLite is adopted or if a future Scorecard release adds Rust native-fuzzer detection.
- **Any closed alert returning** — if a subsequent run re-emits a previously closed alert, the maintainer must verify whether the remediation regressed or the finding is a false positive before re-classifying.
