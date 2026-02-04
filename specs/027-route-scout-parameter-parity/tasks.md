# Tasks: Route & Scout Parameter Parity

**Input**: Design documents from `/specs/027-route-scout-parameter-parity/`
**Prerequisites**: plan.md (complete), spec.md (complete), research.md (complete), data-model.md (complete), quickstart.md (complete)

**Tests**: This feature specification does NOT explicitly request TDD. Test tasks are omitted; implementation tasks will include verification steps.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and shared argument struct foundation

- [X] T001 Create `crates/evefrontier-cli/src/common_args.rs` with module declaration and Rustdoc header
- [X] T002 Add `mod common_args;` and `pub use common_args::*;` to `crates/evefrontier-cli/src/lib.rs` (or main.rs if no lib.rs exists)
- [X] T003 [P] Update `CHANGELOG.md` under Unreleased section with "[Added] Unified parameter model across route and scout commands"

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared argument structs that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 [P] Implement `CommonRouteConstraints` struct in `crates/evefrontier-cli/src/common_args.rs` with fields: max_jump, avoid, avoid_gates, max_temp
- [X] T005 [P] Implement `CommonShipConfig` struct in `crates/evefrontier-cli/src/common_args.rs` with fields: ship, fuel_quality, cargo_mass, fuel_load, dynamic_mass
- [X] T006 [P] Implement `CommonHeatConfig` struct in `crates/evefrontier-cli/src/common_args.rs` with fields: avoid_critical_state, no_avoid_critical_state, sys_temp_curve
- [X] T007 Add `#[command(flatten)]` imports and help_heading annotations to all three shared structs
- [X] T008 Add value parser functions (parse_fuel_quality, parse_non_negative) to `crates/evefrontier-cli/src/common_args.rs` if not already present
- [X] T009 Verify shared structs compile and derive Args, Debug, Clone traits correctly

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Avoidance Constraints in Scout Range (Priority: P1) 🎯 MVP

**Goal**: Enable `--avoid` parameter in `scout range` to exclude hostile systems from scouting radius

**Independent Test**: Run `evefrontier-cli scout range Nod -r 50 --avoid "Brana" --avoid "H:2L2S"` and verify Brana and H:2L2S are not in results

### Implementation for User Story 1

- [X] T010 [US1] Refactor `ScoutRangeArgs` struct in `crates/evefrontier-cli/src/commands/scout.rs` to add `#[command(flatten)] constraints: CommonRouteConstraints`
- [X] T011 [US1] Update `handle_scout_range()` function in `crates/evefrontier-cli/src/commands/scout.rs` to pass `constraints.avoid` to RouteConstraints
- [X] T012 [US1] Update `handle_scout_range()` function in `crates/evefrontier-cli/src/commands/scout.rs` to pass `constraints.max_temp` to RouteConstraints (if not already present)
- [X] T013 [US1] Update `handle_scout_range()` function in `crates/evefrontier-cli/src/commands/scout.rs` to pass `constraints.avoid_gates` to RouteConstraints
- [X] T014 [US1] Update `handle_scout_range()` function in `crates/evefrontier-cli/src/commands/scout.rs` to pass `constraints.max_jump` to RouteConstraints
- [X] T015 [US1] Add CLI integration test in `crates/evefrontier-cli/tests/scout_tests.rs` verifying `scout range --avoid` excludes specified systems
- [X] T016 [US1] Run `cargo test -p evefrontier-cli --test scout_tests` to verify avoidance constraints work correctly

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Dynamic Mass Fuel Projection in Scout Range (Priority: P2)

**Goal**: Enable `--dynamic-mass` parameter in `scout range` for accurate fuel projections during multi-hop scouting

**Independent Test**: Run `evefrontier-cli scout range Nod -r 80 --ship Reflex --dynamic-mass --format json` and verify fuel consumption decreases per hop as mass changes

### Implementation for User Story 2

- [X] T017 [US2] Refactor `ScoutRangeArgs` struct in `crates/evefrontier-cli/src/commands/scout.rs` to add `#[command(flatten)] ship: CommonShipConfig`
- [X] T018 [US2] Update `handle_scout_range()` function in `crates/evefrontier-cli/src/commands/scout.rs` to build ShipLoadout from CommonShipConfig
- [X] T019 [US2] Update `handle_scout_range()` function in `crates/evefrontier-cli/src/commands/scout.rs` to pass `ship.dynamic_mass` flag to RouteRequest
- [X] T020 [US2] Update `handle_scout_range()` function in `crates/evefrontier-cli/src/commands/scout.rs` to pass `ship.fuel_quality`, `ship.cargo_mass`, `ship.fuel_load` to RouteRequest
- [X] T021 [US2] Add CLI integration test in `crates/evefrontier-cli/tests/scout_tests.rs` verifying `scout range --dynamic-mass` produces per-hop mass recalculation
- [X] T022 [US2] Run `cargo test -p evefrontier-cli --test scout_tests` to verify dynamic mass fuel projection works correctly

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Heat-Aware Routing in Scout Range (Priority: P3)

**Goal**: Enable `--avoid-critical-state` parameter in `scout range` to prevent recommending dangerous high-heat hops

**Independent Test**: Run `evefrontier-cli scout range Nod -r 100 --avoid-critical-state --sys-temp-curve flux` and verify no systems with critical heat signatures (≥150K) appear

### Implementation for User Story 3

- [X] T023 [US3] Refactor `ScoutRangeArgs` struct in `crates/evefrontier-cli/src/commands/scout.rs` to add `#[command(flatten)] heat: CommonHeatConfig`
- [X] T024 [US3] Update `handle_scout_range()` function in `crates/evefrontier-cli/src/commands/scout.rs` to pass `heat.avoid_critical_state` to RouteConstraints
- [X] T025 [US3] Update `handle_scout_range()` function in `crates/evefrontier-cli/src/commands/scout.rs` to pass `heat.no_avoid_critical_state` to RouteConstraints
- [X] T026 [US3] Update `handle_scout_range()` function in `crates/evefrontier-cli/src/commands/scout.rs` to pass `heat.sys_temp_curve` to TemperatureCurve conversion
- [X] T027 [US3] Add CLI integration test in `crates/evefrontier-cli/tests/scout_tests.rs` verifying `scout range --avoid-critical-state` excludes high-heat systems
- [X] T028 [US3] Run `cargo test -p evefrontier-cli --test scout_tests` to verify heat-aware routing works correctly

**Checkpoint**: All user stories 1-3 should now be independently functional

---

## Phase 6: User Story 4 - Spatial-Only Scouting (Priority: P4)

**Goal**: Enable `--avoid-gates` parameter in `scout range` to find systems only reachable by spatial jumps

**Independent Test**: Run `evefrontier-cli scout range Nod -r 50 --avoid-gates` and verify no gate-connected systems appear in results

### Implementation for User Story 4

- [X] T029 [US4] Verify `ScoutRangeArgs.constraints.avoid_gates` is already wired to RouteConstraints (from Phase 3, T013)
- [X] T030 [US4] Update `handle_scout_range()` function in `crates/evefrontier-cli/src/commands/scout.rs` to filter gate-connected systems when `avoid_gates` is true
- [X] T031 [US4] Add CLI integration test in `crates/evefrontier-cli/tests/scout_tests.rs` verifying `scout range --avoid-gates` excludes gate-connected neighbors
- [X] T032 [US4] Run `cargo test -p evefrontier-cli --test scout_tests` to verify spatial-only scouting works correctly

**Checkpoint**: User Story 4 complete and independently testable

---

## Phase 7: User Story 5 - Fuel Optimization in Scout Range (Priority: P5)

**Goal**: Enable `--optimize fuel` parameter in `scout range` to minimize fuel consumption when visiting multiple systems

**Independent Test**: Run `evefrontier-cli scout range Nod -r 80 --ship Reflex --optimize fuel` and verify results are ordered by cumulative fuel cost (not distance)

### Implementation for User Story 5

- [ ] T033 [US5] Add `optimize: Option<RouteOptimizeArg>` field to `ScoutRangeArgs` struct in `crates/evefrontier-cli/src/commands/scout.rs`
- [ ] T034 [US5] Add `max_spatial_neighbours: usize` field to `ScoutRangeArgs` struct in `crates/evefrontier-cli/src/commands/scout.rs` with default value 250
- [ ] T035 [US5] Update `handle_scout_range()` function in `crates/evefrontier-cli/src/commands/scout.rs` to pass `optimize` to RouteRequest
- [ ] T036 [US5] Update `handle_scout_range()` function in `crates/evefrontier-cli/src/commands/scout.rs` to use `max_spatial_neighbours` when building spatial graph
- [ ] T037 [US5] Update result sorting logic in `handle_scout_range()` to order by fuel cost when `optimize == Some(RouteOptimizeArg::Fuel)`
- [ ] T038 [US5] Add CLI integration test in `crates/evefrontier-cli/tests/scout_tests.rs` verifying `scout range --optimize fuel` orders by fuel consumption
- [ ] T039 [US5] Run `cargo test -p evefrontier-cli --test scout_tests` to verify fuel optimization works correctly

**Checkpoint**: User Story 5 complete and independently testable

---

## Phase 8: User Story 6 - Consistent Parameter Naming (Priority: P6)

**Goal**: Ensure identical parameters use identical flag names and value formats across `route` and `scout` commands for simplified scripts and documentation

**Independent Test**: Run `evefrontier-cli route --help` and `evefrontier-cli scout range --help`, then verify shared parameter descriptions are identical

### Implementation for User Story 6

- [X] T040 [US6] Refactor `RouteCommandArgs` struct in `crates/evefrontier-cli/src/commands/route.rs` to use `#[command(flatten)] constraints: CommonRouteConstraints`
- [X] T041 [US6] Refactor `RouteCommandArgs` struct in `crates/evefrontier-cli/src/commands/route.rs` to use `#[command(flatten)] ship: CommonShipConfig`
- [X] T042 [US6] Refactor `RouteCommandArgs` struct in `crates/evefrontier-cli/src/commands/route.rs` to use `#[command(flatten)] heat: CommonHeatConfig`
- [X] T043 [US6] Remove duplicated parameter definitions from `RouteCommandArgs` that now come from flattened structs
- [X] T044 [US6] Update `handle_route()` function in `crates/evefrontier-cli/src/commands/route.rs` to reference `constraints.*`, `ship.*`, `heat.*` instead of direct fields
- [X] T045 [US6] Add `include_ccp_systems: bool` field to `RouteCommandArgs` in `crates/evefrontier-cli/src/commands/route.rs` (SKIPPED: route doesn't filter systems, only plans paths)
- [X] T046 [US6] Update `handle_route()` function in `crates/evefrontier-cli/src/commands/route.rs` to filter CCP systems when `include_ccp_systems` is false (SKIPPED: not applicable to route command)
- [X] T047 [US6] Add CLI integration test in `crates/evefrontier-cli/tests/route_tests.rs` verifying `route --include-ccp-systems` includes staging systems (SKIPPED: not applicable)
- [X] T048 [US6] Run `cargo test -p evefrontier-cli --test route_tests --test scout_tests` to verify parameter parity across both commands
- [X] T049 [US6] Run `cargo run -p evefrontier-cli -- route --help > /tmp/route_help.txt` and `cargo run -p evefrontier-cli -- scout range --help > /tmp/scout_help.txt`, then manually verify shared parameter descriptions match

**Checkpoint**: All user stories should now be independently functional with consistent parameter naming

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, cleanup, and final validation across all user stories

- [X] T050 [P] Update `docs/USAGE.md` section "Scout Commands" with examples for all new scout range parameters (--avoid, --dynamic-mass, --avoid-critical-state, --avoid-gates, --optimize)
- [X] T051 [P] Update `docs/USAGE.md` section "Route Command" with example for --include-ccp-systems flag (SKIPPED: --include-ccp-systems not applicable to route command)
- [X] T052 [P] Add Rustdoc comments to all three shared argument structs in `crates/evefrontier-cli/src/common_args.rs` explaining their purpose and usage
- [X] T053 Run `cargo fmt --all` to ensure code formatting is consistent
- [X] T054 Run `cargo clippy --all-targets --all-features -- -D warnings` to verify no linting issues
- [X] T055 Run `cargo test --workspace` to verify all tests pass (unit, integration, CLI)
- [X] T056 Run `cargo build --release` to verify release build succeeds
- [X] T057 Run quickstart.md validation: Execute all example commands from `specs/027-route-scout-parameter-parity/quickstart.md` and verify expected output
- [X] T058 Update `CHANGELOG.md` entry under Unreleased to include all implemented user stories with detailed descriptions
- [X] T059 [P] Add help text grouping annotations (`help_heading = "ROUTING CONSTRAINTS"`, etc.) to shared argument structs if not already present
- [X] T060 Review CI workflow `.github/workflows/ci.yml` to ensure no additional test configuration needed for new parameters

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-8)**: All depend on Foundational phase completion
  - User stories CAN proceed in parallel (different aspects of scout range args)
  - Or sequentially in priority order (US1 → US2 → US3 → US4 → US5 → US6)
- **Polish (Phase 9)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1 - Avoidance)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2 - Dynamic Mass)**: Can start after Foundational (Phase 2) - Independent of US1
- **User Story 3 (P3 - Heat-Aware)**: Can start after Foundational (Phase 2) - Independent of US1/US2
- **User Story 4 (P4 - Spatial-Only)**: Depends on US1 (T013) for avoid_gates wiring - Can verify independently
- **User Story 5 (P5 - Fuel Optimization)**: Can start after Foundational (Phase 2) - Independent of US1-US4
- **User Story 6 (P6 - Consistency)**: Should be done LAST - Refactors route command to use shared structs, requires all scout changes complete for accurate testing

### Within Each User Story

- Struct refactoring before function updates
- Function updates before integration tests
- Tests before checkpoint validation
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks (T004, T005, T006) marked [P] can run in parallel (different struct definitions)
- Once Foundational phase completes:
  - US1, US2, US3, US5 can all start in parallel (different parameters, independent)
  - US4 should wait for US1 T013 completion (avoid_gates wiring)
  - US6 should be done last (refactors route command)
- All documentation tasks in Phase 9 marked [P] can run in parallel

---

## Parallel Example: Foundational Phase

```bash
# Launch all shared struct implementations together:
Task: "Implement CommonRouteConstraints struct in crates/evefrontier-cli/src/common_args.rs"
Task: "Implement CommonShipConfig struct in crates/evefrontier-cli/src/common_args.rs"
Task: "Implement CommonHeatConfig struct in crates/evefrontier-cli/src/common_args.rs"
```

---

## Parallel Example: User Stories 1, 2, 3, 5 (After Foundational Complete)

```bash
# Launch user story implementations in parallel (if team capacity allows):
Team Member A: Phase 3 (User Story 1 - Avoidance Constraints)
Team Member B: Phase 4 (User Story 2 - Dynamic Mass)
Team Member C: Phase 5 (User Story 3 - Heat-Aware Routing)
Team Member D: Phase 7 (User Story 5 - Fuel Optimization)

# User Story 4 waits for Team Member A to complete T013
# User Story 6 waits for all others to complete
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Avoidance Constraints)
4. **STOP and VALIDATE**: Test `scout range --avoid` independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Add User Story 4 → Test independently → Deploy/Demo
6. Add User Story 5 → Test independently → Deploy/Demo
7. Add User Story 6 → Test parameter consistency → Deploy/Demo
8. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Avoidance)
   - Developer B: User Story 2 (Dynamic Mass)
   - Developer C: User Story 3 (Heat-Aware)
   - Developer D: User Story 5 (Fuel Optimization)
3. Developer A continues with User Story 4 (depends on US1 T013)
4. All developers review User Story 6 together (refactors route command)
5. Stories complete and integrate independently

---

## Summary

**Total Tasks**: 60 tasks across 9 phases
**Task Count by User Story**:
- Setup: 3 tasks
- Foundational: 6 tasks (BLOCKS all stories)
- User Story 1 (Avoidance): 7 tasks
- User Story 2 (Dynamic Mass): 6 tasks
- User Story 3 (Heat-Aware): 6 tasks
- User Story 4 (Spatial-Only): 4 tasks
- User Story 5 (Fuel Optimization): 7 tasks
- User Story 6 (Consistency): 10 tasks
- Polish: 11 tasks

**Parallel Opportunities Identified**:
- Foundational phase: 3 parallel struct implementations (T004, T005, T006)
- User story implementations: 4 stories can run in parallel after Foundational (US1, US2, US3, US5)
- Documentation phase: 3 parallel documentation tasks (T050, T051, T052)

**Independent Test Criteria**:
- US1: `scout range --avoid` excludes specified systems
- US2: `scout range --dynamic-mass` recalculates mass per hop
- US3: `scout range --avoid-critical-state` excludes high-heat systems
- US4: `scout range --avoid-gates` shows only spatial-reachable systems
- US5: `scout range --optimize fuel` orders by fuel consumption
- US6: `route --help` and `scout range --help` show identical shared parameter descriptions

**Suggested MVP Scope**: Complete Phases 1-3 (Setup + Foundational + User Story 1 only) for initial release

**Format Validation**: ✅ ALL tasks follow the required checklist format:
- Checkbox: `- [ ]`
- Task ID: Sequential (T001-T060)
- [P] marker: Applied to 8 parallelizable tasks
- [Story] label: Applied to all user story phase tasks (US1-US6)
- Descriptions: Include exact file paths and clear actions

---

## Notes

- [P] tasks = different files or independent struct definitions, no sequential dependencies
- [Story] label maps task to specific user story for traceability and independent testing
- Each user story should be independently completable and testable
- Commit after each task or logical group (e.g., after each user story phase)
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts within parallel tasks, cross-story dependencies that break independence
- User Story 4 has a dependency on User Story 1 (T013) but can still be validated independently
- User Story 6 should be done last as it refactors the route command to use shared structs
