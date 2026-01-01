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
- [X] T009 [US1] Add CLI flags and validation for ship (default: Reflex), fuel quality (default: 10%), cargo mass, fuel load, dynamic mode in `crates/evefrontier-cli/src/main.rs`
- [X] T010 [US1] Render fuel projection in CLI outputs while keeping legacy output unchanged when no ship provided in `crates/evefrontier-cli/src/main.rs`
- [X] T011 [US1] Add CLI integration tests for route fuel projection with fixture dataset including default ship/fuel validation in `crates/evefrontier-cli/tests/route_fuel_cli.rs`

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
- [X] T019 [US5] Created test specs for ship data downloader in `crates/evefrontier-lib/tests/dataset_download_ship_data.rs` (GitHub downloader enhancement deferred to future PR)
- [X] T020 [US5] Added integration tests for ship data fixture validation, parsing, and future downloader idempotency in `crates/evefrontier-lib/tests/dataset_download_ship_data.rs`
- [ ] T025 [US5] Implement downloader logic to detect `ship_data.csv` in release assets, download and cache as `<tag>-ship_data.csv` with `<tag>-ship_data.csv.sha256` checksum, and expose `DatasetPaths.ship_data` from `ensure_dataset`/`ensure_e6c3_dataset`.
- [ ] T026 [US5] Implement detection of `ship_data.csv` in `ReleaseResponse.assets` and extend `select_dataset_asset`/classification helpers in `crates/evefrontier-lib/src/github.rs` to return a `ShipData` asset when present.
- [ ] T027 [US5] Add download + atomic cache write for ship CSV: save as `<tag>-ship_data.csv` under cache dir in `crates/evefrontier-lib/src/github.rs` (reuse existing download helpers and extract/archive handling where appropriate).
- [ ] T028 [US5] Compute and write SHA-256 sidecar `<tag>-ship_data.csv.sha256` after successful download and add verification on cache hit to re-download on mismatch (`crates/evefrontier-lib/src/github.rs`).
- [ ] T029 [US5] Extend `DatasetPaths` to include `ship_data: Option<PathBuf>` and populate it from `ensure_dataset`/`ensure_e6c3_dataset` (`crates/evefrontier-lib/src/dataset.rs`).
- [ ] T030 [US5] Add unit/integration tests for downloader behavior (cache, checksum verification, missing-asset behavior) in `crates/evefrontier-lib/tests/dataset_download_ship_data.rs` using `download_from_source_with_cache` helpers and `docs/fixtures/ship_data.csv` fixture.
- [ ] T031 [US5] Add ShipCatalog integration tests that load cached `ship_data.csv` via `ShipCatalog::from_path` and assert parsing, duplicates and missing headers errors `crates/evefrontier-lib/tests/ship_catalog.rs`.
- [ ] T032 [US5] Add CLI integration test verifying `evefrontier-cli ships` reads the cached ship CSV and prints expected columns (`crates/evefrontier-cli/tests/list_ships.rs`) [P].
- [ ] T033 [US5] Update Lambda build packaging to include `ship_data.csv` (or derived serialized artifact) in bundle and add a test in `crates/evefrontier-lambda-shared/tests` to assert presence (`crates/evefrontier-lambda-shared/*`) — depends on T028.
- [ ] T034 [US5] Update documentation: add cache location and usage to `docs/USAGE.md`, add fixture notes to `docs/fixtures/README.md`, and mark TODOs in `docs/TODO.md` as completed where appropriate (`docs/USAGE.md`, `docs/fixtures/README.md`, `docs/TODO.md`) [P].
- [ ] T035 [US5] Add CHANGELOG.md entry under Unreleased describing ship data caching and fuel projection defaults (`CHANGELOG.md`) [P].
- [ ] T036 [US5] Add CI job or test matrix entry to run downloader-related tests and ensure they pass in CI (`.github/workflows/ci.yml`) (depends on T029/T030/T032).

## Phase 8: Polish & Cross-Cutting
- [X] T021 Updated `docs/USAGE.md` with comprehensive fuel projection section (static/dynamic modes, examples, formula, ship listing)
- [X] T022 Added ship_data.csv fixture documentation to `docs/fixtures/README.md` with protection notes and test examples
- [X] T023 Added fuel calculation feature to CHANGELOG.md under Unreleased → Added section
- [X] T024 Added ship data security section to `docs/SECURITY_AUDIT.md` covering CSV validation, HTTPS downloads, and atomic writes

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
