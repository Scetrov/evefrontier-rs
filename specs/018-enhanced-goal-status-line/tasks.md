# Tasks: Enhanced Mode GOAL Status Line

**Input**: Design documents from `/specs/018-enhanced-goal-status-line/` **Prerequisites**: plan.md
(required), spec.md (required for user stories)

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Ensure design artifacts are ready for implementation.

- [ ] T001 [P] Populate `specs/018-enhanced-goal-status-line/plan.md` with branch, summary, and
      structure details from the spec to replace placeholders.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Confirm supporting styling exists for the new badge.

- [ ] T002 Verify color palette includes/uses the magenta inverted black-hole tag in
      `crates/evefrontier-cli/src/terminal.rs`; add it if missing.

---

## Phase 3: User Story 1 - GOAL status line & black hole badge (Priority: P1) 🎯 MVP

**Goal**: GOAL steps render a status line (min temp, planets, moons) and show a "Black Hole" badge
for systems 30000001-30000003.

**Independent Test**: Run `evefrontier-cli route --from "A 2560" --to "M 974" --format enhanced` and
verify the GOAL status line includes the black hole badge and matches other step alignments.

### Tests for User Story 1 (optional but recommended)

- [ ] T003 [US1] Add regression test covering GOAL status line and black hole badge in
      `crates/evefrontier-cli/tests/enhanced_goal_status.rs`.

### Implementation for User Story 1

- [ ] T004 [US1] Update `crates/evefrontier-cli/src/output.rs` to render step details for GOAL steps
      and detect black hole IDs (30000001-30000003) when formatting the badge.
- [ ] T005 [US1] Extend renderer unit tests in `crates/evefrontier-cli/src/output.rs` to assert GOAL
      status line rendering and black hole badge output.
- [ ] T006 [US1] Add an enhanced-format example with GOAL status line to `docs/USAGE.md` (or the
      most relevant doc) to illustrate expected output.

**Checkpoint**: User Story 1 independently verifiable via the CLI command above and passing
regression tests.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Finalize documentation and release notes.

- [ ] T007 Update `CHANGELOG.md` (Unreleased) noting the GOAL status line fix and black hole badge
      behavior.
- [ ] T008 Record regression run (`cargo test -p evefrontier-cli`) and the manual CLI check in
      `specs/018-enhanced-goal-status-line/quickstart.md` (create if missing).
- [ ] T009 Ensure gate steps report zero fuel consumption in `crates/evefrontier-lib/src/path.rs`
      (fix fuel display showing consumption on gates).

---

## Dependencies & Execution Order

- Setup (Phase 1) → Foundational (Phase 2) → User Story 1 (Phase 3) → Polish (Phase 4).
- User Story 1 depends on palette readiness (T002); tests T003 should be written before
  implementation T004/T005.

## Parallel Opportunities

- T001 and T002 can proceed in parallel.
- Within User Story 1, documentation update T006 can run in parallel after T004 is defined.

## Implementation Strategy

- MVP is Phase 3 (User Story 1). Complete T001-T002, add failing test T003, implement T004, shore up
  tests T005, then update docs T006.
- Finish with Polish (T007-T008) after MVP validation.
