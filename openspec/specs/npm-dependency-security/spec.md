## ADDED Requirements

### Requirement: Patched brace-expansion resolution
The workspace SHALL resolve `brace-expansion` to a version not affected by CVE-2026-13149 (GHSA-3jxr-9vmj-r5cp). All resolved versions in the dependency graph MUST be at or above the patched thresholds: `>= 1.1.16` for 1.x, `>= 2.1.2` for 2.x, or `>= 5.0.7` for 5.x.

#### Scenario: Lockfile uses patched brace-expansion versions
- **WHEN** the workspace dependency graph is resolved from the committed lockfile after applying the pnpm override
- **THEN** every resolved `brace-expansion` package version satisfies the patched threshold for its major version

### Requirement: pnpm override declaration
The root `package.json` SHALL declare a pnpm override entry for `brace-expansion` in the `pnpm.overrides` block that forces resolution to a patched version range.

#### Scenario: Override is present in manifest
- **WHEN** the root `package.json` is inspected for pnpm security overrides
- **THEN** the `pnpm.overrides` block contains a `brace-expansion` entry whose value is a semver range that excludes all versions affected by CVE-2026-13149

### Requirement: Regression validation
The dependency remediation SHALL preserve successful execution of the repository-standard Nx build, test, and lint targets for the workspace.

#### Scenario: Workspace checks pass after override
- **WHEN** the relevant Nx build, test, and lint targets run after the pnpm override is applied and the lockfile is regenerated
- **THEN** all checks complete successfully without requiring application code or configuration changes

### Requirement: Dependabot alert closure
The remediation MUST provide sufficient evidence in the pull request to close Dependabot alert #52, including identification of the vulnerability, the override applied, and the verified patched versions in the updated lockfile.

#### Scenario: PR documents vulnerability remediation
- **WHEN** the pull request containing this change is submitted for review
- **THEN** the PR description identifies CVE-2026-13149 / GHSA-3jxr-9vmj-r5cp, documents the pnpm override applied, and includes verification output confirming patched brace-expansion resolution

### Requirement: Controlled pull request delivery
The remediation MUST be delivered through a feature-branch pull request that preserves repository signing and branch-protection controls.

#### Scenario: Remediation is submitted for review
- **WHEN** the dependency update is ready for integration into `main`
- **THEN** the pull request is submitted from a feature branch, and reverting the override would reintroduce the vulnerability

### Requirement: Patched smol-toml resolution
The workspace SHALL resolve every `smol-toml` instance to version 1.7.1 or newer, excluding versions affected by GHSA-7w5x-hrqm-74c2 / CVE-2026-85730.

#### Scenario: Lockfile resolves a patched version
- **WHEN** pnpm resolves the committed workspace manifest and lockfile
- **THEN** every resolved `smol-toml` package version is at least 1.7.1

### Requirement: Explicit smol-toml remediation
The root manifest SHALL declare a `pnpm.overrides` entry for `smol-toml` that pins a patched version until all vulnerable transitive paths independently resolve safely.

#### Scenario: Override is inspected
- **WHEN** the root `package.json` is inspected for pnpm security overrides
- **THEN** the `pnpm.overrides` block contains a `smol-toml` entry that excludes all vulnerable versions

### Requirement: smol-toml security-audit validation
The remediation SHALL pass the repository Node high-severity security audit without alert suppression.

#### Scenario: Audit runs against committed dependency graph
- **WHEN** `pnpm audit --audit-level high` runs after a frozen-lockfile installation
- **THEN** it completes successfully and reports no high-severity `smol-toml` vulnerability

### Requirement: smol-toml GitHub alert remediation evidence
The pull request SHALL document the vulnerability, its patched resolution, the validation outcome, and the post-merge closure checks for Dependabot #56 and Scorecard #52.

#### Scenario: Pull request is submitted
- **WHEN** the remediation pull request is created
- **THEN** its description identifies GHSA-7w5x-hrqm-74c2 / CVE-2026-85730 and includes the security-audit and lockfile verification evidence
