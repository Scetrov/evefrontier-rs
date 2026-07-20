# Scorecard proposal scope oracle

## Inherited decisions
- All ten findings are OpenSSF Scorecard posture signals, not confirmed CVEs.
- Repository is public, single-maintainer, with strong existing CI, action SHA pins, release signing/SBOMs, and an active `main` ruleset.
- Six alerts are one concrete cause: mutable Docker base-image tags in three service Dockerfiles.
- Independent approvals are impossible today and must not be fabricated.

## Diagnosis
Prioritize actual risk reduction over maximizing Scorecard:

1. **Include: immutable Docker bases and update path — Medium**
   - Pin both `FROM` images in:
     - `crates/evefrontier-service-route/Dockerfile:2,27`
     - `crates/evefrontier-service-scout-gates/Dockerfile:2,27`
     - `crates/evefrontier-service-scout-range/Dockerfile:2,27`
   - Also pin the generated release image base at `.github/workflows/docker-release.yml:178`; otherwise tagged release images remain exposed despite closing the six alerts.
   - Add Docker dependency automation for all service Dockerfile directories. Validate that the chosen updater refreshes digests, not merely tags; if Dependabot cannot, document the manual digest-refresh procedure rather than silently assuming coverage.

2. **Include: proportionate ruleset tightening — Medium**
   - Enable strict/up-to-date status checks for the existing stable required checks (`Build and test workspace`, `Dependency review`).
   - Preserve no-bypass, PR-only, deletion/force-push prevention, squash-only merges, and thread resolution.
   - Do **not** add `CODEOWNERS`, required approval count, or last-push approval merely for Scorecard points: with one maintainer they provide no independent control and may create an unmaintainable policy.

3. **Include: documented Code-Review / Branch-Protection exception — High residual risk**
   - Record the single-maintainer exception in the threat model/security governance material: direct maintainer changes lack independent review; external PRs still receive maintainer review plus required checks.
   - State a trigger: when an independent maintainer joins, add `CODEOWNERS`, require one approval and latest-push approval, then reassess strict check coverage.
   - Do not dismiss alerts #42/#49 as false positives. They accurately expose residual account-compromise and change-control risk.

4. **Include: OpenSSF Best Practices registration — Low**
   - Register, complete an evidence-backed initial questionnaire, and publish the resulting badge/status.
   - Aim for an honest “in progress” or passing result; do not make silver/gold certification or compliance with every optional criterion a delivery gate.

5. **Include: Rust fuzzing as assurance, not Scorecard closure — Medium**
   - Add `cargo-fuzz` targets and a bounded scheduled/optional CI invocation for parser boundaries:
     - `crates/evefrontier-lib/src/fmap.rs`
     - `crates/evefrontier-lib/src/spatial.rs`
     - dataset ZIP extraction in `crates/evefrontier-lib/src/github.rs`
   - Seed with valid fixtures and assert error-or-valid-result/no panic.
   - Explicitly accept that Scorecard does not recognize standalone Rust fuzzing. OSS-Fuzz/ClusterFuzzLite is a separate operational decision, not a prerequisite for useful fuzz coverage.

6. **Include only narrowly proven defects discovered during target analysis**
   - `crates/evefrontier-lib/src/graph.rs:372`: attacker-controlled Lambda `max_spatial_neighbors` can reach `max_neighbors + 1`; use checked/bounded handling plus regression coverage. **High availability risk.**
   - `crates/evefrontier-lib/src/fmap.rs:220`: waypoint count silently truncates to `u16`; reject oversized input and test round-trip limits. **Medium integrity/quality risk.**
   - `crates/evefrontier-lib/src/spatial.rs:1015-1018`: checked file-size arithmetic before allocation. **Medium malformed-input availability risk.**

## Drift / contradiction check
- Treating required self-approval, a self-owned CODEOWNERS entry, or an AI review as remediation conflicts with the established single-maintainer reality. None supplies independent review.
- Closing the six Docker alerts without pinning `.github/workflows/docker-release.yml:178` leaves the release path mutable.
- Claiming that `cargo-fuzz` clears Scorecard’s Fuzzing alert would be inaccurate; it materially improves assurance but is outside Scorecard’s Rust custom-fuzzer detection.
- Broad decompression/ZIP resource ceilings are not yet ready for this proposal: `fmap.rs:317`, `spatial.rs:1204`, and ZIP extraction have plausible exhaustion risks, but safe limits require compatibility/performance policy and fixture measurement.

## Recommendation
Create one focused OpenSpec change: **`harden-scorecard-supply-chain-and-input-boundaries`**.

Scope it as:
- Docker digest pinning plus verified update maintenance.
- Strict required-check freshness in the existing ruleset.
- Best Practices registration and explicit single-maintainer risk acceptance.
- Focused Rust fuzzing infrastructure.
- The three deterministic overflow/truncation/underflow fixes above.

Defer:
- OSS-Fuzz/ClusterFuzzLite onboarding.
- Mandatory independent approvals until an independent maintainer exists.
- Badge silver/gold pursuit.
- Resource-limit policy for gzip/zstd/ZIP extraction; open a follow-up informed by fuzzing results and real dataset sizes.
- Unrelated accepted RustSec advisory work.

## Risks
- Digest pins become stale without a tested update mechanism.
- Strict checks may add merge friction but preserve a meaningful integrity benefit.
- Single-maintainer account compromise remains a high residual risk.
- Fuzzing alone cannot establish parser resource safety without explicit budgets.
- Scorecard Fuzzing/Code-Review may remain open after this proportionate change.

## Need from main agent
None.

## Suggested execution prompt
No executor handoff is warranted; this is proposal-scoping guidance.