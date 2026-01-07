# Tasks: Cooling Time Calculation (024-cooling)

**Input**: Design documents from `/specs/024-cooling/` **Prerequisites**: plan.md (required),
spec.md (required for user stories), research.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing
of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create feature branch `024-cooling` and initialize design documents
- [x] T002 Initialize `research.md` with Newton's Law of Cooling formulas

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure and mathematical constants

- [x] T003 Define `BASE_COOLING_POWER` (1e6) and temperature thresholds in
      `crates/evefrontier-lib/src/ship.rs`
- [x] T004 Implement `compute_cooling_constant` using the Zone Factor model in
      `crates/evefrontier-lib/src/ship.rs`
- [x] T005 Implement `calculate_cooling_time` (exponential model) in
      `crates/evefrontier-lib/src/ship.rs`

**Checkpoint**: Foundation ready - cooldown logic can now be integrated into route projections.

---

## Phase 3: User Story 1 - Cooldown Calculation (Priority: P1) 🎯 MVP

**Goal**: Accurately calculate the time needed to cool down to `HEAT_NOMINAL` (30.0K) after a jump.

**Independent Test**: `cargo test -p evefrontier-lib --lib ship::tests`

### Implementation for User Story 1

- [x] T006 [P] [US1] Add unit tests for `calculate_cooling_time` in
      `crates/evefrontier-lib/src/ship.rs`
- [x] T007 [US1] Refactor `attach_heat` to calculate `wait_time_seconds` targeting 30.0K in
      `crates/evefrontier-lib/src/output.rs`
- [x] T008 [US1] Update `HeatProjection` unit tests to verify exponential wait times in
      `crates/evefrontier-lib/src/output.rs`

**Checkpoint**: User Story 1 is complete. `HeatProjection` now contains accurate
`wait_time_seconds`.

---

## Phase 4: User Story 2 - User Interface Indicators (Priority: P2)

**Goal**: Display the cooldown time in a human-readable `XmYs` format in the CLI.

**Independent Test**:
`cargo run -p evefrontier-cli -- route "Nod" "Brana" --ship "Reflex" --enhanced`

### Implementation for User Story 2

- [x] T009 [P] [US2] Implement `format_cooldown_duration` helper in
      `crates/evefrontier-cli/src/output_helpers.rs`
- [x] T010 [P] [US2] Update `ColumnWidths` and `build_heat_segment` to render the cooldown string in
      `crates/evefrontier-cli/src/output_helpers.rs`
- [x] T011 [US2] Update integration tests in `crates/evefrontier-lib/tests/route_heat_projection.rs`
      to reflect 30.0K target

**Checkpoint**: User Story 2 is complete. CLI output now shows `(2m4s to cool)` for overheated
jumps.

---

## Phase 5: User Story 3 - Lambda Updates (Priority: P3)

**Goal**: Ensure Lambda API consumers receive the new cooldown duration in the response.

**Independent Test**: Test `evefrontier-lambda-route` handler with a ship loadout that generates
heat.

### Implementation for User Story 3

- [x] T012 [US3] Verify `HeatProjection` JSON serialization includes `wait_time_seconds` for Lambda
      responses
- [x] T013 [US3] Ensure all Lambda integration tests pass with the new thermal model

**Checkpoint**: User Story 3 is complete.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Cleanup, documentation, and final validation.

- [x] T014 [P] Update `docs/HEAT_MECHANICS.md` to document the Newton's Law of Cooling
      implementation
- [x] T015 [P] Update `CHANGELOG.md` with the new cooling time indicator
- [x] T016 Run final `cargo test --workspace` and verify 78+ tests pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 2)**: Core math - BLOCKS all story integration.
- **User Stories (Phase 3-5)**: Can be implemented sequentially.
- **Polish (Phase 6)**: Final documentation and verification.

### Parallel Opportunities

- T006, T009, T010 can run in parallel as they touch different files (lib vs cli).
- Documentation updates (T014, T015) can be done anytime after the logic is stable.

---

## Implementation Strategy

### MVP First

The MVP is the core mathematical transition in `evefrontier-lib`. Once `wait_time_seconds` is
populated correctly using Newton's Law, the CLI and Lambda automatically benefit, even if the fancy
`XmYs` string isn't yet in the CLI.

### Incremental Delivery

1. **Foundation**: Core math in `ship.rs`.
2. **Logic**: Integration in `output.rs`.
3. **UI**: Formatting and rendering in `evefrontier-cli`.
