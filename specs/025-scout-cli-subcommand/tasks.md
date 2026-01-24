# Tasks: Scout CLI Subcommand

**Input**: Design documents from `/specs/025-scout-cli-subcommand/`  
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅, quickstart.md ✅

**Tests**: Integration tests included as explicitly requested in spec.md (NFR-3).

**Organization**: Tasks grouped by user story (FR-1/FR-2) to enable independent implementation.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[US1]**: Scout Gates subcommand (FR-1)
- **[US2]**: Scout Range subcommand (FR-2 through FR-5)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: CLI argument types and module scaffolding shared by both subcommands

- [x] T001 Add `ScoutCommandArgs`, `ScoutSubcommand`, `ScoutGatesArgs`, `ScoutRangeArgs` structs to `crates/evefrontier-cli/src/main.rs`
- [x] T002 Add `Scout(ScoutCommandArgs)` variant to `Command` enum in `crates/evefrontier-cli/src/main.rs`
- [x] T003 Create `crates/evefrontier-cli/src/commands/scout.rs` with module structure and placeholder functions
- [x] T004 Export `scout` module from `crates/evefrontier-cli/src/commands/mod.rs`

**Checkpoint**: `cargo build -p evefrontier-cli` compiles; `evefrontier-cli scout --help` shows subcommands

---

## Phase 2: Foundational (Shared Output Helpers)

**Purpose**: Output formatting functions used by both subcommands

- [x] T005 Add `ScoutGatesResult`, `GateNeighbor` structs to `crates/evefrontier-cli/src/output_helpers.rs` (moved to avoid circular deps)
- [x] T006 [P] Add `ScoutRangeResult`, `RangeNeighbor`, `RangeQueryParams` structs to `crates/evefrontier-cli/src/output_helpers.rs`
- [x] T007 Add `format_scout_gates_basic()` function to `crates/evefrontier-cli/src/output_helpers.rs`
- [x] T008 [P] Add `format_scout_gates_enhanced()` function to `crates/evefrontier-cli/src/output_helpers.rs`
- [x] T009 [P] Add `format_scout_range_basic()` function to `crates/evefrontier-cli/src/output_helpers.rs`
- [x] T010 [P] Add `format_scout_range_enhanced()` function to `crates/evefrontier-cli/src/output_helpers.rs`

**Checkpoint**: All struct types defined; formatting functions compile (not yet wired)

---

## Phase 3: User Story 1 — Scout Gates (Priority: P1) 🎯 MVP

**Goal**: `evefrontier-cli scout gates <SYSTEM>` lists gate-connected neighbors

**Independent Test**: `evefrontier-cli scout gates "Nod"` returns neighbors; `--format json` produces valid JSON

### Tests for User Story 1

- [x] T011 [US1] Create `crates/evefrontier-cli/tests/scout_gates.rs` with integration tests:
  - `test_scout_gates_basic_output`
  - `test_scout_gates_json_output`
  - `test_scout_gates_unknown_system_suggests_matches`
  - `test_scout_gates_empty_result`

### Implementation for User Story 1

- [x] T012 [US1] Implement `handle_scout_gates()` in `crates/evefrontier-cli/src/commands/scout.rs`:
  - Load starmap from dataset
  - Resolve system by name (fuzzy matching on failure)
  - Query `starmap.adjacency.get(&system_id)`
  - Format output per `--format` flag
- [x] T013 [US1] Wire `ScoutSubcommand::Gates` dispatch in `crates/evefrontier-cli/src/main.rs` run function
- [x] T014 [US1] Add error handling for unknown system with fuzzy suggestions (reuse existing pattern)

**Checkpoint**: `evefrontier-cli scout gates "Nod"` works; all US1 tests pass

---

## Phase 4: User Story 2 — Scout Range (Priority: P2)

**Goal**: `evefrontier-cli scout range <SYSTEM>` lists systems within spatial radius with filters

**Independent Test**: `evefrontier-cli scout range "Nod" --limit 5` returns nearby systems; filters work

### Tests for User Story 2

- [x] T015 [US2] Create `crates/evefrontier-cli/tests/scout_range.rs` with integration tests:
  - `test_scout_range_default_limit`
  - `test_scout_range_with_radius`
  - `test_scout_range_with_max_temp`
  - `test_scout_range_combined_filters`
  - `test_scout_range_json_output`
  - `test_scout_range_unknown_system_suggests_matches`

### Implementation for User Story 2

- [x] T016 [US2] Implement `handle_scout_range()` in `crates/evefrontier-cli/src/commands/scout.rs`:
  - Load starmap and spatial index (auto-build if missing with warning)
  - Resolve system by name (fuzzy matching on failure)
  - Build `NeighbourQuery { k: limit, radius, max_temperature }`
  - Call `spatial_index.nearest_filtered(position, &query)`
  - Format output per `--format` flag
- [x] T017 [US2] Wire `ScoutSubcommand::Range` dispatch in `crates/evefrontier-cli/src/main.rs` run function
- [x] T018 [US2] Add validation for `--limit` (1-100 range) and positive values for `--radius`/`--max-temp`

**Checkpoint**: `evefrontier-cli scout range "Nod" --limit 5` works; all US2 tests pass

---

## Phase 5: Polish & Documentation

**Purpose**: Documentation updates and final validation

- [x] T019 [P] Add "Scout Command" section to `docs/USAGE.md` with examples from quickstart.md
- [x] T020 [P] Update CLI command list in `README.md` to include `scout gates` and `scout range`
- [x] T021 [P] Add entry to `CHANGELOG.md` under Unreleased: `[Added] scout gates and scout range CLI subcommands`
- [x] T022 Run `cargo test --workspace` to validate all tests pass
- [x] T023 Run `cargo clippy --all-targets` to ensure no warnings
- [x] T024 Validate quickstart.md examples work as documented

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1: Setup
    ↓
Phase 2: Foundational (output helpers)
    ↓
    ├── Phase 3: User Story 1 (Scout Gates) 🎯 MVP
    │       ↓
    └── Phase 4: User Story 2 (Scout Range)
            ↓
        Phase 5: Polish & Documentation
```

### User Story Dependencies

- **User Story 1 (Scout Gates)**: Can start after Phase 2 — No dependencies on US2
- **User Story 2 (Scout Range)**: Can start after Phase 2 — No dependencies on US1

### Within Each User Story

- Tests written first (T011, T015)
- Implementation follows (T012-T014, T016-T018)
- Tests validated passing before moving to next story

### Parallel Opportunities

```bash
# Phase 2: All output helpers can be written in parallel
T007, T008, T009, T010 — different functions, same file section

# Phase 3+4: User stories can run in parallel if staffed
US1 (T011-T014) || US2 (T015-T018)

# Phase 5: All documentation updates in parallel
T019, T020, T021 — different files
```

---

## Parallel Example: Phase 2 (Output Helpers)

```bash
# Launch all formatting functions together:
Task T007: "Add format_scout_gates_basic() to output_helpers.rs"
Task T008: "Add format_scout_gates_enhanced() to output_helpers.rs"
Task T009: "Add format_scout_range_basic() to output_helpers.rs"
Task T010: "Add format_scout_range_enhanced() to output_helpers.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: Foundational output helpers (T005-T010)
3. Complete Phase 3: User Story 1 — Scout Gates (T011-T014)
4. **STOP and VALIDATE**: `evefrontier-cli scout gates "Nod"` works
5. Can ship/demo MVP with just gate scouting

### Full Feature Delivery

1. Complete MVP (above)
2. Add Phase 4: User Story 2 — Scout Range (T015-T018)
3. Complete Phase 5: Polish & Documentation (T019-T024)
4. Full feature ready for release

---

## Summary

| Phase | Tasks | Parallel? | Estimated LOC |
|-------|-------|-----------|---------------|
| 1. Setup | T001-T004 | No | ~80 |
| 2. Foundational | T005-T010 | Yes (T007-T010) | ~150 |
| 3. US1: Scout Gates | T011-T014 | No | ~120 |
| 4. US2: Scout Range | T015-T018 | No | ~150 |
| 5. Polish | T019-T024 | Yes (T019-T021) | ~100 |
| **Total** | **24 tasks** | | **~600 LOC** |

---

## Notes

- All file paths are relative to repository root
- Use `docs/fixtures/minimal/static_data.db` for integration tests
- Follow existing patterns in `main.rs` for argument parsing
- Follow existing patterns in `output_helpers.rs` for formatting
- Fuzzy matching pattern available in `evefrontier-lib` via `starmap.fuzzy_system_matches()`
