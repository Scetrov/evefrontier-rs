## ADDED Requirements

### Requirement: Patched rustls dependency resolution

Every committed Cargo lockfile SHALL resolve no `rustls` release affected by RUSTSEC-2026-0285 and
SHALL resolve `rustls` 0.23.45 or newer when the package is present.

#### Scenario: Lockfiles use patched rustls releases

- **WHEN** all committed Cargo lockfiles are scanned
- **THEN** every resolved `rustls` package is version 0.23.45 or newer

### Requirement: Security scan validation

The remediation SHALL pass dependency security validation without suppressing or ignoring
RUSTSEC-2026-0285.

#### Scenario: Scanner checks all committed lockfiles

- **WHEN** a vulnerability scanner evaluates every committed Cargo lockfile using current advisory
  data
- **THEN** it completes without reporting RUSTSEC-2026-0285

### Requirement: Controlled pull request delivery

The remediation SHALL be delivered through a signed feature-branch pull request that preserves
repository branch-protection controls and records the security impact and validation evidence.

#### Scenario: Remediation is submitted for review

- **WHEN** the dependency update is ready for integration into `main`
- **THEN** the pull request identifies RUSTSEC-2026-0285, lists the checks performed, and explains
  that reverting the lockfile update would reintroduce the vulnerability
