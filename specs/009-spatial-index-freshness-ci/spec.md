# Feature Specification: Spatial Index Freshness CI Verification

**Feature Branch**: `009-spatial-index-freshness-ci`  
**Created**: 2025-12-30  
**Status**: Draft  
**Input**: docs/TODO.md "Security & Operations" section - "Add CI to verify spatial index artifact freshness against dataset version"

## Problem Statement

The EVE Frontier toolkit uses two key artifacts:

1. **Dataset** (`static_data.db`) - SQLite database containing solar systems, gates, and celestial data
2. **Spatial Index** (`static_data.db.spatial.bin`) - Precomputed KD-tree for efficient spatial queries

Currently, there is no automated verification that the spatial index was built from the current
dataset. If the dataset is updated but the spatial index is not rebuilt, the index will contain
stale data leading to:

- Missing systems (new systems in dataset won't appear in spatial queries)
- Stale coordinates (system positions may have changed)
- Incorrect temperature data (min_external_temp values may have changed)
- Silent failures where spatial routing produces suboptimal or incorrect paths

## Goals

1. Add metadata to spatial index files linking them to their source dataset
2. Create CI job that verifies spatial index freshness before release
3. Provide clear error messaging when spatial index is out of sync
4. Document operational procedure for regenerating spatial index after dataset updates

## Non-Goals

1. Automatic spatial index regeneration in CI (manual trigger preferred for explicit control)
2. Runtime freshness validation (too expensive for cold-start)
3. Migration of existing spatial index files (new format only going forward)
4. Backwards compatibility with existing spatial index format (version bump acceptable)

## User Scenarios & Testing

### User Story 1 - CI Freshness Validation (Priority: P1)

A release engineer wants CI to automatically verify that the spatial index artifact matches the
dataset version before creating a release, preventing release of mismatched artifacts.

**Why this priority**: This is the core requirement - preventing broken releases is the primary
goal. All other scenarios support this.

**Independent Test**: Run CI workflow with matching and mismatched artifacts to verify detection.

**Acceptance Scenarios**:

1. **Given** a spatial index built from dataset v1.0.0, **When** CI runs with dataset v1.0.0,
   **Then** the freshness check passes.
2. **Given** a spatial index built from dataset v1.0.0, **When** CI runs with dataset v2.0.0,
   **Then** the freshness check fails with clear error message.
3. **Given** a spatial index without source metadata, **When** CI runs, **Then** the freshness
   check fails with message indicating legacy index format.
4. **Given** no spatial index exists, **When** CI runs, **Then** the freshness check fails with
   message indicating missing index.

---

### User Story 2 - Source Metadata in Index (Priority: P1)

A developer wants the spatial index file to contain metadata about its source dataset so that
freshness can be verified programmatically.

**Why this priority**: Required for User Story 1 - without metadata, freshness cannot be verified.

**Independent Test**: Build index from dataset, inspect metadata, verify source info present.

**Acceptance Scenarios**:

1. **Given** a dataset with release tag `e6c3`, **When** `index-build` command runs, **Then** the
   generated spatial index includes source metadata with `dataset_tag: "e6c3"`.
2. **Given** a dataset with file checksum X, **When** `index-build` command runs, **Then** the
   generated spatial index includes `dataset_checksum: "X"`.
3. **Given** a spatial index with source metadata, **When** loaded with `SpatialIndex::load()`,
   **Then** the metadata is accessible via `source_metadata()` method.

---

### User Story 3 - CLI Freshness Verification (Priority: P2)

A developer wants a CLI command to verify spatial index freshness locally before committing,
catching mismatches early in the development cycle.

**Why this priority**: Supports shift-left detection but CI (US1) is the critical gate.

**Independent Test**: Run CLI command with matching and mismatched artifacts locally.

**Acceptance Scenarios**:

1. **Given** `evefrontier-cli index-verify` is run, **When** index matches dataset, **Then**
   output shows "Spatial index is fresh" with exit code 0.
2. **Given** `evefrontier-cli index-verify` is run, **When** index doesn't match dataset, **Then**
   output shows mismatch details and exit code 1.
3. **Given** `evefrontier-cli index-verify --json` is run, **Then** output is JSON with fields:
   `fresh`, `dataset_tag`, `index_source_tag`, `dataset_checksum`, `index_source_checksum`.

---

### User Story 4 - Operational Documentation (Priority: P3)

A platform operator wants clear documentation on when and how to regenerate the spatial index
after dataset updates.

**Why this priority**: Important for maintainability but not blocking release functionality.

**Independent Test**: Documentation review confirms procedures are clear and actionable.

**Acceptance Scenarios**:

1. **Given** documentation exists, **When** operator reads "Regenerating Spatial Index" section,
   **Then** steps include: download dataset, run index-build, commit artifacts.
2. **Given** documentation exists, **When** operator reads "CI Failure Resolution" section,
   **Then** steps explain how to resolve freshness failures.

---

### Edge Cases

- What if dataset has no `.db.release` marker? Use file checksum only (warn about missing tag).
- What if spatial index is newer than dataset? Still fail (index must match exactly).
- What if dataset checksum calculation is expensive? Use file mtime + size as fast check first.
- What if running in Lambda with bundled artifacts? Skip freshness check (build-time validation).

## Requirements

### Functional Requirements

- **FR-001**: Spatial index format MUST include source metadata section in header
- **FR-002**: Source metadata MUST contain dataset file SHA-256 checksum (32 bytes)
- **FR-003**: Source metadata MUST contain dataset release tag (variable-length string, max 64 chars)
- **FR-004**: `SpatialIndex::build()` MUST accept optional `DatasetMetadata` parameter
- **FR-005**: `SpatialIndex::source_metadata()` MUST return the embedded source metadata
- **FR-006**: CI workflow MUST fail if spatial index source checksum doesn't match dataset checksum
- **FR-007**: CLI MUST provide `index-verify` subcommand for local verification
- **FR-008**: Spatial index format version MUST increment (v1 → v2) for new metadata section

### Non-Functional Requirements

- **NFR-001**: Source metadata MUST add <128 bytes to spatial index file size
- **NFR-002**: Freshness verification MUST complete in <1 second for typical datasets
- **NFR-003**: Checksum calculation MUST use streaming to avoid loading entire dataset into memory

### Key Entities

- **DatasetMetadata**: Source information embedded in spatial index (checksum, tag, timestamp)
- **FreshnessResult**: Verification result (fresh, stale, unknown, missing)
- **SourceChecksum**: SHA-256 hash of dataset file (32 bytes)

## Success Criteria

### Measurable Outcomes

- **SC-001**: CI release workflow fails when spatial index doesn't match dataset
- **SC-002**: `index-verify` command returns correct exit code for fresh/stale states
- **SC-003**: New spatial index files contain source metadata readable by `SpatialIndex::load()`
- **SC-004**: Documentation includes step-by-step index regeneration procedure
- **SC-005**: Zero false positives in freshness detection (fresh index never flagged as stale)

## Technical Notes (for Plan phase)

### Existing Infrastructure

- `spatial.rs` module has `save()` and `load()` methods with versioned header
- Current header is 16 bytes: magic (4), version (1), flags (1), node_count (4), reserved (6)
- Dataset download creates `.db.release` marker with tag info
- CI workflow already runs tests and builds artifacts

### Format Evolution

Current spatial index format (v1):
```text
Header (16 bytes) | Compressed Nodes | Checksum (32 bytes)
```

Proposed format (v2):
```text
Header (16 bytes) | Source Metadata (variable) | Compressed Nodes | Checksum (32 bytes)
```

Source metadata section:
```text
- Metadata length: u16 (2 bytes)
- Dataset checksum: [u8; 32] (32 bytes)
- Tag length: u8 (1 byte)
- Tag: [u8; tag_length] (variable, max 64 bytes)
- Timestamp: i64 (8 bytes, Unix epoch seconds)
```

### Dependencies

- `sha2` crate already in workspace for checksum calculation
- No new dependencies required
