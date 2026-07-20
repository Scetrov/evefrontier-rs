## Context

The 2026-07-20 OpenSSF Scorecard run reports 7.6/10 and leaves alerts #42-#51 open. Six alerts are direct references to mutable `FROM` tags in three service Dockerfiles. The remaining checks report partial branch protection (3/10), no independent recent review, no OpenSSF Best Practices registration, and no detected fuzzing. These are posture signals rather than CVEs, but they intersect real trust boundaries documented in `docs/threat-model.md`: CI/release inputs, external datasets, compressed tokens, binary sidecars, and service requests.

The repository already has strong baseline controls: SHA-pinned Actions, an active no-bypass `mainline` ruleset, PR-only changes, required checks, CodeQL, dependency audits, Trivy, SBOM generation, provenance, and cosign signing. It also has exactly one direct human collaborator, so mandatory independent approval cannot currently be satisfied honestly.

Fuzz-target reconnaissance identified three deterministic defects suitable for immediate correction: fmap waypoint-count truncation, unchecked spatial-index file-size subtraction, and unchecked Lambda-controlled spatial-neighbor arithmetic. Broader gzip/zstd/ZIP resource ceilings require compatibility and performance data and are not yet ready to standardize.

## Goals / Non-Goals

**Goals:**
- Make every container base used by local/service and release builds immutable and maintainable.
- Strengthen enforceable branch freshness and security checks without creating an impossible self-review policy.
- Record the single-maintainer review exception and a concrete trigger for removing it.
- Add bounded, repeatable Rust fuzzing around the highest-value custom parsers.
- Correct the three proven input-boundary arithmetic/truncation defects.
- Publish honest OpenSSF Best Practices evidence and disposition every Scorecard finding by risk, not by score alone.

**Non-Goals:**
- Achieving a 10/10 Scorecard or forcing every alert closed.
- Treating AI/bot review, self-owned CODEOWNERS, or a second account as independent review.
- Requiring OSS-Fuzz or ClusterFuzzLite in this change.
- Pursuing OpenSSF silver or gold status.
- Defining universal gzip, zstd, ZIP, request-body, or dataset size ceilings before measuring valid workloads.
- Changing normal route results, public response schemas, deployment authentication, or the accepted RustSec advisory policy.

## Decisions

1. **Pin readable tags to multi-architecture digests at every container build boundary.** Each service Dockerfile will use `image:tag@sha256:<manifest-list-digest>` for both the Rust builder and distroless runtime. The inline release Dockerfile will become a checked-in release Dockerfile using the same pinned distroless reference so it is reviewable and updateable. A manifest-list digest preserves the existing amd64/arm64 build matrix. Tag-only pinning was rejected because tags remain mutable; architecture-specific digests were rejected because they break the multi-architecture release path.

2. **Pair immutability with automated refresh and validation.** Dependabot's Docker ecosystem will cover the three service Dockerfile directories and the checked-in release Dockerfile directory, grouping the same image update across directories where supported. Implementation must prove that an update changes both the readable tag and/or digest as appropriate and that builds still resolve for amd64 and arm64. If the updater cannot refresh a reference, the release documentation will identify the exact manual digest-resolution and verification command. Unmaintained fixed digests were rejected because they exchange substitution risk for patch latency.

3. **Tighten only branch controls that remain meaningful for one maintainer.** The active `mainline` ruleset will keep no bypass actors, PR-only changes, deletion/force-push prevention, squash-only merges, thread resolution, linear history, CodeQL/code-quality gates, and required build/dependency checks. Required status checks will become strict/up-to-date and the stable `Security audit` context will be required. The ineffective code-owner requirement will be disabled while no CODEOWNERS file exists. Required human approval and latest-push approval are deferred until a second independent maintainer exists; enabling them now would block maintenance without supplying independent assurance.

4. **Treat review absence as an explicit residual risk.** The threat model/security governance material will state that owner-authored changes receive CI and optional automated analysis but not independent human review. External contributions are reviewed by the maintainer. The exception is re-evaluated when a second trusted maintainer joins or before the project is used in a materially higher-impact deployment; then one approval, latest-push approval, and CODEOWNERS become required. Fabricated or bot approvals were rejected because OpenSSF correctly does not treat them as a second accountable human.

5. **Use native Rust fuzzing for assurance, not alert gaming.** A standalone cargo-fuzz package/project will provide targets for fmap decoding/round trips, spatial-index byte loading, and local dataset ZIP extraction. Nx will expose non-cacheable fuzz targets, and a scheduled/manual workflow will run bounded durations with committed seed corpora and persist crash artifacts. PRs retain deterministic regression tests rather than long fuzz runs. Standalone cargo-fuzz may not clear Scorecard because its custom-function detector does not support Rust; OSS-Fuzz/ClusterFuzzLite onboarding remains a later operational decision.

6. **Correct proven bounds at the narrowest ownership layer.** Fmap encoding will reject more than `u16::MAX` waypoints before conversion. File-backed spatial-index loading will use checked size arithmetic and return the existing typed load error before allocation. A shared maximum spatial-neighbor policy, aligned with the graph's existing safe cap, will validate Lambda input, while graph construction retains checked/saturating defense against non-Lambda callers. Valid zero/default semantics are preserved where already supported.

7. **Defer broad decompression limits until evidence exists.** Fuzzing will exercise malformed gzip, zstd, and ZIP inputs and record crashes/timeouts, but this change will not invent production limits that could reject legitimate datasets. Follow-up limits must be based on valid fixture/release sizes, deployment memory budgets, and measured expansion ratios.

8. **Publish evidence with qualified claims.** The repository will register for an OpenSSF Best Practices in-progress or passing badge, link the status, and keep answers grounded in repository artifacts. After changes land, Scorecard will be rerun and each alert will be closed, dismissed, or retained with a factual rationale. Code-Review and high-tier Branch-Protection limitations are accepted risks until independent review is possible; a Rust fuzz setup is documented even if the detector remains inconclusive.

## Risks / Trade-offs

- **[Risk] Digest pins stop receiving security fixes** → Configure and test Docker update automation, document manual refresh, and retain weekly dependency review.
- **[Risk] A digest is architecture-specific** → Verify it is a manifest-list digest and build both amd64 and arm64 before merge.
- **[Risk] Strict required checks create merge friction or deadlock on renamed jobs** → Use only stable contexts, test the ruleset on a proposal PR, and document the API rollback.
- **[Risk] Single-maintainer account compromise remains sufficient to approve malicious code** → Preserve no-bypass PR/check enforcement, require phishing-resistant account security outside repository code, record the exception, and trigger independent review controls when staffing changes.
- **[Risk] Fuzzing consumes unbounded CI time** → Use explicit per-target time/RSS budgets, schedule it separately from required PR CI, and keep crash corpora as deterministic regressions.
- **[Risk] Fuzz-discovered resource exhaustion cannot be fixed safely without limits** → Record evidence and open a separate policy change rather than silently choosing incompatible ceilings.
- **[Trade-off] Some Scorecard alerts may remain open** → Prefer real Rust assurance and honest governance over detector-specific scaffolding.

## Migration Plan

1. Add the checked-in release Dockerfile, pin all seven base-image references, configure Docker updates, and validate multi-architecture builds plus image scanning.
2. Apply the ruleset update through the GitHub API/UI, verify it on a pull request, and update the threat model/security governance documentation.
3. Add the three narrow input-boundary fixes and deterministic boundary tests through Nx.
4. Add the cargo-fuzz project, seed corpora, Nx targets, and bounded scheduled/manual workflow; retain any discovered crashes as regression tests.
5. Register and publish the OpenSSF Best Practices status, rerun Scorecard, and record each alert disposition.

Container rollback restores the previous tags but also restores mutability, so it is allowed only for a confirmed digest/platform regression and must be followed by a corrected digest. A ruleset rollback restores non-strict checks if the configured contexts deadlock, while retaining PR/no-bypass/force-push/deletion protections. Rust behavior changes can be reverted normally, but the corresponding malformed input must remain documented as an accepted defect until an alternative fix lands.

## Open Questions

None. Exact current image digests are implementation-time evidence and must be resolved and verified immediately before the change is submitted.
