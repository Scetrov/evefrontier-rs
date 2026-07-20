## 1. Establish the Hardening Baseline

- [x] 1.1 Record the current Scorecard run, alert #42-#51 details, `mainline` ruleset JSON, and current multi-architecture digests for the Rust and distroless base tags.
- [x] 1.2 Verify the selected digests are manifest-list digests containing both `linux/amd64` and `linux/arm64` before changing build inputs.

## 2. Make Container Builds Immutable and Maintainable

- [x] 2.1 Pin the Rust builder and distroless runtime references by readable tag plus SHA-256 digest in all three service Dockerfiles.
- [x] 2.2 Replace the release workflow's inline Dockerfile with a checked-in release Dockerfile and pin its distroless base by the verified manifest-list digest.
- [x] 2.3 Configure Docker ecosystem dependency updates for all service and release Dockerfile directories, grouping duplicate image updates where supported.
- [x] 2.4 Validate with a test update or documented evidence that the updater preserves and refreshes digest pins; document a manual manifest-digest refresh procedure for any unsupported reference.
- [x] 2.5 Build all service images for amd64 and arm64 and verify non-root execution, Trivy scanning, provenance, signing configuration, and SBOM generation remain enabled.

## 3. Correct Proven Input-Boundary Defects

- [x] 3.1 Add a typed fmap encoding error for waypoint collections larger than `u16::MAX` and reject them before count conversion.
- [x] 3.2 Add fmap boundary and round-trip tests covering the maximum representable count and oversized rejection without constructing invalid tokens.
- [x] 3.3 Replace spatial-index compressed-size subtraction with checked length validation that returns the existing typed load error before allocation.
- [x] 3.4 Add malformed spatial-index regression tests for recognized headers with missing payload, metadata, or checksum sections and assert no panic/oversized allocation path.
- [x] 3.5 Define a shared safe spatial-neighbor maximum, validate Lambda `max_spatial_neighbors` against it, and make graph self-accounting arithmetic overflow-safe for direct callers.
- [x] 3.6 Add Lambda and library regression tests for valid default/zero/boundary values, values above the safe maximum, and `usize::MAX`.

## 4. Add Focused Rust Fuzzing

- [x] 4.1 Create a standalone cargo-fuzz package/project with pinned tooling/dependencies and exclude or isolate it correctly from the production Cargo workspace.
- [x] 4.2 Add an fmap target with valid seed tokens, arbitrary malformed-token coverage, and supported encode/decode round-trip invariants.
- [x] 4.3 Add a spatial-index byte-loader target with valid fixture seeds and malformed header, metadata, payload, checksum, and compressed-data mutations.
- [x] 4.4 Add a local dataset ZIP extraction target with valid fixture seeds and assertions that malformed archives return errors and never write outside the temporary destination.
- [x] 4.5 Add non-cacheable, serial Nx fuzz targets with explicit duration and resource budgets in accordance with ADR 0017.
- [x] 4.6 Add a scheduled and manually dispatched fuzz workflow with explicit least-privilege permissions and full-commit-SHA-pinned actions that runs the Nx targets and uploads minimized crash artifacts without blocking ordinary pull requests.
- [x] 4.7 Document local fuzz commands, corpus maintenance, crash minimization, and the requirement to convert fixed crashes into deterministic regression tests.

## 5. Tighten and Document Repository Governance

- [x] 5.1 Update the threat model/security governance documentation with the single-maintainer independent-review exception, residual account-compromise risk, external-contribution review behavior, and activation trigger for a second maintainer.
- [x] 5.2 Update the active `mainline` ruleset to require up-to-date branches and the stable `Security audit` context while preserving PR-only, no-bypass, deletion, force-push, thread-resolution, CodeQL, code-quality, and existing required-check controls.
- [x] 5.3 Disable the ineffective code-owner review requirement while no CODEOWNERS file exists, and verify the updated ruleset does not claim independent approval.
- [x] 5.4 Exercise the updated ruleset on a pull request and capture evidence that stale branches or failed required checks cannot merge without deadlocking successful maintainer changes.

## 6. Publish and Reconcile Security Posture Evidence

- [ ] 6.1 Register the repository with the OpenSSF Best Practices program and complete an evidence-backed initial questionnaire without overstating unmet criteria.
- [x] 6.2 Publish the awarded in-progress or passing badge/status in the README or security documentation and link it to the public project record.
- [x] 6.3 Document that Scorecard findings are posture signals requiring verification, not confirmed vulnerabilities, and record the evidence/classification for alerts #42-#51.
- [x] 6.4 Rerun Scorecard after remediation and record which alerts closed, which remain accepted single-maintainer risks, and whether Rust cargo-fuzz remains undetected.
- [x] 6.5 Dismiss or retain remaining code-scanning alerts only with specific comments covering residual risk, detector limitations, and the condition that triggers re-evaluation.

## 7. Validate and Deliver

- [x] 7.1 Run the relevant uncached Nx audit, build, test, lint, and clippy targets with locked dependency resolution and resolve all regressions.
- [x] 7.2 Run each bounded Nx fuzz target against its committed seed corpus and confirm crash artifacts are empty or converted to fixed deterministic tests.
- [x] 7.3 Review the final source, workflow, Docker, documentation, and repository-setting changes against all four capability specs and confirm no unrelated security control was weakened.
- [x] 7.4 Open a pull request that links alerts #42-#51, records validation and external-settings evidence, identifies accepted residual risks, and documents rollback implications for digest pins and ruleset changes.
