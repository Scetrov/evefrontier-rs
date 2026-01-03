# Implementation Plan: Spatial Index Freshness CI Verification

**Branch**: `009-spatial-index-freshness-ci` | **Date**: 2025-12-30 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/009-spatial-index-freshness-ci/spec.md`

## Summary

Add source metadata tracking to spatial index files (v2 format) and a CI verification job to
ensure the spatial index artifact matches the current dataset version before release. This
prevents releasing mismatched artifacts that could cause silent routing failures.

## Technical Context

**Language/Version**: Rust 1.91.1 (per `.rust-toolchain`)  
**Primary Dependencies**: sha2 (checksum), postcard+zstd (serialization), clap (CLI)  
**Storage**: File-based (spatial index `.spatial.bin`, dataset `.db`)  
**Testing**: cargo test, integration tests with fixture dataset  
**Target Platform**: Linux (CI), all platforms (local development)  
**Project Type**: Rust workspace with library and CLI crates  
**Performance Goals**: <1s verification time for 50MB dataset  
**Constraints**: Backward compatible loading of v1 files, CI must fail deterministically  
**Scale/Scope**: Single spatial index file per dataset, ~50MB dataset size

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-Driven Development | ✅ PASS | Tests for checksum, metadata embedding, verification |
| II. Library-First Architecture | ✅ PASS | All logic in evefrontier-lib, CLI is thin wrapper |
| III. Architecture Decision Records | ⚠️ N/A | Enhancement, not architectural change |
| IV. Clean Code & Cognitive Load | ✅ PASS | Clear separation of concerns, simple API |
| V. Security-First Development | ✅ PASS | SHA-256 for integrity, no secrets involved |
| VI. Testing Tiers | ✅ PASS | Unit tests + integration tests for verification |
| VII. Refactoring & Tech Debt | ✅ PASS | Extends existing spatial module cleanly |

**Gate Result**: ✅ PASS - All applicable principles satisfied

## Project Structure

### Documentation (this feature)

```text
specs/009-spatial-index-freshness-ci/
├── plan.md                           # This file
├── spec.md                           # Feature specification
├── research.md                       # Phase 0 research findings
├── data-model.md                     # Data structures
├── quickstart.md                     # Usage guide
├── contracts/
│   ├── cli-index-verify.md           # CLI command contract
│   └── lib-spatial-metadata.md       # Library API contract
└── tasks.md                          # Phase 2 output (NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
crates/
├── evefrontier-lib/
│   └── src/
│       ├── spatial.rs               # MODIFY: Add metadata types, v2 format, verify_freshness
│       └── error.rs                 # MODIFY: Add freshness error variants (if needed)
└── evefrontier-cli/
    └── src/
        └── main.rs                  # MODIFY: Add index-verify subcommand

.github/
└── workflows/
    └── ci.yml                       # MODIFY: Add spatial-index-freshness job

docs/
├── USAGE.md                         # MODIFY: Document index-verify command
└── TODO.md                          # MODIFY: Mark task complete
```

**Structure Decision**: Source metadata types and verification logic live in `spatial.rs` following
the existing module pattern. The CLI adds a thin `index-verify` subcommand. CI workflow gets a new
job after the test job.

## Complexity Tracking

> No Constitution violations requiring justification.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |

---

## Phase 0: Research Findings

### R-001: Spatial Index Format Extension

**Decision**: Version bump to v2 with variable-length metadata section  
**Rationale**: Clean, explicit versioning; single file contains all data; checksum covers metadata

Format evolution:
- v1: Header (16B) | Compressed Nodes | Checksum (32B)
- v2: Header (16B) | Metadata Section | Compressed Nodes | Checksum (32B)

### R-002: Dataset Checksum Calculation

**Decision**: Full file SHA-256 with streaming reads  
**Rationale**: Reliable, ~100ms for 50MB file, acceptable for build-time operation

### R-003: CI Workflow Integration

**Decision**: New job `spatial-index-freshness` in CI workflow, after `test` job  
**Rationale**: Early detection in PR workflow, blocks merge if stale

### R-004: CLI Command Design

**Decision**: `index-verify` subcommand with JSON output and exit codes  
**Rationale**: CI-friendly, scriptable, consistent with existing CLI patterns

### R-005: Backward Compatibility

**Decision**: v1 files load successfully but fail CI freshness check  
**Rationale**: Forces upgrade without blocking local development

---

## Phase 1: Design Artifacts

### Generated Artifacts

1. **[data-model.md](./data-model.md)**: Core data structures
   - `DatasetMetadata`: Source information (checksum, tag, timestamp)
   - `FreshnessResult`: Verification outcome enum
   - `VerifyOutput`: CLI output structure
   - Spatial index v2 file format specification

2. **[contracts/cli-index-verify.md](./contracts/cli-index-verify.md)**: CLI contract
   - Command synopsis and options
   - Exit codes (0-5)
   - Human and JSON output formats
   - CI integration examples

3. **[contracts/lib-spatial-metadata.md](./contracts/lib-spatial-metadata.md)**: Library API
   - `DatasetMetadata` type
   - `SpatialIndex::build_with_metadata()`
   - `SpatialIndex::source_metadata()`
   - `compute_dataset_checksum()`
   - `verify_freshness()`

4. **[quickstart.md](./quickstart.md)**: Usage guide
   - Common workflows
   - Troubleshooting CI failures
   - Library usage examples
   - File format changes summary

---

## Phase 2: Implementation Tasks

> **Note**: Detailed task breakdown will be generated by `/speckit.tasks` command.

### High-Level Implementation Order

1. **Library: Data Structures** (TDD)
   - Add `DatasetMetadata` struct to `spatial.rs`
   - Add `FreshnessResult` enum to `spatial.rs`
   - Write unit tests for serialization/deserialization

2. **Library: Checksum Functions** (TDD)
   - Implement `compute_dataset_checksum()`
   - Implement `read_release_tag()`
   - Write unit tests with fixture files

3. **Library: v2 Format** (TDD)
   - Modify `SpatialIndex::save()` to write v2 format
   - Modify `SpatialIndex::load()` to read v1 and v2 formats
   - Add `build_with_metadata()` constructor
   - Add `source_metadata()` getter
   - Write integration tests with fixture database

4. **Library: Freshness Verification** (TDD)
   - Implement `verify_freshness()` function
   - Write unit tests for all `FreshnessResult` variants

5. **CLI: index-verify Command**
   - Add `IndexVerify` subcommand struct
   - Implement handler calling library functions
   - Add JSON output support
   - Write integration tests using `assert_cmd`

6. **CI: Freshness Job**
   - Add `spatial-index-freshness` job to `ci.yml`
   - Configure dependency on `test` job
   - Test with fixture dataset

7. **Documentation**
   - Update `docs/USAGE.md` with `index-verify` command
   - Update `docs/TODO.md` to mark task complete
   - Update CHANGELOG.md

### Testing Strategy

- Unit tests for all new functions in `spatial.rs`
- Integration tests with `docs/fixtures/minimal/static_data.db`
- CLI integration tests using `assert_cmd` crate
- CI workflow tested with both fresh and stale scenarios

---

## Re-Check: Constitution Compliance (Post-Design)

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Test-Driven Development | ✅ | Tests specified for all new functions |
| II. Library-First Architecture | ✅ | All logic in evefrontier-lib |
| III. Architecture Decision Records | ⚠️ N/A | Enhancement only |
| IV. Clean Code & Cognitive Load | ✅ | Simple API, clear naming |
| V. Security-First Development | ✅ | SHA-256 integrity check |
| VI. Testing Tiers | ✅ | Unit + integration + CI tests |
| VII. Refactoring & Tech Debt | ✅ | Clean extension of spatial module |

**Final Gate Result**: ✅ PASS

---

## Appendix: File Changes Summary

### New Files
- None (all changes are modifications)

### Modified Files
| File | Change |
|------|--------|
| `crates/evefrontier-lib/src/spatial.rs` | Add metadata types, v2 format, verification |
| `crates/evefrontier-cli/src/main.rs` | Add `index-verify` subcommand |
| `.github/workflows/ci.yml` | Add `spatial-index-freshness` job |
| `docs/USAGE.md` | Document `index-verify` command |
| `docs/TODO.md` | Mark task complete |
| `CHANGELOG.md` | Add entry for this feature |

### Test Files
| File | Coverage |
|------|----------|
| `crates/evefrontier-lib/tests/spatial_index.rs` | Existing file, add metadata tests |
| `crates/evefrontier-cli/tests/cli_tests.rs` | Add `index-verify` integration tests |
