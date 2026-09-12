## ADDED Requirements

### Requirement: Patched smol-toml resolution
The workspace SHALL resolve every `smol-toml` instance to version 1.7.1 or newer, excluding versions affected by GHSA-7w5x-hrqm-74c2 / CVE-2026-85730.

#### Scenario: Lockfile resolves a patched version
- **WHEN** pnpm resolves the committed workspace manifest and lockfile
- **THEN** every resolved `smol-toml` package version is at least 1.7.1

### Requirement: Explicit transitive remediation
The root manifest SHALL declare a `pnpm.overrides` entry for `smol-toml` that pins a patched version until all vulnerable transitive paths independently resolve safely.

#### Scenario: Override is inspected
- **WHEN** the root `package.json` is inspected for pnpm security overrides
- **THEN** the `pnpm.overrides` block contains a `smol-toml` entry that excludes all vulnerable versions

### Requirement: Node security-audit validation
The remediation SHALL pass the repository Node high-severity security audit without alert suppression.

#### Scenario: Audit runs against committed dependency graph
- **WHEN** `pnpm audit --audit-level high` runs after a frozen-lockfile installation
- **THEN** it completes successfully and reports no high-severity `smol-toml` vulnerability

### Requirement: GitHub alert remediation evidence
The pull request SHALL document the vulnerability, its patched resolution, the validation outcome, and the post-merge closure checks for Dependabot #56 and Scorecard #52.

#### Scenario: Pull request is submitted
- **WHEN** the remediation pull request is created
- **THEN** its description identifies GHSA-7w5x-hrqm-74c2 / CVE-2026-85730 and includes the security-audit and lockfile verification evidence