# Tasks: Sensible Routing Defaults

**Input**: Design documents from `/specs/023-routing-defaults/`
**Prerequisites**: plan.md, spec.md

## Phase 1: Setup (Shared Infrastructure)

- [ ] T001 Initialize feature documentation and verify baseline tests pass in `crates/evefrontier-lib`
- [ ] T002 Verify `Reflex` ship exists in `data/ship_data.csv`

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core library defaults that affect all user stories and components.

- [ ] T003 [P] Update `RouteOptimization` enum in `crates/evefrontier-lib/src/routing.rs` to default to `Fuel`
- [ ] T004 [P] Update `DEFAULT_MAX_SPATIAL_NEIGHBORS` constant in `crates/evefrontier-lib/src/graph.rs` to `250`
- [ ] T005 [P] Update `RouteConstraints::default()` in `crates/evefrontier-lib/src/routing.rs` to set `avoid_critical_state: true`
- [ ] T006 [P] Add unit test in `crates/evefrontier-lib/src/routing.rs` to verify `RouteRequest` default consistency

---

## Phase 3: User Story 1 - Simplified Routing (Priority: P1) 🎯 MVP

**Goal**: Automatically apply sensible defaults (Reflex, Fuel optimization, 250 neighbors) when no flags are provided.

**Independent Test**: Running `evefrontier-cli route "Nod" "Brana"` (using the fixture starmap) should produce a route optimized for fuel using the Reflex ship by default.

### Implementation for User Story 1

- [ ] T007 [P] [US1] Update `RouteOptionsArgs` in `crates/evefrontier-cli/src/main.rs` to set `ship` default value to `"Reflex"`
- [ ] T008 [P] [US1] Update `RouteOptionsArgs` in `crates/evefrontier-cli/src/main.rs` to set `optimize` default to `fuel`
- [ ] T009 [P] [US1] Update `RouteOptionsArgs` in `crates/evefrontier-cli/src/main.rs` to set `max_spatial_neighbours` default to `250`
- [ ] T010 [P] [US1] Update `RouteRequest` in `crates/evefrontier-lambda-shared/src/requests.rs` to include system-wide defaults for `ship` and `max_spatial_neighbors`
- [ ] T011 [US1] Ensure `Reflex` default is handled correctly in `RouteCommandArgs::to_request` mapping in `crates/evefrontier-cli/src/main.rs`

---

## Phase 4: User Story 2 - Explicit Control (Priority: P2)

**Goal**: Allow users to override all defaults with explicit flags (e.g., `--ship "None"` or `--optimize distance`).

**Independent Test**: Running with `--ship "None" --optimize distance` should result in a route calculation that ignores fuel.

### Implementation for User Story 2

- [ ] T012 [US2] Verify CLI flag overrides in `crates/evefrontier-cli/src/main.rs` accurately pass values to `RouteRequest` even when they match the old defaults.

---

## Phase 5: User Story 3 - Disabling Protections (Priority: P3)

**Goal**: Provide a mechanism to disable the new default "avoid critical state" safety feature.

**Independent Test**: Running with `--no-avoid-critical-state` should allow a route through a critical system if it's the shortest path.

### Implementation for User Story 3

- [ ] T013 [P] [US3] Implement `--no-avoid-critical-state` flag in `crates/evefrontier-cli/src/main.rs`
- [ ] T014 [US3] Update `RouteCommandArgs::to_request` in `crates/evefrontier-cli/src/main.rs` to map the disable flag to `avoid_critical_state: false`

---

## Phase 6: Polish & Cross-Cutting Concerns

- [ ] T015 Update `docs/USAGE.md` with a "Default Routing Behavior" section explaining the new defaults
- [ ] T016 Verify all CLI examples in README still function with new defaults or update them if necessary
- [ ] T017 [P] Add integration test in `crates/evefrontier-cli/tests/` to verify default route output for a known system pair

## Dependencies
- Phase 2 must be complete before User Story implementation as it sets the core library expectations.

## Implementation Strategy
- **Incremental Delivery**: Complete Phase 2 and 3 first to deliver the core MVP.
- **Safety First**: Prioritize User Story 3 shortly after to ensure users aren't locked into the "safety" defaults if they need to bypass them.
