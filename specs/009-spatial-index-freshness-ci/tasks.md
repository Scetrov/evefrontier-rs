# Tasks: Spatial Index Freshness CI Verification

**Input**: Design documents from `/specs/009-spatial-index-freshness-ci/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Tests are included following TDD per Constitution Principle I.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Library crate**: `crates/evefrontier-lib/src/`
- **CLI crate**: `crates/evefrontier-cli/src/`
- **Library tests**: `crates/evefrontier-lib/tests/`
- **CLI tests**: `crates/evefrontier-cli/tests/`
- **CI workflows**: `.github/workflows/`
- **Documentation**: `docs/`

---

## Phase 1: Setup

**Purpose**: Prepare test infrastructure and fixtures for TDD workflow

- [X] T001 Create spatial index v2 test file in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`
- [X] T002 [P] Create test fixture helper to generate temp spatial index files in `crates/evefrontier-lib/tests/common/mod.rs` (create if not exists)
- [X] T003 [P] Add test dependency `hex` to `crates/evefrontier-lib/Cargo.toml` for checksum assertions

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data types that ALL user stories depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Tests for Foundational Types (TDD - Write tests FIRST)

- [X] T004 [P] Write test `test_dataset_metadata_serialization` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`
- [X] T005 [P] Write test `test_dataset_metadata_default` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`
- [X] T006 [P] Write test `test_freshness_result_variants` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`

### Implementation for Foundational Types

- [X] T007 Add `DatasetMetadata` struct to `crates/evefrontier-lib/src/spatial.rs` with fields: checksum, release_tag, build_timestamp
- [X] T008 Add `FreshnessResult` enum to `crates/evefrontier-lib/src/spatial.rs` with variants: Fresh, Stale, LegacyFormat, Missing, DatasetMissing, Error
- [X] T009 Add `VerifyOutput` and `VerifyDiagnostics` structs to `crates/evefrontier-lib/src/spatial.rs`
- [X] T010 Add `metadata: Option<DatasetMetadata>` field to `SpatialIndex` struct in `crates/evefrontier-lib/src/spatial.rs`
- [X] T011 Export new types in `crates/evefrontier-lib/src/lib.rs`

**Checkpoint**: Foundation ready - `cargo test -p evefrontier-lib` passes, foundational types exist

---

## Phase 3: User Story 1 - CI Freshness Validation (Priority: P1) 🎯 MVP

**Goal**: CI automatically verifies spatial index matches dataset before release

**Independent Test**: Run `verify_freshness()` with matching and mismatched artifacts

### Tests for User Story 1 (TDD - Write tests FIRST)

- [X] T012 [P] [US1] Write test `test_compute_dataset_checksum` for fixture database in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`
- [X] T013 [P] [US1] Write test `test_read_release_tag_exists` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`
- [X] T014 [P] [US1] Write test `test_read_release_tag_missing` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`
- [X] T015 [P] [US1] Write test `test_verify_freshness_fresh` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`
- [X] T016 [P] [US1] Write test `test_verify_freshness_stale` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`
- [X] T017 [P] [US1] Write test `test_verify_freshness_missing` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`
- [X] T018 [P] [US1] Write test `test_verify_freshness_legacy_format` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`
- [X] T019 [P] [US1] Write test `test_verify_freshness_dataset_missing` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`

### Implementation for User Story 1

- [X] T020 [US1] Implement `compute_dataset_checksum()` function in `crates/evefrontier-lib/src/spatial.rs` using streaming SHA-256
- [X] T021 [US1] Implement `read_release_tag()` function in `crates/evefrontier-lib/src/spatial.rs` to parse `.db.release` marker
- [X] T022 [US1] Implement `verify_freshness()` function in `crates/evefrontier-lib/src/spatial.rs` returning `FreshnessResult`
- [X] T023 [US1] Add `spatial-index-freshness` job to `.github/workflows/ci.yml` after `test` job, calling `cargo run -p evefrontier-cli -- index-verify`

**Checkpoint**: `verify_freshness()` works with all result variants, CI job added (will fail until CLI exists)

---

## Phase 4: User Story 2 - Source Metadata in Index (Priority: P1)

**Goal**: Spatial index files contain embedded source dataset metadata (v2 format)

**Independent Test**: Build index, save to disk, reload, verify metadata accessible

### Tests for User Story 2 (TDD - Write tests FIRST)

- [X] T024 [P] [US2] Write test `test_build_with_metadata` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`
- [X] T025 [P] [US2] Write test `test_source_metadata_accessor` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`
- [X] T026 [P] [US2] Write test `test_save_load_v2_format` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`
- [X] T027 [P] [US2] Write test `test_load_v1_format_no_metadata` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`
- [X] T028 [P] [US2] Write test `test_v2_header_flags` in `crates/evefrontier-lib/tests/spatial_index_metadata.rs`

### Implementation for User Story 2

- [X] T029 [US2] Add `FLAG_HAS_METADATA` constant (bit 1) to `crates/evefrontier-lib/src/spatial.rs`
- [X] T030 [US2] Add `INDEX_VERSION_V2` constant (value 2) to `crates/evefrontier-lib/src/spatial.rs`
- [X] T031 [US2] Implement `SpatialIndex::build_with_metadata()` in `crates/evefrontier-lib/src/spatial.rs`
- [X] T032 [US2] Implement `SpatialIndex::source_metadata()` getter in `crates/evefrontier-lib/src/spatial.rs`
- [X] T033 [US2] Modify `SpatialIndex::save()` to write v2 format with metadata section in `crates/evefrontier-lib/src/spatial.rs`
- [X] T034 [US2] Modify `SpatialIndex::load()` to read v1 and v2 formats in `crates/evefrontier-lib/src/spatial.rs`
- [X] T035 [US2] Update `index-build` CLI command to use `build_with_metadata()` in `crates/evefrontier-cli/src/main.rs`

**Checkpoint**: Index files contain v2 format with metadata, backward compatible with v1 loading

---

## Phase 5: User Story 3 - CLI Freshness Verification (Priority: P2)

**Goal**: Developers can verify freshness locally with `evefrontier-cli index-verify`

**Independent Test**: Run CLI command, verify output format and exit codes

### Tests for User Story 3 (TDD - Write tests FIRST)

- [X] T036 [P] [US3] Write integration test `test_index_verify_fresh` using assert_cmd in `crates/evefrontier-cli/tests/cli_tests.rs`
- [X] T037 [P] [US3] Write integration test `test_index_verify_stale` using assert_cmd in `crates/evefrontier-cli/tests/cli_tests.rs`
- [X] T038 [P] [US3] Write integration test `test_index_verify_missing` using assert_cmd in `crates/evefrontier-cli/tests/cli_tests.rs`
- [X] T039 [P] [US3] Write integration test `test_index_verify_json_output` using assert_cmd in `crates/evefrontier-cli/tests/cli_tests.rs`
- [X] T040 [P] [US3] Write integration test `test_index_verify_exit_codes` using assert_cmd in `crates/evefrontier-cli/tests/cli_tests.rs`

### Implementation for User Story 3

- [X] T041 [US3] Add `IndexVerify` subcommand struct with options (--data-dir, --json, --quiet, --strict) in `crates/evefrontier-cli/src/main.rs`
- [X] T042 [US3] Implement `handle_index_verify()` function in `crates/evefrontier-cli/src/main.rs`
- [X] T043 [US3] Implement human-readable output formatting in `handle_index_verify()`
- [X] T044 [US3] Implement JSON output formatting using `VerifyOutput` struct
- [X] T045 [US3] Implement exit code mapping from `FreshnessResult` variants

**Checkpoint**: CLI `index-verify` command works with all flags and exit codes

---

## Phase 6: User Story 4 - Operational Documentation (Priority: P3)

**Goal**: Clear documentation for regenerating spatial index after dataset updates

**Independent Test**: Review documentation confirms procedures are actionable

### Implementation for User Story 4

- [X] T046 [P] [US4] Add "index-verify" command documentation to `docs/USAGE.md`
- [X] T047 [P] [US4] Add "Regenerating Spatial Index" section to `docs/USAGE.md`
- [X] T048 [P] [US4] Add "Troubleshooting CI Failures" section to `docs/USAGE.md`
- [X] T049 [P] [US4] Add "Spatial Index Format v2" section to `docs/USAGE.md`
- [X] T049b [P] [US4] Add "Lambda Freshness Behavior" section to `docs/USAGE.md` explaining that freshness is validated at build-time only (bundled artifacts skip runtime check)

**Checkpoint**: Documentation complete with operational procedures

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup and validation

- [ ] T050 Update `docs/TODO.md` to mark "Add CI to verify spatial index artifact freshness" as complete
- [ ] T051 [P] Add CHANGELOG.md entry under Unreleased with feature description
- [ ] T052 [P] Run `cargo clippy --workspace` and fix any warnings
- [ ] T053 [P] Run `cargo fmt --all` and commit formatting changes
- [ ] T054 Run `cargo test --workspace` to verify all tests pass
- [ ] T055 Run quickstart.md validation: build index, verify freshness, test CI job locally

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational - verification functions
- **User Story 2 (Phase 4)**: Depends on Foundational - v2 format and metadata embedding
- **User Story 3 (Phase 5)**: Depends on US1 + US2 - CLI wraps library functions
- **User Story 4 (Phase 6)**: Depends on US3 - documents CLI command
- **Polish (Phase 7)**: Depends on all stories complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - Core verification logic
- **User Story 2 (P1)**: Can start after Foundational - Can run in parallel with US1
- **User Story 3 (P2)**: Depends on US1 (verify_freshness) + US2 (build_with_metadata) - CLI integration
- **User Story 4 (P3)**: Depends on US3 - Documentation requires CLI to be complete

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD)
- Data structures before functions
- Functions before integration
- Story complete before moving to next priority

### Parallel Opportunities

Within **Phase 2 (Foundational)**:
- T004, T005, T006 can run in parallel (different test functions)
- T007, T008, T009 can run in parallel (different structs)

Within **Phase 3 (US1)**:
- T012-T019 can run in parallel (all tests)
- T020, T021 can run in parallel (independent functions)

Within **Phase 4 (US2)**:
- T024-T028 can run in parallel (all tests)
- T029, T030 can run in parallel (constants)

Within **Phase 5 (US3)**:
- T036-T040 can run in parallel (all tests)

Within **Phase 6 (US4)**:
- T046-T049 can run in parallel (different doc sections)

Within **Phase 7 (Polish)**:
- T051, T052, T053 can run in parallel

---

## Parallel Example: Phase 3 (User Story 1)

```bash
# Launch all tests for User Story 1 together:
T012: test_compute_dataset_checksum
T013: test_read_release_tag_exists
T014: test_read_release_tag_missing
T015: test_verify_freshness_fresh
T016: test_verify_freshness_stale
T017: test_verify_freshness_missing
T018: test_verify_freshness_legacy_format
T019: test_verify_freshness_dataset_missing

# After tests written (and failing), implement functions:
T020: compute_dataset_checksum() - streaming SHA-256
T021: read_release_tag() - parse .db.release marker
# These can run in parallel (different functions)

# Then sequential:
T022: verify_freshness() - depends on T020, T021
T023: CI job - depends on T022
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 - verify_freshness() works
4. Complete Phase 4: User Story 2 - v2 format with metadata
5. **STOP and VALIDATE**: Test `verify_freshness()` with fixture dataset
6. CI job can now pass (once CLI exists in US3)

### Incremental Delivery

1. Setup + Foundational → Types exist
2. Add US1 + US2 → Library complete, can verify programmatically
3. Add US3 → CLI `index-verify` command works
4. Add US4 → Documentation complete
5. Polish → Ready for PR

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Each user story should be independently completable and testable
- **TDD**: Verify tests fail before implementing
- Commit after each task or logical group
- Run `cargo test -p evefrontier-lib` after each library task
- Run `cargo test -p evefrontier-cli` after each CLI task
