# Tasks: Lambda-focused Tests

**Input**: Design from `docs/TODO.md` - "Add Lambda-focused tests (unit tests and, if possible, integration tests using `lambda_runtime::run` mocks)"

**Prerequisites**: 
- Existing Lambda crates: `evefrontier-lambda-route`, `evefrontier-lambda-scout-gates`, `evefrontier-lambda-scout-range`
- Shared infrastructure: `evefrontier-lambda-shared`
- Fixture database: `docs/fixtures/minimal_static_data.db` (8 systems: Nod, Brana, D:2NAS, G:3OA0, H:2L2S, J:35IA, Y:3R7E, E1J-M5G)

**Tests**: This feature IS about tests - all tasks implement test coverage.

**Organization**: Tasks are organized to build shared infrastructure first, then test each Lambda independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[US#]**: Which user story this task belongs to

## Path Conventions

- Lambda shared: `crates/evefrontier-lambda-shared/`
- Route Lambda: `crates/evefrontier-lambda-route/`
- Scout-Gates Lambda: `crates/evefrontier-lambda-scout-gates/`
- Scout-Range Lambda: `crates/evefrontier-lambda-scout-range/`
- Fixtures: `docs/fixtures/`

---

## Phase 1: Setup (Shared Test Infrastructure)

**Purpose**: Create reusable test utilities for all Lambda crates

- [ ] T001 Add `tokio` and `lambda_runtime` to dev-dependencies in `crates/evefrontier-lambda-shared/Cargo.toml`
- [ ] T002 Create test utilities module at `crates/evefrontier-lambda-shared/src/test_utils.rs`
- [ ] T003 Implement `fixture_db_bytes()` function using `include_bytes!` for `docs/fixtures/minimal_static_data.db`
- [ ] T004 Implement `mock_lambda_context()` helper returning `lambda_runtime::Context`
- [ ] T005 Export test utilities from `crates/evefrontier-lambda-shared/src/lib.rs` under `#[cfg(test)]`

---

## Phase 2: Foundational (Shared Crate Test Coverage)

**Purpose**: Ensure shared infrastructure has complete test coverage before testing individual Lambdas

**⚠️ CRITICAL**: Lambda handler tests depend on shared utilities being correct

- [ ] T006 [P] Add tests for `from_lib_error()` mapping all `LibError` variants in `crates/evefrontier-lambda-shared/src/problem.rs`
- [ ] T007 [P] Add tests for `LambdaResponse` JSON serialization in `crates/evefrontier-lambda-shared/src/response.rs`
- [ ] T008 [P] Add tests for `RouteAlgorithm` conversion (`From<RouteAlgorithm> for evefrontier_lib::RouteAlgorithm`) in `crates/evefrontier-lambda-shared/src/requests.rs`
- [ ] T009 Add integration test loading fixture via test utilities in `crates/evefrontier-lambda-shared/tests/runtime_integration.rs`

**Checkpoint**: Shared test infrastructure validated and ready for Lambda handler tests

---

## Phase 3: User Story 1 - Route Lambda Tests (Priority: P1) 🎯 MVP

**Goal**: Complete test coverage for route planning Lambda handler

**Independent Test**: Run `cargo test -p evefrontier-lambda-route` - all tests pass

### Unit Tests for Route Lambda

- [ ] T010 [US1] Add test module to `crates/evefrontier-lambda-route/src/main.rs`
- [ ] T011 [P] [US1] Test valid route request (Nod → Brana) returns success response
- [ ] T012 [P] [US1] Test route request with all algorithms (BFS, Dijkstra, A*)
- [ ] T013 [P] [US1] Test route request with constraints (max_jump, avoid_gates, max_temperature)
- [ ] T014 [P] [US1] Test invalid request (empty `from` field) returns 400 error
- [ ] T015 [P] [US1] Test unknown system returns 404 with suggestions
- [ ] T016 [P] [US1] Test unreachable route (avoided goal) returns 404

### Integration Tests for Route Lambda

- [ ] T017 [US1] Create integration test file `crates/evefrontier-lambda-route/tests/integration.rs`
- [ ] T018 [US1] Test full handler flow with mock `LambdaEvent` for valid request
- [ ] T019 [US1] Test JSON response structure matches documented contract

**Checkpoint**: Route Lambda has full test coverage

---

## Phase 4: User Story 2 - Scout-Gates Lambda Tests (Priority: P2)

**Goal**: Complete test coverage for gate neighbor discovery Lambda handler

**Independent Test**: Run `cargo test -p evefrontier-lambda-scout-gates` - all tests pass

### Unit Tests for Scout-Gates Lambda

- [ ] T020 [US2] Add test module to `crates/evefrontier-lambda-scout-gates/src/main.rs`
- [ ] T021 [P] [US2] Test valid gates request (Nod) returns neighbors
- [ ] T022 [P] [US2] Test system with multiple gate connections
- [ ] T023 [P] [US2] Test invalid request (empty `system` field) returns 400 error
- [ ] T024 [P] [US2] Test unknown system returns 404 with suggestions

### Integration Tests for Scout-Gates Lambda

- [ ] T025 [US2] Create integration test file `crates/evefrontier-lambda-scout-gates/tests/integration.rs`
- [ ] T026 [US2] Test full handler flow with mock `LambdaEvent`
- [ ] T027 [US2] Test JSON response structure matches documented contract

**Checkpoint**: Scout-Gates Lambda has full test coverage

---

## Phase 5: User Story 3 - Scout-Range Lambda Tests (Priority: P3)

**Goal**: Complete test coverage for spatial range query Lambda handler

**Independent Test**: Run `cargo test -p evefrontier-lambda-scout-range` - all tests pass

### Unit Tests for Scout-Range Lambda

- [ ] T028 [US3] Add test module to `crates/evefrontier-lambda-scout-range/src/main.rs`
- [ ] T029 [P] [US3] Test valid range request with default limit
- [ ] T030 [P] [US3] Test range request with custom limit
- [ ] T031 [P] [US3] Test range request with radius filter
- [ ] T032 [P] [US3] Test range request with temperature filter
- [ ] T033 [P] [US3] Test invalid request (limit=0) returns 400 error
- [ ] T034 [P] [US3] Test invalid request (limit>100) returns 400 error
- [ ] T035 [P] [US3] Test unknown system returns 404 with suggestions

### Integration Tests for Scout-Range Lambda

- [ ] T036 [US3] Create integration test file `crates/evefrontier-lambda-scout-range/tests/integration.rs`
- [ ] T037 [US3] Test full handler flow with mock `LambdaEvent`
- [ ] T038 [US3] Test JSON response structure matches documented contract

**Checkpoint**: Scout-Range Lambda has full test coverage

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: CI validation, documentation, and cleanup

- [ ] T039 Verify all Lambda crates have test targets in their `project.json` files
- [ ] T040 Run `pnpm nx run-many -t test --projects=evefrontier-lambda-*` to validate CI integration
- [ ] T041 Update `docs/TODO.md` to mark Lambda tests task complete
- [ ] T042 Add CHANGELOG.md entry under Unreleased: `[testing] Add Lambda-focused unit and integration tests`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - tests shared utilities
- **User Stories (Phases 3-5)**: All depend on Foundational phase completion
  - User stories can proceed in parallel after Phase 2
  - Or sequentially in priority order (Route → Scout-Gates → Scout-Range)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **Route Lambda (US1)**: Can start after Phase 2 - No dependencies on other Lambdas
- **Scout-Gates Lambda (US2)**: Can start after Phase 2 - No dependencies on other Lambdas
- **Scout-Range Lambda (US3)**: Can start after Phase 2 - No dependencies on other Lambdas

### Within Each User Story

- Unit tests in `main.rs` before integration tests in `tests/`
- Test setup before test implementation
- All tests marked [P] within a phase can run in parallel

### Parallel Opportunities

- T006, T007, T008 can run in parallel (different files)
- All unit tests within each Lambda (T011-T016, T021-T024, T029-T035) can run in parallel
- Once Phase 2 completes, all three Lambda test suites can be developed in parallel

---

## Parallel Example: Phase 3 (Route Lambda)

```bash
# All unit tests can be implemented in parallel:
T011: Test valid route request
T012: Test all algorithms
T013: Test constraints
T014: Test invalid request
T015: Test unknown system
T016: Test unreachable route

# Then integration tests sequentially:
T017 → T018 → T019
```

---

## Implementation Strategy

### MVP First (Route Lambda Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: Route Lambda Tests
4. **STOP and VALIDATE**: `cargo test -p evefrontier-lambda-route` passes
5. Can merge Route Lambda tests before other Lambdas

### Incremental Delivery

1. Setup + Foundational → Test utilities ready
2. Add Route Lambda tests → Validate → Commit (MVP!)
3. Add Scout-Gates tests → Validate → Commit
4. Add Scout-Range tests → Validate → Commit
5. Polish → Update docs → Final commit

---

## Notes

- [P] tasks = different test functions, can be written independently
- [US#] label maps task to specific Lambda handler
- Each Lambda should be independently testable
- Use `docs/fixtures/minimal_static_data.db` for all fixture data
- Systems available: Nod, Brana, D:2NAS, G:3OA0, H:2L2S, J:35IA, Y:3R7E, E1J-M5G
- Spatial index needed for scout-range tests - will need to build from fixture
- Commit after each phase for incremental progress
