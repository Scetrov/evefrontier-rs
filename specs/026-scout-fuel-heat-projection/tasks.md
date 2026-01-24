# Tasks: Scout Fuel and Heat Projection

**Input**: Design documents from `/specs/026-scout-fuel-heat-projection/`  
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/cli-interface.md ✓, quickstart.md ✓

**Tests**: Included per plan.md Constitution Check (Test-Driven Development) and spec.md Success Criteria.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **CLI crate**: `crates/evefrontier-cli/src/`
- **Library crate**: `crates/evefrontier-lib/src/`
- **Tests**: `crates/evefrontier-cli/tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and CLI argument extensions

- [x] T001 Add ship options to `ScoutRangeArgs` struct in `crates/evefrontier-cli/src/main.rs` (`--ship`, `--fuel-quality`, `--cargo-mass`, `--fuel-load`)
- [x] T002 [P] Add clap validation for new arguments (fuel-quality 1-100, cargo-mass ≥0, fuel-load ≥0)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core algorithm and data structures that MUST be complete before user story implementation

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 Create `nearest_neighbor_order` function in `crates/evefrontier-cli/src/commands/scout.rs` implementing greedy Hamiltonian path ordering
- [x] T004 [P] Extend `RangeNeighbor` struct in `crates/evefrontier-cli/src/output_helpers.rs` with fuel/heat projection fields (`hop_fuel`, `cumulative_fuel`, `remaining_fuel`, `hop_heat`, `cumulative_heat`, `cooldown_seconds`, `fuel_warning`, `heat_warning`)
- [x] T005 [P] Extend `ScoutRangeResult` struct in `crates/evefrontier-cli/src/output_helpers.rs` with ship info and route totals (`ship`, `total_distance_ly`, `total_fuel`, `final_heat`)
- [x] T006 [P] Create `ShipInfo` struct in `crates/evefrontier-cli/src/output_helpers.rs` for response serialization (`name`, `fuel_capacity`, `fuel_quality`)

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Basic Fuel/Heat Projection (Priority: P1) 🎯 MVP

**Goal**: When `--ship` is specified, show systems in nearest-neighbor visiting order with fuel cost and heat for each hop.

**Independent Test**: `evefrontier-cli scout range "Nod" --limit 5 --ship Reflex` shows ordered route with fuel/heat per hop.

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T007 [P] [US1] Create integration test file `crates/evefrontier-cli/tests/scout_range_fuel.rs` with test module scaffold
- [x] T008 [P] [US1] Add test `test_scout_range_with_ship_returns_fuel_fields` verifying JSON output contains fuel fields
- [x] T009 [P] [US1] Add test `test_scout_range_nearest_neighbor_ordering` verifying systems are ordered by nearest-neighbor heuristic
- [x] T010 [P] [US1] Add test `test_scout_range_fuel_cumulative_tracking` verifying cumulative fuel and remaining fuel are correct

### Implementation for User Story 1

- [x] T011 [US1] Integrate `ShipCatalog` lookup in `handle_scout_range` when `--ship` provided in `crates/evefrontier-cli/src/commands/scout.rs`
- [x] T012 [US1] Create `ShipLoadout` from CLI args (fuel_load, cargo_mass, fuel_quality) in `crates/evefrontier-cli/src/commands/scout.rs`
- [x] T013 [US1] Apply `nearest_neighbor_order` to systems when ship is specified in `crates/evefrontier-cli/src/commands/scout.rs`
- [x] T014 [US1] Calculate per-hop fuel cost using `calculate_jump_fuel_cost` for each hop in `crates/evefrontier-cli/src/commands/scout.rs`
- [x] T015 [US1] Track cumulative fuel consumed and remaining fuel across route in `crates/evefrontier-cli/src/commands/scout.rs`
- [x] T016 [US1] Update `RangeNeighbor` distance_ly to show hop distance (not origin distance) when ship specified in `crates/evefrontier-cli/src/commands/scout.rs`
- [x] T017 [US1] Populate `ScoutRangeResult` ship info and route totals in `crates/evefrontier-cli/src/commands/scout.rs`

**Checkpoint**: At this point, User Story 1 should be fully functional - `scout range --ship` returns fuel projections in visit order

---

## Phase 4: User Story 2 - Heat Tracking (Priority: P2)

**Goal**: Add per-hop heat generation and cumulative heat tracking to scout range output.

**Independent Test**: `evefrontier-cli scout range "Nod" --limit 5 --ship Reflex` shows heat per hop and cumulative heat.

### Tests for User Story 2

- [x] T018 [P] [US2] Add test `test_scout_range_heat_per_hop` verifying heat is calculated for each hop using `calculate_jump_heat`
- [x] T019 [P] [US2] Add test `test_scout_range_heat_cumulative` verifying cumulative heat accumulates correctly across route

### Implementation for User Story 2

- [x] T020 [US2] Calculate per-hop heat using `calculate_jump_heat` for each hop in `crates/evefrontier-cli/src/commands/scout.rs`
- [x] T021 [US2] Track cumulative heat across the route in `crates/evefrontier-cli/src/commands/scout.rs`
- [x] T022 [US2] Populate `final_heat` in `ScoutRangeResult` in `crates/evefrontier-cli/src/commands/scout.rs`

**Checkpoint**: At this point, User Stories 1 AND 2 should both work - fuel and heat are tracked

---

## Phase 5: User Story 3 - Warnings and Cooldowns (Priority: P3)

**Goal**: Display REFUEL warning when fuel insufficient, OVERHEATED/CRITICAL warnings with cooldown times.

**Independent Test**: Route with insufficient fuel shows `⚠ REFUEL`; route with high heat shows `⚠ OVERHEATED` or `🔥 CRITICAL (wait Xs)`.

### Tests for User Story 3

- [x] T023 [P] [US3] Add test `test_scout_range_fuel_warning` verifying REFUEL warning when fuel insufficient for hop
- [x] T024 [P] [US3] Add test `test_scout_range_overheated_warning` verifying OVERHEATED warning when heat ≥ 90
- [x] T025 [P] [US3] Add test `test_scout_range_critical_cooldown` verifying CRITICAL warning with cooldown time when heat ≥ 150

### Implementation for User Story 3

- [x] T026 [US3] Add REFUEL warning detection when remaining fuel < hop cost in `crates/evefrontier-cli/src/commands/scout.rs`
- [x] T027 [US3] Add OVERHEATED warning detection when cumulative heat ≥ 90 in `crates/evefrontier-cli/src/commands/scout.rs`
- [x] T028 [US3] Add CRITICAL warning with cooldown calculation when heat ≥ 150 in `crates/evefrontier-cli/src/commands/scout.rs`
- [x] T029 [US3] Populate warning fields (`fuel_warning`, `heat_warning`, `cooldown_seconds`) in `RangeNeighbor` in `crates/evefrontier-cli/src/commands/scout.rs`

**Checkpoint**: All warning scenarios now display correctly

---

## Phase 6: User Story 4 - Output Formatting (Priority: P4)

**Goal**: Format fuel/heat projections in all output formats (enhanced, basic, JSON, text, emoji, note).

**Independent Test**: All `--format` options render fuel/heat information correctly.

### Tests for User Story 4

- [x] T030 [P] [US4] Add test `test_scout_range_enhanced_format_with_ship` verifying enhanced format shows fuel/heat columns
- [x] T031 [P] [US4] Add test `test_scout_range_json_format_with_ship` verifying JSON includes all fuel/heat fields per contracts/cli-interface.md

### Implementation for User Story 4

- [x] T032 [US4] Update `format_scout_range_enhanced` to include fuel/heat columns when ship present in `crates/evefrontier-cli/src/output_helpers.rs`
- [x] T033 [US4] Update `format_scout_range_basic` to include fuel/heat summary when ship present in `crates/evefrontier-cli/src/output_helpers.rs`
- [x] T034 [US4] Update `format_scout_range_text` to include fuel/heat when ship present in `crates/evefrontier-cli/src/output_helpers.rs`
- [x] T035 [US4] Update `format_scout_range_emoji` to include fuel/heat when ship present in `crates/evefrontier-cli/src/output_helpers.rs`
- [x] T036 [US4] Update `format_scout_range_note` to include fuel/heat summary when ship present in `crates/evefrontier-cli/src/output_helpers.rs`
- [x] T037 [US4] Add summary footer (total distance, fuel consumed, remaining, final heat) to enhanced format in `crates/evefrontier-cli/src/output_helpers.rs`
- [x] T038 [US4] Render warning icons (⚠, 🔥) in text/enhanced/emoji formats in `crates/evefrontier-cli/src/output_helpers.rs`

**Checkpoint**: All output formats now display fuel/heat projections correctly

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, validation, and cleanup

- [x] T039 [P] Update `docs/USAGE.md` with scout range fuel/heat projection examples
- [x] T040 [P] Update `CHANGELOG.md` with feature entry for scout fuel/heat projection
- [x] T041 [P] Add validation for unknown ship name with fuzzy suggestions in `crates/evefrontier-cli/src/commands/scout.rs`
- [x] T042 [P] Add validation error when fuel_load exceeds ship capacity in `crates/evefrontier-cli/src/commands/scout.rs`
- [x] T043 Run `cargo clippy --workspace` and fix any warnings
- [x] T044 Run `cargo fmt --all` to ensure consistent formatting
- [x] T045 Run `cargo test --workspace` to verify all tests pass
- [x] T046 Run quickstart.md validation: execute all example commands from quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User Story 1 (P1): Can proceed immediately after Foundational
  - User Story 2 (P2): Depends on User Story 1 (uses same route calculation loop)
  - User Story 3 (P3): Depends on User Stories 1 & 2 (warnings based on fuel/heat values)
  - User Story 4 (P4): Depends on User Stories 1, 2 & 3 (formats all fields including warnings)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

```
Setup (T001-T002)
    │
    ▼
Foundational (T003-T006)
    │
    ▼
User Story 1: Fuel Projection (T007-T017) ──► MVP Complete
    │
    ▼
User Story 2: Heat Tracking (T018-T022)
    │
    ▼
User Story 3: Warnings (T023-T029)
    │
    ▼
User Story 4: Output Formatting (T030-T038)
    │
    ▼
Polish (T039-T046)
```

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Data structures before business logic
- Core implementation before output formatting
- Story complete before moving to next priority

### Parallel Opportunities

Within each phase, tasks marked [P] can run in parallel:

- **Phase 1**: T001 and T002 are sequential (T002 depends on T001)
- **Phase 2**: T004, T005, T006 can run in parallel (different structs)
- **Phase 3**: T007-T010 (tests) can run in parallel; T011-T017 are sequential
- **Phase 4**: T018-T019 (tests) can run in parallel; T020-T022 are sequential
- **Phase 5**: T023-T025 (tests) can run in parallel; T026-T029 are sequential
- **Phase 6**: T030-T031 (tests) can run in parallel; T032-T038 work on same file
- **Phase 7**: T039-T042 can run in parallel (different files)

---

## Parallel Example: Phase 2 Foundational

```bash
# Launch all data structure tasks together:
Task T004: "Extend RangeNeighbor struct in crates/evefrontier-cli/src/output_helpers.rs"
Task T005: "Extend ScoutRangeResult struct in crates/evefrontier-cli/src/output_helpers.rs"
Task T006: "Create ShipInfo struct in crates/evefrontier-cli/src/output_helpers.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (add CLI args)
2. Complete Phase 2: Foundational (data structures + algorithm)
3. Complete Phase 3: User Story 1 (fuel projection)
4. **STOP and VALIDATE**: Test `scout range --ship Reflex` independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → MVP! (basic fuel projection)
3. Add User Story 2 → Test independently → Heat tracking added
4. Add User Story 3 → Test independently → Warnings working
5. Add User Story 4 → Test independently → All formats complete
6. Each story adds value without breaking previous stories

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- All fuel/heat calculations MUST reuse existing library functions from `evefrontier-lib/src/ship.rs`
- Nearest-neighbor algorithm is O(n²) which is acceptable for n ≤ 100 systems (NFR-1)
