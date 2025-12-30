# Data Model: Spatial Index Freshness CI Verification

**Feature Branch**: `009-spatial-index-freshness-ci`  
**Date**: 2025-12-30  
**Status**: Complete

## Overview

This document defines the data structures for spatial index source metadata and freshness
verification results. These structures enable tracking the relationship between a spatial index
artifact and its source dataset.

## Core Entities

### DatasetMetadata

Represents information about the source dataset used to build a spatial index. This metadata is
embedded in the spatial index file header (v2 format).

```rust
/// Metadata about the source dataset used to build a spatial index.
///
/// Embedded in the spatial index file to enable freshness verification.
/// The checksum provides cryptographic proof of the exact dataset version,
/// while the tag provides human-readable identification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetMetadata {
    /// SHA-256 checksum of the source dataset file.
    ///
    /// Computed by hashing the entire `.db` file contents.
    /// This is the primary identifier for freshness verification.
    pub checksum: [u8; 32],
    
    /// Release tag from the `.db.release` marker file (e.g., "e6c3").
    ///
    /// May be None if the dataset was not downloaded via the standard
    /// release mechanism (e.g., manually placed file).
    pub release_tag: Option<String>,
    
    /// Unix timestamp (seconds since epoch) when the index was built.
    ///
    /// Used for informational/debugging purposes only; not used for
    /// freshness verification.
    pub build_timestamp: i64,
}
```

**Invariants:**
- `checksum` is always 32 bytes (SHA-256 digest size)
- `release_tag` if present, is at most 64 characters
- `build_timestamp` is positive and represents a valid Unix timestamp

**Validation Rules:**
- Checksum must be non-zero (all-zero indicates uninitialized)
- Release tag must match pattern `^[a-zA-Z0-9._-]{1,64}$` if present

---

### FreshnessResult

Represents the outcome of comparing a spatial index's source metadata against the current dataset.

```rust
/// Result of verifying spatial index freshness against the current dataset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FreshnessResult {
    /// Index is fresh - source metadata matches current dataset.
    Fresh {
        /// The matching checksum (hex-encoded).
        checksum: String,
        /// The matching release tag, if present.
        release_tag: Option<String>,
    },
    
    /// Index is stale - source metadata doesn't match current dataset.
    Stale {
        /// Expected checksum from current dataset (hex-encoded).
        expected_checksum: String,
        /// Actual checksum from index source metadata (hex-encoded).
        actual_checksum: String,
        /// Expected release tag, if available.
        expected_tag: Option<String>,
        /// Actual release tag from index, if available.
        actual_tag: Option<String>,
    },
    
    /// Index is in legacy format (v1) without source metadata.
    LegacyFormat {
        /// Index file path.
        index_path: String,
        /// Human-readable message explaining the situation.
        message: String,
    },
    
    /// Spatial index file is missing.
    Missing {
        /// Expected index file path.
        expected_path: String,
    },
    
    /// Dataset file is missing.
    DatasetMissing {
        /// Expected dataset file path.
        expected_path: String,
    },
    
    /// Error occurred during verification.
    Error {
        /// Error message.
        message: String,
    },
}
```

**State Transitions:**
- `Fresh` → terminal (success)
- `Stale` → user runs `index-build` → `Fresh`
- `LegacyFormat` → user runs `index-build` → `Fresh`
- `Missing` → user runs `index-build` → `Fresh`

---

### VerifyOutput

Structured output for the `index-verify` CLI command, designed for both human and machine
consumption.

```rust
/// Structured output for the index-verify CLI command.
///
/// Supports both human-readable (default) and JSON (--json flag) formats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOutput {
    /// Verification result.
    pub result: FreshnessResult,
    
    /// Whether the index is considered fresh (for exit code determination).
    pub is_fresh: bool,
    
    /// Recommended action, if any.
    pub recommended_action: Option<String>,
    
    /// Additional diagnostic information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<VerifyDiagnostics>,
}

/// Diagnostic information for troubleshooting freshness issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyDiagnostics {
    /// Path to the dataset file checked.
    pub dataset_path: String,
    
    /// Path to the spatial index file checked.
    pub index_path: String,
    
    /// Dataset file size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_size: Option<u64>,
    
    /// Index file size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_size: Option<u64>,
    
    /// Index format version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_version: Option<u8>,
    
    /// Time taken for verification in milliseconds.
    pub verification_time_ms: u64,
}
```

---

## Spatial Index File Format (v2)

### Header Structure

```text
Offset  Size  Field                Description
------  ----  -----                -----------
0       4     magic                b"EFSI" (EVE Frontier Spatial Index)
4       1     version              2 (v2 format)
5       1     flags                Bit 0: has_temperature, Bit 1: has_metadata
6       4     node_count           Number of indexed systems (u32 LE)
10      2     metadata_length      Length of metadata section (u16 LE)
12      4     reserved             Reserved for future use

Total: 16 bytes
```

**Version Semantics:**
- v1 files: header version byte = 1, flags bit 1 = 0 (no metadata)
- v2 files: header version byte = 2, flags bit 1 = 1 (has metadata)

The `INDEX_VERSION_V2` constant (value 2) in code corresponds to the header version byte.

### Metadata Section (new in v2)

```text
Offset  Size  Field                Description
------  ----  -----                -----------
0       32    dataset_checksum     SHA-256 of source dataset file
32      1     tag_length           Length of release tag (0 if none)
33      N     release_tag          UTF-8 release tag (N = tag_length, max 64)
33+N    8     build_timestamp      Unix timestamp (i64 LE)

Total: 41 + tag_length bytes (min 41, max 105)
```

### Body Structure (unchanged from v1)

```text
- Compressed nodes: postcard-serialized Vec<IndexNode>, zstd compressed
```

### Footer Structure (unchanged from v1)

```text
Offset  Size  Field                Description
------  ----  -----                -----------
0       32    checksum             SHA-256 of compressed body

Total: 32 bytes
```

**Note:** The footer checksum covers only the compressed nodes body, not the header or metadata
section. This matches v1 behavior. A future v3 could extend checksum coverage.

---

## Dataset Release Marker Format

The `.db.release` marker file is created by the dataset downloader and contains:

```text
requested=latest
resolved=e6c3
```

Or for explicit tag downloads:

```text
requested=tag
resolved=e6c3
```

**Enhancement for freshness verification:**

Add optional checksum line:

```text
requested=latest
resolved=e6c3
checksum=a1b2c3d4e5f6...64 hex chars...
```

This allows fast freshness verification without re-hashing the entire dataset file.

---

## Relationships

```
┌─────────────────┐
│     Dataset     │
│  (static_data   │
│     .db)        │
├─────────────────┤
│ • File content  │
│ • SHA-256 hash  │
└────────┬────────┘
         │ builds
         ▼
┌─────────────────┐
│  SpatialIndex   │
│  (.spatial.bin) │
├─────────────────┤
│ • KD-tree nodes │
│ • DatasetMeta-  │
│   data (embed)  │
└────────┬────────┘
         │ verified by
         ▼
┌─────────────────┐
│ FreshnessResult │
├─────────────────┤
│ • Fresh         │
│ • Stale         │
│ • LegacyFormat  │
│ • Missing       │
└─────────────────┘
```

---

## Migration Notes

### v1 → v2 Index Files

- v1 files: header[5] flags has bit 1 clear (no metadata)
- v2 files: header[5] flags has bit 1 set (has metadata)

The `SpatialIndex::load()` function will:
1. Read header and check version
2. If version == 1 or flags bit 1 is clear: return `SourceMetadata: None`
3. If version == 2 and flags bit 1 is set: read and parse metadata section

The `SpatialIndex::save()` function will:
1. Always write v2 format
2. Always set flags bit 1 (has metadata)
3. Require `DatasetMetadata` parameter (or compute if dataset path provided)
