## Why

The repository has ten open OpenSSF Scorecard alerts: six concrete mutable-container findings and four posture findings covering branch protection, independent review, best-practices evidence, and fuzzing. None proves an exploitable vulnerability, but the image mutability, parser edge cases, and governance gaps identify proportionate opportunities to improve build integrity and input resilience without pretending a single maintainer can provide independent human approval.

## What Changes

- Pin the Rust builder and distroless runtime images by multi-architecture digest in all three service Dockerfiles and in a checked-in release Dockerfile.
- Add automated Docker base-image update coverage and a documented digest-refresh/verification path so immutable pins do not become stale.
- Tighten the existing `mainline` ruleset by requiring up-to-date branches and stable security checks while preserving PR-only changes, no bypass actors, no force pushes, and no branch deletion.
- Record the single-maintainer exception for independent approval, including the residual account-compromise risk and the trigger for enabling required approval, latest-push approval, and CODEOWNERS when a second maintainer exists.
- Register the repository with the OpenSSF Best Practices program and publish an honest in-progress or passing status backed by repository evidence.
- Add focused Rust fuzzing for fmap tokens, spatial-index binaries, and dataset ZIP extraction, orchestrated through Nx and run on a bounded schedule.
- Fix deterministic input-boundary defects found while selecting fuzz targets: reject fmap waypoint counts above `u16::MAX`, reject malformed spatial-index sizes before allocation, and bound Lambda `max_spatial_neighbors` before graph arithmetic.
- Re-run and triage Scorecard after remediation, documenting accepted limitations rather than optimizing solely for alert closure.

## Capabilities

### New Capabilities
- `container-build-integrity`: Requires immutable, maintainable base-image references across local and release container builds.
- `repository-change-governance`: Defines enforceable branch controls and an explicit, reviewable single-maintainer risk exception.
- `input-boundary-resilience`: Defines deterministic bounds, error behavior, and fuzz coverage for untrusted binary, archive, token, and request inputs.
- `security-posture-evidence`: Defines evidence-backed OpenSSF posture publication and finding triage without conflating heuristic alerts with vulnerabilities.

### Modified Capabilities

None.

## Impact

- **Container/release configuration:** Three service Dockerfiles, `.github/workflows/docker-release.yml`, and `.github/dependabot.yml`.
- **Repository settings and governance:** The GitHub `mainline` ruleset, threat model/security documentation, and OpenSSF Best Practices registration.
- **Rust code and tests:** fmap encoding, spatial-index loading, Lambda request validation/graph construction, fuzz targets/corpora, and Nx project targets.
- **CI:** A bounded scheduled fuzz workflow/target and additional required security-check policy; normal build, test, audit, signing, provenance, and SBOM controls remain.
- **Behavior:** Previously oversized fmap routes, malformed spatial-index files, and excessive Lambda neighbor counts will fail with typed validation errors instead of truncating, underflowing, or overflowing. Normal supported inputs and public response schemas remain unchanged.
