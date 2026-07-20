## ADDED Requirements

### Requirement: Protected default-branch changes
The default branch SHALL reject deletion and non-fast-forward updates, SHALL have no bypass actors, and SHALL require changes through pull requests with resolved review threads and linear squash history.

#### Scenario: Direct or destructive update is attempted
- **WHEN** an actor attempts to push directly, force-push, delete, or bypass policy on the default branch
- **THEN** the active repository ruleset rejects the operation

### Requirement: Fresh required security checks
Pull requests to the default branch MUST be up to date with the target branch and MUST pass the stable workspace build/test, dependency review, and security audit contexts before merge.

#### Scenario: Pull request is behind or a required check fails
- **WHEN** a pull request is behind the default branch or any required security/build context is unsuccessful
- **THEN** the repository ruleset prevents merge

### Requirement: Honest single-maintainer review exception
While the repository has fewer than two independent trusted maintainers, governance documentation MUST state that owner-authored changes cannot receive independent human approval, identify the residual account-compromise and change-quality risk, require maintainer review of external contributions before merge, and SHALL NOT represent AI, bot, self-owned CODEOWNERS, or alternate self-controlled accounts as independent review.

#### Scenario: Owner-authored change is merged
- **WHEN** the sole maintainer submits a repository change
- **THEN** required automated controls run and the documented exception accurately describes the absence of independent human approval

#### Scenario: External contribution policy is documented
- **WHEN** governance documentation describes how contributions are merged
- **THEN** it requires maintainer review of external pull requests in addition to required automated checks

### Requirement: Independent-review activation trigger
The governance exception SHALL be re-evaluated when a second independent trusted maintainer joins or before a materially higher-impact deployment is adopted; once independent review is sustainable, the ruleset MUST require at least one approval, approval of the latest reviewable push, and review from maintained CODEOWNERS coverage.

#### Scenario: Independent maintainer becomes available
- **WHEN** a second trusted human can regularly review changes
- **THEN** the maintainer updates CODEOWNERS and activates required independent approval controls before retiring the exception
