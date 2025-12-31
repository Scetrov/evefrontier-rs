# Tasks — Ship Data & Fuel Calculations (ADR 0015)

## Phase 1: Setup
- [X] T001 Confirm ADR 0015 ratification or recorded approval gate before implementation in `docs/adrs/0015-fuel-cost-heat-impact-calculation.md`

## Phase 2: Foundational
- [X] T002 Add ship data fixture and checksum marker under `docs/fixtures/ship_data.csv` and update `docs/fixtures/README.md`
- [X] T003 Implement `ShipAttributes`, `ShipLoadout`, and `ShipCatalog` with strict validation in `crates/evefrontier-lib/src/ship.rs`
- [X] T004 Add unit tests for ship catalog parsing/validation using fixture in `crates/evefrontier-lib/tests/ship_catalog.rs`
- [X] T005 Implement fuel cost calculators (static/dynamic) and `FuelProjection` types in `crates/evefrontier-lib/src/ship.rs`
- [X] T006 Add unit tests for fuel calculators (static/dynamic) in `crates/evefrontier-lib/tests/fuel_calc.rs`
- [X] T007 Extend route data structures to carry optional fuel projection in `crates/evefrontier-lib/src/path.rs` with schema detection preserved
- [X] T008 Add route-level aggregation tests for fuel projection fields in `crates/evefrontier-lib/tests/route_fuel_projection.rs`

## Phase 3: User Story 1 (P1) — CLI fuel projection
- [X] T009 [US1] Add CLI flags and validation for ship, fuel quality, cargo mass, fuel load, dynamic mode in `crates/evefrontier-cli/src/main.rs`
- [X] T010 [US1] Render fuel projection in CLI outputs while keeping legacy output unchanged when no ship provided in `crates/evefrontier-cli/src/main.rs`
- [X] T011 [US1] Add CLI integration tests for route fuel projection with fixture dataset in `crates/evefrontier-cli/tests/route_fuel_cli.rs`

## Phase 4: User Story 2 (P1) — Lambda fuel projection
- [X] T012 [US2] Extend Lambda request/response schemas with ship/loadout and fuel projection fields in `crates/evefrontier-lambda-route/src/models.rs`
- [X] T013 [US2] Compute fuel projection in Lambda handler with backward compatibility for callers without ship data in `crates/evefrontier-lambda-route/src/main.rs`
- [X] T014 [US2] Add Lambda handler tests covering fuel projection JSON shape and legacy mode in `crates/evefrontier-lambda-route/tests/fuel_projection.rs`

## Phase 5: User Story 3 (P2) — List ships via CLI
- [X] T015 [P] [US3] Add `--list-ships` command/flag to display catalog table in `crates/evefrontier-cli/src/main.rs`
- [X] T016 [P] [US3] Add CLI integration test for ship listing using fixture in `crates/evefrontier-cli/tests/list_ships.rs`

## Phase 6: User Story 4 (P2) — Dynamic mass recalculation
- [ ] T017 [US4] Implement dynamic mass mode that updates mass per hop in fuel calculator in `crates/evefrontier-lib/src/ship.rs`
- [ ] T018 [US4] Add tests comparing static vs dynamic fuel totals for the same route in `crates/evefrontier-lib/tests/fuel_dynamic.rs`

## Phase 7: User Story 5 (P2) — Downloader caches ship data
- [ ] T019 [US5] Extend downloader to fetch/cache `ship_data.csv` alongside DB with checksum/atomic write in `crates/evefrontier-lib/src/github.rs`
- [ ] T020 [US5] Add downloader tests ensuring cache reuse and checksum validation for `ship_data.csv` in `crates/evefrontier-lib/tests/dataset_download.rs`

## Phase 8: Polish & Cross-Cutting
- [ ] T021 Update documentation with CLI/Lambda fuel examples and flags in `docs/USAGE.md` and `README.md`
- [ ] T022 Add fixtures note and guard instructions for ship data in `docs/fixtures/README.md`
- [ ] T023 Add CHANGELOG entry under Unreleased for fuel projection feature in `CHANGELOG.md`
- [ ] T024 Add security note for CSV input validation and HTTPS-only download in `docs/SECURITY_AUDIT.md`

## Dependencies
- Foundational (T002–T008) must complete before US1–US5 phases.
- US1 (T009–T011) and US2 (T012–T014) can run after foundational; US2 depends on shared models from T007.
- US3 (T015–T016) depends on catalog from T003–T004.
- US4 (T017–T018) depends on fuel calculators from T005–T006.
- US5 (T019–T020) depends on catalog/download patterns from T002–T004.
- Polish (T021–T024) after all user stories.

## Parallel Execution Examples
- Run T015 and T016 in parallel with T017/T018 after T003–T006 complete.
- Run T019 and T020 in parallel with T021 once foundational tasks are done.

## Implementation Strategy
- MVP: Complete Foundational + US1 to deliver CLI fuel projection with tests.
- Incremental: Add US2 Lambda parity, then US3 list-ships, US4 dynamic mass, US5 downloader caching, finishing with polish/docs/security.
