# ADR 0008: Software currency — keep dependencies on latest stable versions

## Status

Accepted (policy)

## Context

Outdated dependencies increase the likelihood of running vulnerable code. This repository uses
multiple ecosystems (Rust and Node) and therefore needs a clear policy to minimize exposure.

## Decision

Adopt a policy to keep direct dependencies up-to-date on the latest stable versions, subject to the
following practical constraints:

- Update policies apply to direct dependencies only; transitive dependencies should be handled via
  tooling when vulnerabilities are discovered.
- Prefer non-breaking updates (minor/patch) as routine maintenance. Major upgrades that require
  changes should be scheduled and tested in a dedicated branch/pull request.

Automation

- Configure an automated dependency update service (for example Dependabot, Renovate) for both Cargo
  (Rust) and the Node toolchain packages. Configure automatic PR creation for minor/patch updates
  and optionally group updates for related packages.
- Run dependency scanning in CI (`cargo audit` for Rust; suitable SCA for Node) and fail pipelines
  on high-severity vulnerabilities.

Release/Review process

- Dependabot/automation PRs should include test runs and be reviewed by at least one maintainer
  before merging.
- Major-version upgrades require a brief ADR or a follow-up note in the PR describing the migration
  plan and risk assessment.

Rationale

- Keeping dependencies current reduces the window of exposure for known vulnerabilities.
- Automation reduces manual effort and surfaces problems early.

Consequences

- Increased maintenance overhead for dependency updates, but improved security posture.
- CI will need to run dependency scans and tests for each bump PR; ensure CI capacity and caching
  are configured to reduce cost/time impact.

Notes and suggested configuration

- Enable Dependabot or Renovate on the repository with separate config for Rust and Node manifests.
  Configure semantic versioning rules to automatically merge safe patch/minor updates if tests pass
  and no vulnerability flags are raised.
- Add `cargo-audit` to CI and consider a nightly scan for transitive vulnerabilities.
