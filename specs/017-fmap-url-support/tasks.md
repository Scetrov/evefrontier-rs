# Tasks: fmap URL Support

**Input**: Design documents from `/specs/017-fmap-url-support/`
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/ ✓

**Tests**: Included - TDD required per Constitution I.

**Organization**: Tasks are grouped by user story (FR-1 through FR-6) to enable independent implementation.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which requirement this task implements (FR-1 through FR-6)
- Include exact file paths in descriptions

## Path Conventions

- **Library**: `crates/evefrontier-lib/src/`
- **CLI**: `crates/evefrontier-cli/src/`
- **Tests**: `crates/evefrontier-lib/tests/`, `crates/evefrontier-cli/tests/`
- **Fixtures**: `docs/fixtures/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project dependencies and basic module structure

- [ ] T001 Add `flate2` and `base64` direct dependencies in `crates/evefrontier-lib/Cargo.toml`
- [ ] T002 Create empty `fmap.rs` module stub in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T003 Add `pub mod fmap;` export in `crates/evefrontier-lib/src/lib.rs`
- [ ] T004 [P] Create test vectors fixture file at `docs/fixtures/fmap_test_vectors.json`

---

## Phase 2: Foundational (Types & Constants)

**Purpose**: Core types that all encoding/decoding depends on

**⚠️ CRITICAL**: No encoding/decoding work can begin until this phase is complete

- [ ] T005 [P] Define `WaypointType` enum with Start(0), Jump(1), NpcGate(2), SmartGate(3), SetDestination(4) in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T006 [P] Define `Waypoint` struct with `system_id: u32` and `waypoint_type: WaypointType` in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T007 [P] Define `FmapToken` struct with token, waypoint_count, bit_width, version in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T008 [P] Define constants `BASE_SYSTEM_ID`, `FMAP_VERSION`, `FMAP_HEADER_SIZE`, `MAX_BIT_WIDTH`, `WAYPOINT_TYPE_BITS` in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T009 Add fmap error variants to `crates/evefrontier-lib/src/error.rs`: FmapBase64DecodeError, FmapDecompressionError, FmapCompressionError, FmapUnsupportedVersion, FmapInvalidBitWidth, FmapTruncatedData, FmapInvalidSystemId

**Checkpoint**: Foundation ready - encoding/decoding implementation can begin

---

## Phase 3: FR-1 - Encode Waypoints to fmap Token (Priority: P1) 🎯 MVP

**Goal**: Encode route waypoints to fmap URL format (bitpacked + gzip + base64url)

**Independent Test**: `cargo test -p evefrontier-lib fmap::encode`

### Tests for FR-1 (TDD - Write First)

- [ ] T010 [P] [FR-1] Write unit test `test_calculate_bit_width` for bit width calculation in `crates/evefrontier-lib/tests/fmap.rs`
- [ ] T011 [P] [FR-1] Write unit test `test_encode_single_waypoint` in `crates/evefrontier-lib/tests/fmap.rs`
- [ ] T012 [P] [FR-1] Write unit test `test_encode_multiple_waypoints` in `crates/evefrontier-lib/tests/fmap.rs`
- [ ] T013 [P] [FR-1] Write unit test `test_encode_invalid_system_id_below_base` returns error in `crates/evefrontier-lib/tests/fmap.rs`

### Implementation for FR-1

- [ ] T014 [FR-1] Implement `BitWriter` helper struct with `write_bits(value, bit_count)` and `finish()` in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T015 [FR-1] Implement `calculate_bit_width(max_offset: u32) -> u8` helper in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T016 [FR-1] Implement `encode_raw_bitpacked(waypoints) -> Vec<u8>` (header + payload) in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T017 [FR-1] Implement `to_base64url(bytes) -> String` helper in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T018 [FR-1] Implement public `encode_fmap_token(waypoints: &[Waypoint]) -> Result<FmapToken, Error>` in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T019 [FR-1] Run tests and verify all FR-1 tests pass

**Checkpoint**: FR-1 encoding works independently - can encode any valid waypoint list

---

## Phase 4: FR-2 - Decode fmap Token to Waypoints (Priority: P1)

**Goal**: Decode fmap URL tokens back to waypoint lists (for validation/debugging)

**Independent Test**: `cargo test -p evefrontier-lib fmap::decode`

### Tests for FR-2 (TDD - Write First)

- [ ] T020 [P] [FR-2] Write unit test `test_decode_single_waypoint` in `crates/evefrontier-lib/tests/fmap.rs`
- [ ] T021 [P] [FR-2] Write unit test `test_decode_multiple_waypoints` in `crates/evefrontier-lib/tests/fmap.rs`
- [ ] T022 [P] [FR-2] Write unit test `test_decode_invalid_base64` returns FmapBase64DecodeError in `crates/evefrontier-lib/tests/fmap.rs`
- [ ] T023 [P] [FR-2] Write unit test `test_decode_invalid_version` returns FmapUnsupportedVersion in `crates/evefrontier-lib/tests/fmap.rs`
- [ ] T024 [P] [FR-2] Write unit test `test_decode_truncated_data` returns FmapTruncatedData in `crates/evefrontier-lib/tests/fmap.rs`

### Implementation for FR-2

- [ ] T025 [FR-2] Implement `BitReader` helper struct with `read_bits(bit_count) -> u32` in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T026 [FR-2] Implement `from_base64url(s: &str) -> Result<Vec<u8>, Error>` helper in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T027 [FR-2] Implement `decode_raw_bitpacked(bytes: &[u8]) -> Result<Vec<Waypoint>, Error>` in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T028 [FR-2] Implement public `decode_fmap_token(token: &str) -> Result<Vec<Waypoint>, Error>` in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T029 [FR-2] Run tests and verify all FR-2 tests pass

**Checkpoint**: FR-2 decoding works independently - can decode any valid token

---

## Phase 5: FR-3 - Round-trip & Cross-Implementation Compatibility (Priority: P1)

**Goal**: Verify encoding/decoding is lossless and compatible with JS reference

**Independent Test**: `cargo test -p evefrontier-lib fmap::roundtrip`

### Tests for FR-3 (TDD - Write First)

- [ ] T030 [P] [FR-3] Write test `test_roundtrip_single_waypoint` (encode → decode → compare) in `crates/evefrontier-lib/tests/fmap.rs`
- [ ] T031 [P] [FR-3] Write test `test_roundtrip_multiple_waypoints` in `crates/evefrontier-lib/tests/fmap.rs`
- [ ] T032 [P] [FR-3] Write test `test_roundtrip_edge_case_max_offset` (large system IDs) in `crates/evefrontier-lib/tests/fmap.rs`
- [ ] T033 [P] [FR-3] Write test `test_decode_js_reference_token` using vectors from `docs/fixtures/fmap_test_vectors.json` in `crates/evefrontier-lib/tests/fmap.rs`
- [ ] T034 [FR-3] Generate JS reference tokens using Node.js `bitpacking.js` and add to `docs/fixtures/fmap_test_vectors.json`
- [ ] T035 [FR-3] Run all round-trip and cross-implementation tests

**Checkpoint**: FR-1, FR-2, FR-3 complete - library encoding/decoding is production-ready

---

## Phase 6: FR-4 - CLI `--format fmap` (Priority: P2)

**Goal**: Add CLI output format option `--format fmap` to generate shareable URLs

**Independent Test**: `cargo run -p evefrontier-cli -- route Nod Brana --format fmap`

### Tests for FR-4 (TDD - Write First)

- [ ] T036 [P] [FR-4] Write integration test `test_cli_route_format_fmap_outputs_url` in `crates/evefrontier-cli/tests/fmap_integration.rs`
- [ ] T037 [P] [FR-4] Write integration test `test_cli_route_format_fmap_custom_base_url` in `crates/evefrontier-cli/tests/fmap_integration.rs`

### Implementation for FR-4

- [ ] T038 [FR-4] Implement `route_to_waypoints(route: &Route) -> Vec<Waypoint>` conversion in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T039 [FR-4] Implement `route_to_fmap_url(route: &Route, base_url: Option<&str>) -> Result<String, Error>` in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T040 [FR-4] Add `Fmap` variant to `OutputFormat` enum in `crates/evefrontier-cli/src/main.rs` (or dedicated output module)
- [ ] T041 [FR-4] Add `--fmap-base-url` CLI argument with default and env var support in `crates/evefrontier-cli/src/main.rs`
- [ ] T042 [FR-4] Integrate fmap output into route command output handling
- [ ] T043 [FR-4] Run CLI integration tests and verify fmap output works

**Checkpoint**: FR-4 complete - users can generate shareable URLs from routes

---

## Phase 7: FR-5 - CLI `fmap decode` Subcommand (Priority: P2)

**Goal**: Add CLI command `fmap decode <token>` to decode and display route tokens

**Independent Test**: `cargo run -p evefrontier-cli -- fmap decode <token>`

### Tests for FR-5 (TDD - Write First)

- [ ] T044 [P] [FR-5] Write integration test `test_cli_fmap_decode_text_output` in `crates/evefrontier-cli/tests/fmap_integration.rs`
- [ ] T045 [P] [FR-5] Write integration test `test_cli_fmap_decode_json_output` in `crates/evefrontier-cli/tests/fmap_integration.rs`
- [ ] T046 [P] [FR-5] Write integration test `test_cli_fmap_decode_invalid_token_error` in `crates/evefrontier-cli/tests/fmap_integration.rs`

### Implementation for FR-5

- [ ] T047 [FR-5] Create `fmap` subcommand with `decode` action in `crates/evefrontier-cli/src/main.rs`
- [ ] T048 [FR-5] Implement text output formatter for decoded waypoints (table format)
- [ ] T049 [FR-5] Implement JSON output formatter for decoded waypoints
- [ ] T050 [FR-5] Add system name lookup when dataset is available
- [ ] T051 [FR-5] Add helpful error messages for decode failures
- [ ] T052 [FR-5] Run CLI integration tests for fmap decode

**Checkpoint**: FR-5 complete - users can inspect shared tokens

---

## Phase 8: FR-6 - Waypoint Type Mapping (Priority: P1)

**Goal**: Map waypoint types correctly from Route steps

**Independent Test**: `cargo test -p evefrontier-lib fmap::waypoint_type_mapping`

### Tests for FR-6 (TDD - Write First)

- [ ] T053 [P] [FR-6] Write test `test_route_to_waypoints_first_is_start` in `crates/evefrontier-lib/tests/fmap.rs`
- [ ] T054 [P] [FR-6] Write test `test_route_to_waypoints_gate_jumps_are_npc_gate` in `crates/evefrontier-lib/tests/fmap.rs`
- [ ] T055 [P] [FR-6] Write test `test_route_to_waypoints_spatial_jumps_are_jump` in `crates/evefrontier-lib/tests/fmap.rs`

### Implementation for FR-6

- [ ] T056 [FR-6] Ensure `route_to_waypoints` correctly maps step types (T038 covers this, verify logic)
- [ ] T057 [FR-6] Run waypoint type mapping tests

**Checkpoint**: All functional requirements complete

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, cleanup, and final validation

- [ ] T058 [P] Add Rustdoc comments to all public types and functions in `crates/evefrontier-lib/src/fmap.rs`
- [ ] T059 [P] Update `docs/USAGE.md` with fmap URL examples
- [ ] T060 [P] Update `CHANGELOG.md` with fmap feature entry under Unreleased
- [ ] T061 Run `cargo clippy --workspace --all-targets -D warnings` and fix any issues
- [ ] T062 Run `cargo fmt --all` and verify formatting
- [ ] T063 Run full test suite: `cargo test --workspace`
- [ ] T064 Validate quickstart.md examples work correctly
- [ ] T065 Update `docs/TODO.md` to mark fmap URL support task as complete

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup) ──────────────────────────────────────┐
                                                       │
Phase 2 (Foundational) ◄──────────────────────────────┘
    │
    ├──► Phase 3 (FR-1: Encode) ──────────────────────┐
    │                                                  │
    ├──► Phase 4 (FR-2: Decode) ──────────────────────┤
    │                                                  │
    └──► Phase 8 (FR-6: Type Mapping) ─────────────────┤
                                                       │
         Phase 5 (FR-3: Round-trip) ◄─────────────────┤
              │                                        │
              ▼                                        │
         Phase 6 (FR-4: CLI format) ◄─────────────────┤
              │                                        │
              ▼                                        │
         Phase 7 (FR-5: CLI decode) ◄─────────────────┘
              │
              ▼
         Phase 9 (Polish) ◄───────────────────────────
```

### Parallel Opportunities

**Within Setup (Phase 1)**:
- T001-T003 sequential (dependency order)
- T004 can run in parallel with T001-T003

**Within Foundational (Phase 2)**:
- T005, T006, T007, T008 can all run in parallel
- T009 can run in parallel with above

**After Foundational**:
- Phase 3 (FR-1), Phase 4 (FR-2), Phase 8 (FR-6) can start in parallel
- Phase 5 (FR-3) requires Phase 3 AND Phase 4
- Phase 6 (FR-4) requires Phase 3 AND Phase 8
- Phase 7 (FR-5) requires Phase 4

**Within Each Phase**:
- All tests marked [P] can run in parallel (write before implementation)
- Implementation tasks generally sequential within phase

---

## MVP Scope

**Minimum Viable Product (Phases 1-5)**:
- Setup infrastructure
- Types and constants
- Encode waypoints (FR-1)
- Decode tokens (FR-2)
- Round-trip validation (FR-3)

**Full Feature (Phases 1-9)**:
- All of MVP plus:
- CLI `--format fmap` (FR-4)
- CLI `fmap decode` (FR-5)
- Waypoint type mapping (FR-6)
- Documentation and polish

---

## Task Summary

| Phase | Tasks | Parallel | Est. Effort |
|-------|-------|----------|-------------|
| 1: Setup | T001-T004 | 1 | 0.5h |
| 2: Foundational | T005-T009 | 5 | 1h |
| 3: FR-1 Encode | T010-T019 | 4 | 2h |
| 4: FR-2 Decode | T020-T029 | 5 | 2h |
| 5: FR-3 Round-trip | T030-T035 | 4 | 1h |
| 6: FR-4 CLI format | T036-T043 | 2 | 1.5h |
| 7: FR-5 CLI decode | T044-T052 | 3 | 1.5h |
| 8: FR-6 Type mapping | T053-T057 | 3 | 0.5h |
| 9: Polish | T058-T065 | 3 | 1h |
| **Total** | **65** | **30** | **~11h** |
