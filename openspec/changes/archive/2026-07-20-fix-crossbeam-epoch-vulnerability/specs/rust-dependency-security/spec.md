## ADDED Requirements

### Requirement: Patched Rust dependency resolution
The workspace SHALL resolve `crossbeam-epoch` to version 0.9.20 or newer and SHALL NOT resolve a version affected by RUSTSEC-2026-0204.

#### Scenario: Lockfile uses a patched release
- **WHEN** the workspace dependency graph is resolved from the committed lockfile
- **THEN** every resolved `crossbeam-epoch` package is version 0.9.20 or newer

### Requirement: Security audit validation
The remediation SHALL pass the repository's uncached Rust dependency security audit without suppressing or ignoring RUSTSEC-2026-0204.

#### Scenario: Audit checks the updated dependency graph
- **WHEN** the repository-standard Nx security audit target runs against the updated lockfile
- **THEN** the audit completes without reporting RUSTSEC-2026-0204

### Requirement: Regression validation
The dependency remediation SHALL preserve successful compilation, tests, formatting, and static analysis for the affected workspace using repository-standard Nx targets and locked dependency resolution.

#### Scenario: Workspace checks pass
- **WHEN** the relevant Nx build, test, lint, and clippy targets run after the lockfile update
- **THEN** all checks complete successfully without requiring application code or public API changes

### Requirement: Controlled pull request delivery
The remediation MUST be delivered through a feature-branch pull request that preserves repository signing and branch-protection controls and records the security impact, validation evidence, and rollback implications.

#### Scenario: Remediation is submitted for review
- **WHEN** the dependency update is ready for integration into `main`
- **THEN** the pull request identifies RUSTSEC-2026-0204, lists the checks performed, and explains that reverting the update would reintroduce the vulnerability
