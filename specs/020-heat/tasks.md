---
description: "Task list for Heat Mechanics feature implementation"
---

# Tasks: Heat Mechanics Integration

**Input**: Design documents from `specs/020-heat/` (plan.md, research.md, data-model.md, contracts/,
quickstart.md) **Prerequisites**: `plan.md`, `research.md`, `data-model.md` (all present)

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create test scaffolding and ensure the design artifacts are staged for implementation

- [ ] T001 [P] Create test scaffold `crates/evefrontier-lib/tests/heat_calculations.rs` (add module,
      imports, and TODO test stubs)
- [ ] T002 [P] Add fixture ship CSV for tests `docs/fixtures/ship_data.csv` (confirm presence or add
      minimal fixture)
- [ ] T003 [P] Add quick sanity unit test harness file `crates/evefrontier-lib/tests/common.rs`
      (helper helpers for fixtures)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Implement the core heat calculation function and data types used by all stories

- [ ] T004 Implement unit tests for heat formula in
      `crates/evefrontier-lib/tests/heat_calculations.rs` (tests for known values from
      `specs/020-heat/research.md`)
 - [ ] T005 Implement `calculate_jump_heat(total_mass_kg, distance_ly, hull_mass_kg)` in
       `crates/evefrontier-lib/src/ship.rs` (TDD: make tests pass). The function uses the fixed
       internal calibration constant (`1e-7`) and does not accept calibration as a parameter.
 - [ ] T006 Add `HeatConfig` type (no calibration field; calibration fixed server-side) to
       `crates/evefrontier-lib/src/ship.rs` and validate inputs
- [ ] T007 Add unit tests validating `ShipAttributes` parsing handles absence of per-ship heat fields and that canonical CSV schema (no per-ship heat columns) loads correctly in `crates/evefrontier-lib/tests/heat_calculations.rs`
- [ ] T008 Export new heat API (functions and types) from `crates/evefrontier-lib/src/lib.rs`

**Checkpoint**: Foundation ready — user story implementation can begin

---

## Phase 3: User Story 1 - Heat Calculation (Priority: P1) 🎯 MVP

**Goal**: Provide accurate per-hop heat calculation (static & dynamic mass modes)

**Independent Test**: Unit tests validate `calculate_jump_heat()` against research test cases and
gate zero-heat case

### Implementation

- [ ] T009 [US1] Add test cases for dynamic vs static mass modes in
      `crates/evefrontier-lib/tests/heat_calculations.rs` (verify small differences for Reflex
      examples)
- [ ] T010 [US1] Ensure `calculate_jump_heat()` is invoked with correct `total_mass` (reuse
      `ShipLoadout::total_mass_kg()` pattern) in `crates/evefrontier-lib/src/ship.rs`
- [ ] T011 [US1] Add unit tests for edge-cases (zero distance for gates, extreme masses) in
      `crates/evefrontier-lib/tests/heat_calculations.rs`

**Checkpoint**: `calculate_jump_heat()` is implemented and well-tested

---

## Phase 4: User Story 2 - Status Line Display (Priority: P1)

**Goal**: Attach heat projections to each route step and display heat in plain/rich/note outputs

**Independent Test**: Integration test that `RouteSummary::from_plan` + `attach_heat()` yields
`RouteStep.heat` and `RouteSummary.heat` with expected values and serialization

### Implementation

- [ ] T012 [US2] Add `HeatProjection` and `HeatSummary` structs to
      `crates/evefrontier-lib/src/output.rs` (serde annotations)
- [ ] T013 [US2] Implement `RouteSummary::attach_heat(ship, loadout, &HeatConfig)` in
      `crates/evefrontier-lib/src/output.rs` (mirror `attach_fuel()` behavior; gate = 0 heat)
- [ ] T014 [US2] Add integration tests `crates/evefrontier-lib/tests/route_heat_projection.rs`
      verifying per-hop heat and cumulative non-decreasing property
- [ ] T015 [US2] Update `RouteSummary::render_plain`, `render_rich`, `render_note` in
      `crates/evefrontier-lib/src/output.rs` to include heat status (format: `Heat: +XX.XX (cumulative: YYY.YY units)`)

**Checkpoint**: CLI/Lambda consumers receive heat fields in `RouteSummary` and text outputs include
heat

---

## Phase 5: User Story 3 - Heat Warnings (Priority: P2)

**Goal**: Add warning and error thresholds to per-hop and summary heat

**Independent Test**: Integration test that triggers warning at >75% and error at >100% and that
warnings appear in `RouteSummary.heat.warnings` and rendered output

- [ ] T016 [US3] Implement threshold checks in `attach_heat()` using canonical thresholds (`HEAT_OVERHEATED`, `HEAT_CRITICAL`) and attach messages to `HeatProjection.warning` and `HeatSummary.warnings` in `crates/evefrontier-lib/src/output.rs`
- [ ] T017 [US3] Add integration tests `crates/evefrontier-lib/tests/route_heat_warnings.rs` for
      warning and error scenarios

---

## Phase 6: User Story 4 - JSON Schema & Lambda Integration (Priority: P2)

**Goal**: Ensure heat fields are present in JSON output and Lambda responses; validate contract

**Independent Test**: Lambda unit/integration tests assert `heat` object present and matches schema

- [ ] T018 [US4] Add unit tests in `crates/evefrontier-lambda-route/tests/heat_response.rs`
      asserting `RouteSummary` serialized JSON includes `heat` per
      `specs/020-heat/contracts/heat_response.json`
- [ ] T019 [US4] Add integration test in `crates/evefrontier-lib/tests/route_heat_json.rs` verifying
      JSON serialization of `HeatProjection` and `HeatSummary`
- [ ] T020 [US4] Ensure Lambda handler calls `attach_heat()` when ship data present in
      `crates/evefrontier-lambda-route/src/lib.rs` (mirror existing `attach_fuel()` usage)

---

## Phase 7: User Story 5 - CLI Integration & Documentation (Priority: P3)

**Goal**: Wire heat projection into CLI and update user-facing docs and examples

**Independent Test**: CLI integration test that `evefrontier-cli route --ship Reflex` prints heat
lines; docs updated in `docs/USAGE.md`

- [ ] T021 [US5] Update CLI `route` code in `crates/evefrontier-cli/src/main.rs` to call
      `summary.attach_heat(ship, &loadout, &HeatConfig::default())` after `attach_fuel()`
- [ ] T022 [US5] Add CLI integration test `crates/evefrontier-cli/tests/route_heat_output.rs` to
      assert plain text output includes heat snippet
- [ ] T023 [US5] Update `docs/USAGE.md` with heat quickstart and examples (copy from
      `specs/020-heat/quickstart.md`)
- [ ] T024 [US5] Update `CHANGELOG.md` with a short `Unreleased` entry describing Heat mechanics
      feature

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup, tests, and docs

- [ ] T025 [P] Update ADR `docs/adrs/0015-fuel-cost-heat-impact-calculation.md` with "Heat
      Implementation Details" and status line examples
- [ ] T026 [P] Improve `docs/HEAT_MECHANICS.md` with a short table of example calculations and link
      to `docs/USAGE.md`
- [ ] T027 [P] Run `cargo fmt`, `cargo clippy --all-targets --all-features`, and
      `cargo test --workspace`; fix any issues found
- [ ] T028 [P] Add CI job or extend existing CI to include heat tests if necessary
      (`.github/workflows/ci.yml`)

---

## Dependencies & Execution Order

- Foundation tasks (T004-T008) must complete before US1/US2 tasks begin
- `attach_heat()` (T013) depends on `calculate_jump_heat()` (T005) and type definitions (T012)
- CLI/Lambda wiring (T021/T020) depends on `attach_heat()` (T013)

## Parallel Opportunities

- Tests (T004, T009, T011, T014, T017, T018, T019, T022) can be authored in parallel (marked [P]
  where safe)
- Documentation updates (T023, T025, T026) can proceed in parallel with implementation fixes

---

## Implementation Strategy

- Follow TDD: write failing tests first (see test tasks above), implement minimal code to pass
  tests, then refactor
- Deliver MVP with US1 and US2 (T005, T012, T013, T014) first, then add warnings (T016/T017) and
  integration (T020/T021)
- Keep changes small and iterative; open a PR from `020-heat` with each completed checkpoint

---

## Estimated Effort (per task)

- T004–T006: 0.5 day
- T009–T011: 0.5 day
- T012–T015: 0.75 day
- T016–T017: 0.5 day
- T018–T020: 0.5 day
- T021–T024: 0.5 day
- T025–T028: 0.5 day
- T029–T033: 0.75 day
- T034–T040: 1.0 day

**Total**: ~5–6 days (mvp + polish)
