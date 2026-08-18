## ADDED Requirements

### Requirement: Patched h2 dependency resolution

The workspace SHALL resolve no version of `h2` affected by RUSTSEC-2026-0258. If `h2` is retained in
the committed Cargo dependency resolution, every retained `h2` package SHALL be version 0.4.16 or
newer.

#### Scenario: Lockfile excludes or patches vulnerable h2

- **WHEN** the workspace dependency graph is resolved from the committed `Cargo.lock`
- **THEN** it contains no `h2` version earlier than 0.4.16

### Requirement: RustSec audit validation

The remediation SHALL pass an uncached Rust dependency audit without suppressing or ignoring
RUSTSEC-2026-0258.

#### Scenario: Audit scans the updated lockfile

- **WHEN** `cargo audit` runs against the committed lockfile using current RustSec advisory data
- **THEN** it completes without reporting RUSTSEC-2026-0258

### Requirement: Regression validation

The dependency remediation SHALL preserve successful compilation, tests, formatting, and static
analysis for the affected workspace using repository-standard Nx targets and locked dependency
resolution.

#### Scenario: Workspace validation completes

- **WHEN** the relevant Nx build, test, lint, and clippy targets run after the lockfile remediation
- **THEN** they complete successfully without requiring application-code or public-API changes

### Requirement: Controlled pull request delivery

The remediation SHALL be delivered in a reviewable pull request with security-validation evidence.

#### Scenario: Remediation is submitted for review

- **WHEN** the dependency update is ready for integration into `main`
- **THEN** the pull request identifies RUSTSEC-2026-0258, lists the checks performed, and explains
  that reverting the lockfile change would reintroduce the audit finding
