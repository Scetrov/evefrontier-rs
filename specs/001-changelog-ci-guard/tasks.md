---
description: "Task list for CI guard requiring CHANGELOG.md modification for code changes"
---

# Tasks: CI Guard for CHANGELOG.md Modifications

**Input**: Design documents from `/specs/001-changelog-ci-guard/`
**Prerequisites**: plan.md (complete), spec.md (complete)
**Tests**: Included - validation via test PRs and CI job execution

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1=Enforce requirement, US2=Provide guidance)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Understand current CI structure and prepare for new job

- [ ] T001 Review `.github/workflows/ci.yml` structure and identify placement for changelog-guard job
- [ ] T002 Review existing GitHub Actions workflow patterns in the repository (e.g., `.github/workflows/adr-governance.yml`) for error message style
- [ ] T003 [P] Review CONTRIBUTING.md to identify existing changelog guidance (if any) vs. what needs to be added

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Document requirements and prepare implementation plan

**⚠️ CRITICAL**: These tasks must complete before workflow implementation begins

- [ ] T004 Create test cases document listing scenarios to verify (all acceptance criteria from spec.md)
  - Create `specs/001-changelog-ci-guard/test-cases.md` with all 8 acceptance scenarios
  - Document expected outcomes for each test case
- [ ] T005 Define exact exemption patterns (doc paths, config files) and validate against real repository structure
- [ ] T006 [P] Research GitHub Actions `paths` filter syntax and bash scripting for file detection
- [ ] T007 [P] Prepare error message template with exemption rules and CONTRIBUTING.md link
- [ ] T008 Document the label-bypass mechanism for emergency fixes (e.g., `skip-changelog-check`)
- [ ] T008a [P] Research GitHub Actions ability to detect Dependabot/Renovate PR authors
  - Determine if `github.actor` context can identify bot PRs
  - Evaluate whether bot PRs should be exempt or require CHANGELOG updates
  - Document decision rationale
- [ ] T008b [P] Decide and document Draft PR handling
  - Research GitHub Actions `pull_request.draft` context field
  - Decide: check only when "Ready for Review" OR check all PRs
  - Document decision rationale with trade-offs

**Checkpoint**: Foundation ready - workflow implementation can now begin

---

## Phase 3: User Story 1 - Enforce CHANGELOG.md Updates for Code Changes (Priority: P1) 🎯 MVP

**Goal**: Implement CI job that detects code changes and fails if CHANGELOG.md is missing

**Independent Test**: Create a test PR modifying src/* files without CHANGELOG.md - CI should fail with clear message

### Tests for User Story 1

- [ ] T009 [US1] Test Case: PR modifies code (src/**) without CHANGELOG.md → CI FAILS
- [ ] T010 [US1] Test Case: PR modifies code (src/**) WITH CHANGELOG.md → CI PASSES
- [ ] T011 [US1] Test Case: PR modifies docs/** only, no CHANGELOG.md → CI PASSES (exempt)
- [ ] T012 [US1] Test Case: PR modifies README.md only, no CHANGELOG.md → CI PASSES (exempt)
- [ ] T013 [US1] Test Case: PR modifies .github/workflows/** only, no CHANGELOG.md → CI PASSES (exempt)
- [ ] T014 [US1] Test Case: PR modifies crates/** (Lambda/lib) without CHANGELOG.md → CI FAILS
- [ ] T015 [US1] Test Case: PR modifies tests/** without CHANGELOG.md → CI FAILS
- [ ] T016 [US1] Test Case: PR modifies Cargo.toml without CHANGELOG.md → CI FAILS

### Implementation for User Story 1

- [ ] T017 [P] [US1] Create shell script for file pattern detection in `.github/scripts/check-changelog.sh`
  - Input: Detect if PR contains code file changes (src/, crates/, examples/, tests/, benches/, Cargo.toml, Makefile)
  - Input: Detect if PR contains CHANGELOG.md modification
  - Output: Exit code 0 if OK, 1 if failure
  - Include comprehensive comments for maintainability

- [ ] T018 [P] [US1] Prepare exemption rules in script (docs/**, *.md root files, .github/**, .gitignore, .nvmrc, etc.)
  - Map exemption patterns from spec.md Table 1
  - Test script against sample file lists

- [ ] T019 [US1] Add GitHub Actions job to `.github/workflows/ci.yml` named "changelog-guard"
  - Trigger: on pull_request to main branch
  - Use `paths` filter to run only when relevant files change
  - Call `.github/scripts/check-changelog.sh`
  - Exit with appropriate status

- [ ] T020 [P] [US1] Implement label-bypass mechanism in check-changelog.sh
  - Read GitHub Actions context for labels
  - Skip check if `skip-changelog-check` label is present
  - Log that check was skipped and why

- [ ] T021 [US1] Test the workflow locally via act or on a test branch
  - Verify exit codes
  - Verify job runs only on relevant paths
  - Test bypass label functionality

**Checkpoint**: User Story 1 complete - CI job enforces CHANGELOG.md requirement for code changes

---

## Phase 4: User Story 2 - Provide Clear Guidance on Exemptions (Priority: P2)

**Goal**: Make failure messages helpful with clear exemption rules and links

**Independent Test**: Trigger the workflow failure and verify error message explains what's required

**Note**: This user story can be implemented independently after Phase 3 completes. It enhances the MVP with excellent developer experience but is not required for baseline enforcement functionality. If time-constrained, this can be deferred to v2 with clear messaging for users.

### Tests for User Story 2

- [ ] T022 [US2] Verify failure message is shown in GitHub PR checks UI
- [ ] T023 [US2] Verify failure message includes list of exempt file patterns
- [ ] T024 [US2] Verify failure message includes link to CONTRIBUTING.md
- [ ] T025 [US2] Verify failure message includes example CHANGELOG.md entry format

### Implementation for User Story 2

- [ ] T026 [US2] Implement descriptive error message in check-changelog.sh
  - Message should state: "CHANGELOG.md must be updated for code changes"
  - List exempt file patterns (docs/**, *.md files at root, etc.)
  - Include link to CONTRIBUTING.md#changelog-maintenance
  - Show example entry format from ADR 0010

- [ ] T027 [P] [US2] Update `CONTRIBUTING.md` with "Maintaining CHANGELOG.md" section
  - Explain why changelog discipline is important (reference ADR 0010)
  - Show changelog entry format: `date - author - [category] - description`
  - List categories used in the project (from existing CHANGELOG.md)
  - Provide examples of good changelog entries
  - Note that CI will enforce this requirement
  - Document the `skip-changelog-check` label for emergency-only use

- [ ] T028 [US2] Update GitHub PR template (`.github/pull_request_template.md`) if exists
  - Add reminder to check CHANGELOG.md
  - Reference CONTRIBUTING.md guidance link

- [ ] T029 [US2] Add CI job step output to display help message on failure
  - Use GitHub Actions `run:` step with explicit message
  - Include command to read more: `cat CONTRIBUTING.md | grep -A 20 "## Maintaining CHANGELOG"`

**Checkpoint**: User Stories 1 & 2 complete - enforcement working with clear guidance

---

## Phase 5: Integration & Documentation

**Purpose**: Ensure CI job integrates properly and is documented for maintainers

- [ ] T030 [P] Verify changelog-guard job runs in correct order (after code quality checks, before merge)
- [ ] T031 [P] Update `.github/workflows/README.md` or create workflow documentation
  - Document purpose: "Ensure all code changes include CHANGELOG.md updates"
  - Document bypasses: "Label `skip-changelog-check` skips this check (emergency only)"
  - Document file patterns that trigger/exempt the check

- [ ] T032 Update `docs/TODO.md` to mark this task complete
  - Update checkbox: "✅ Add CI guard requiring CHANGELOG.md modification for non-doc code changes"

- [ ] T033 Create example CI workflow run (screenshot or documentation) showing:
  - Passing run (code + CHANGELOG modified)
  - Failing run (code only, no CHANGELOG)
  - Passing run with bypass label

---

## Phase 6: Polish & Validation

**Purpose**: Final verification and documentation

- [ ] T034 Test the complete workflow end-to-end on a test branch
  - Create PR with code changes only → Verify CI fails
  - Add CHANGELOG.md update → Verify CI passes
  - Test with documentation-only changes → Verify CI passes

- [ ] T035 Verify error messages are clear and helpful in actual GitHub PR UI
- [ ] T036 [P] Review file exemption patterns against real-world examples
  - Confirm no false positives (e.g., examples/ is code, should fail)
  - Confirm no false negatives (e.g., docs/ is exempt, should pass)

- [ ] T037 Document any edge cases discovered during testing
  - Merge conflicts in CHANGELOG.md
  - Dependabot/Renovate PRs (should they be exempt?)
  - Draft PRs (should checks run?)

- [ ] T038 Merge PR to main branch with appropriate review

---

## Dependencies & Execution Strategy

### Parallel Opportunities

- **Phase 1**: All tasks can run in parallel
- **Phase 2**: T005, T006, T007, T008, T008a, T008b can run in parallel once T004 is drafted
- **Phase 4**: T027, T028 can run in parallel after T026 is started
- **Phase 6**: T035, T036 can run in parallel

### MVP Scope

**User Story 1 only** (T009-T021): Deliverable MVP that enforces CHANGELOG.md requirement for code changes. This alone provides immediate value by preventing changelog debt.

**Full scope** (T001-T038): Complete enforcement with excellent developer experience through clear guidance and documentation.

---

**Total Tasks**: 40 (was 38, added T008a and T008b)  
**Estimated Duration**: ~4 hours (MVP), ~6.5 hours (full scope)  
**Team Size**: 1 person sufficient (32% parallelizable tasks)
