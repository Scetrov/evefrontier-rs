# Feature Specification: CI Guard for CHANGELOG.md Modifications

**Feature Branch**: `001-changelog-ci-guard`  
**Created**: 2025-12-07  
**Status**: Draft  
**Input**: From docs/TODO.md - "Add CI guard requiring `CHANGELOG.md` modification for non-doc code
changes ([ADR 0010](adrs/0010-maintain-changelog.md))"

## User Scenarios & Testing

### User Story 1 - Enforce CHANGELOG.md Updates for Code Changes (Priority: P1)

As a **project maintainer**, I want the CI system to **require CHANGELOG.md modifications for
non-documentation code changes**, so that **the project changelog stays synchronized with actual
code updates**.

**Why this priority**: This is the core feature that ensures the changelog discipline documented in
ADR 0010 is enforced automatically. Without it, developers can merge code changes without updating
the changelog, breaking the discipline intended by the ADR.

**Independent Test**: The feature is fully testable by simulating a PR with code changes but without
CHANGELOG.md modification - the CI should fail with a clear message.

**Acceptance Scenarios**:

1. **Given** a PR that modifies `src/**` files, **When** the PR does not modify `CHANGELOG.md`,
   **Then** the CI workflow fails with message "CHANGELOG.md must be updated for code changes"
2. **Given** a PR that modifies `docs/**` only, **When** CHANGELOG.md is not modified, **Then** the
   CI workflow passes (documentation changes are exempt)
3. **Given** a PR that modifies code AND includes CHANGELOG.md update, **When** changes are
   committed, **Then** the CI workflow passes
4. **Given** a PR that modifies test files only (`tests/**`, `benches/**`), **When** CHANGELOG.md is
   not modified, **Then** the CI workflow fails (tests are code)
5. **Given** a PR that only modifies `README.md`, `.github/workflows/**`, or configuration files,
   **When** CHANGELOG.md is not modified, **Then** the CI workflow passes (meta/config changes are
   exempt)

---

### User Story 2 - Provide Clear Guidance on Exemptions (Priority: P2)

As a **developer**, I want the CI failure message to **explain which file types are exempt and how
to properly update CHANGELOG.md**, so that **I can understand what's required and fix the issue
quickly**.

**Why this priority**: Good developer experience - clear error messages reduce friction and support
the policy intent.

**Independent Test**: CI job logs should contain documented exemption rules and links to
CONTRIBUTING.md guidelines.

**Acceptance Scenarios**:

1. **Given** a CI failure due to missing CHANGELOG.md, **When** I read the failure message, **Then**
   it lists the exempt file patterns
2. **Given** a CI failure, **When** I review the message, **Then** it includes a link to
   CONTRIBUTING.md section on changelog maintenance
3. **Given** a CI failure, **When** I review the message, **Then** it shows an example of a proper
   CHANGELOG.md entry

---

## Requirements

### Functional Requirements

- **FR-001**: CI workflow MUST detect when code files are modified in a PR (src/, crates/, Lambda
  crates)
- **FR-002**: CI workflow MUST check if CHANGELOG.md is included in the PR changeset
- **FR-003**: CI workflow MUST FAIL the job if code changed but CHANGELOG.md was not modified
- **FR-004**: CI workflow MUST PASS the job if only documentation/config files changed (docs/,
  .github/, \*.md files at root)
- **FR-005**: CI workflow MUST PASS the job if code files AND CHANGELOG.md were both modified
- **FR-006**: CI workflow MUST provide a clear, actionable error message on failure
- **FR-007**: CI workflow MUST document which file patterns trigger the requirement and which are
  exempt
- **FR-008**: The check MUST run on all PRs targeting the main branch
- **FR-009**: The check MUST allow maintainers to skip it via a GitHub label (e.g.,
  `skip-changelog-check`) for emergency fixes

### Key Entities

N/A - This is purely a CI/workflow feature with no data model or entities

### File Patterns & Exemptions

| Pattern                      | Type          | Requires CHANGELOG | Reason                               |
| ---------------------------- | ------------- | ------------------ | ------------------------------------ |
| `src/**`                     | Source code   | YES                | Core library code                    |
| `crates/**`                  | Rust crates   | YES                | Application/Lambda code              |
| `examples/**`                | Examples      | YES                | User-facing examples                 |
| `benches/**`                 | Benchmarks    | YES                | Performance code (code-level change) |
| `tests/**`                   | Tests         | YES                | Test code is code                    |
| `Cargo.toml`                 | Dependencies  | YES                | Dependency updates are code changes  |
| `Makefile`                   | Build script  | YES                | Build system changes                 |
| `docs/**`                    | Documentation | NO                 | Pure documentation                   |
| `specs/**`                   | Specifications | NO                 | Planning/design documents            |
| `.github/workflows/**`       | CI config     | NO                 | Meta/infrastructure                  |
| `*.md` (at root)             | Root docs     | NO                 | Meta/documentation                   |
| `.gitignore`, `.nvmrc`, etc. | Config files  | NO                 | Infrastructure configuration         |

---

## Edge Cases

- **Empty CHANGELOG.md update**: What if CHANGELOG.md is modified but only whitespace/formatting
  changes? Should be **REJECTED** - must add a meaningful entry
- **Multiple commits in PR**: The check should look at the entire PR diff, not individual commits
- **Merge conflicts in CHANGELOG.md**: If CHANGELOG.md has merge conflicts, the check should fail
  until resolved
- **Dependabot/Renovate PRs**: Should these be exempt? **Proposal**: Exempt PRs authored by
  `dependabot[bot]` or `renovate[bot]`
- **Draft PRs**: Should they be checked? **Proposal**: Only check when PR is marked "Ready for
  Review"

---

## Implementation Constraints & Decisions

1. **Technology**: GitHub Actions workflow (YAML-based)
2. **CI Job Name**: `changelog-guard` (referenced in `.github/workflows/ci.yml`)
3. **Trigger**: On `pull_request` with `paths` filter for code files
4. **Language**: Bash/shell scripting for path detection
5. **Integration**: Add new job to `.github/workflows/ci.yml` (add to existing workflow, not separate file)
6. **Error Message**: Should match project's existing error message style (see
   `.github/workflows/adr-governance.yml` for reference)
7. **Documentation**: Update CONTRIBUTING.md with changelog maintenance section (if not already
   present)
8. **Bot PRs**: Research whether to exempt Dependabot/Renovate PRs (see Edge Cases section)
9. **Draft PRs**: Decision needed: check only when "Ready for Review" or check all PRs (see Edge Cases section)
