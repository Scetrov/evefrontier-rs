# Research: Spatial Index Freshness CI Verification

**Feature Branch**: `009-spatial-index-freshness-ci`  
**Date**: 2025-12-30  
**Status**: Complete

## R-001: Spatial Index Format Extension for Source Metadata

### Question
How should we extend the existing spatial index format (v1) to include source dataset metadata
while maintaining backward compatibility awareness?

### Findings
The current spatial index format (v1) has this structure:
- Header: 16 bytes (magic, version, flags, node_count, reserved)
- Body: postcard-serialized nodes, zstd compressed
- Footer: SHA-256 checksum of compressed body

The header has 6 reserved bytes. However, 6 bytes is insufficient for:
- Dataset checksum (32 bytes)
- Release tag (variable, up to 64 bytes)
- Build timestamp (8 bytes)

**Options evaluated:**

1. **Extend header with variable-length metadata section** (CHOSEN)
   - Bump version to 2
   - Add metadata section between header and compressed body
   - Metadata length in header allows forward compatibility
   - Pros: Clean separation, explicit versioning
   - Cons: Breaking change, v1 readers can't read v2 files

2. **Embed metadata in reserved bytes + sidecar file**
   - Keep header compact, write metadata to `.spatial.bin.meta` file
   - Pros: No format change
   - Cons: Two files to track, risk of desync

3. **Embed metadata in footer after checksum**
   - Append variable-length metadata after checksum
   - Reader can detect presence by file size
   - Pros: Backward compatible reading (v1 readers ignore extra data)
   - Cons: Awkward layout, checksum doesn't cover metadata

### Decision
**Option 1: Variable-length metadata section with version bump.**

Rationale:
- Clean, explicit format evolution
- Single file contains all data
- Checksum covers metadata (integrity)
- Version field already exists for this purpose

### Implementation
```rust
// New format v2 layout
const INDEX_VERSION_V2: u8 = 2;

// Header changes:
// - bytes 10-11: metadata section length (u16, little-endian)
// - bytes 12-15: reserved

// Metadata section (after header, before compressed nodes):
struct SourceMetadata {
    dataset_checksum: [u8; 32],  // SHA-256 of dataset file
    dataset_tag: String,         // Release tag (max 64 chars)
    build_timestamp: i64,        // Unix epoch seconds
}

// Serialization: postcard, no compression (small payload)
```

---

## R-002: Dataset Checksum Calculation

### Question
How should we calculate the dataset checksum efficiently for large (>100MB) database files?

### Findings
The e6c3 dataset is approximately 50MB. SHA-256 hashing at ~500MB/s means:
- 50MB / 500 MB/s = ~100ms
- Acceptable for CLI command but too slow for frequent checks

**Options evaluated:**

1. **Full file SHA-256** (CHOSEN for accuracy)
   - Read file in streaming chunks (8KB buffer)
   - Compute SHA-256 incrementally
   - ~100ms for 50MB file
   - Pros: Reliable, deterministic, catches any file change
   - Cons: Slowest option

2. **File size + mtime + first/last 1KB hash**
   - Fast heuristic (~1ms)
   - Pros: Very fast
   - Cons: Can miss content changes if size unchanged

3. **SQLite schema hash**
   - Query `sqlite_master` for CREATE statements
   - Hash the concatenated DDL
   - Pros: Detects schema changes
   - Cons: Doesn't detect data changes

### Decision
**Full file SHA-256 for index building, cached in `.db.release` marker for verification.**

Rationale:
- Index building is infrequent (only on dataset update)
- 100ms overhead is acceptable during build
- For verification, we can cache checksum in the `.db.release` marker
- CI can pre-compute checksum once and reuse

### Implementation
```rust
use sha2::{Digest, Sha256};
use std::io::Read;

fn compute_dataset_checksum(path: &Path) -> Result<[u8; 32]> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(8192, file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    
    Ok(hasher.finalize().into())
}
```

---

## R-003: CI Workflow Integration

### Question
Where should the spatial index freshness check be added in the existing CI workflow?

### Findings
Current CI workflow (`.github/workflows/ci.yml`) structure:
1. `fmt` - Rust formatting check
2. `clippy` - Linting
3. `test` - Unit and integration tests
4. `security-audit` - Cargo audit
5. `changelog-guard` - CHANGELOG enforcement
6. `complexity-check` - Cyclomatic complexity

Release workflow (`.github/workflows/release.yml`):
1. Build multi-arch binaries
2. Generate checksums
3. Sign with cosign
4. Generate SBOM
5. Create GitHub release

**Options evaluated:**

1. **New job in CI workflow** (CHOSEN)
   - Add `spatial-index-freshness` job
   - Run after tests (ensures dataset loads correctly)
   - Fail the workflow if index is stale
   - Pros: Catches issues early, blocks PRs
   - Cons: Adds ~30s to CI time

2. **Gate in release workflow only**
   - Check freshness before creating release
   - Pros: Doesn't slow down PR workflow
   - Cons: Late detection, blocks release last-minute

3. **Nightly scheduled check**
   - Separate workflow running on schedule
   - Posts issue if stale
   - Pros: Non-blocking
   - Cons: Can be ignored, not integrated into PR flow

### Decision
**New job in CI workflow (`spatial-index-freshness`), gated on test job.**

Rationale:
- Early detection in PR workflow
- Blocks merge if stale (shift-left)
- Ensures release workflow doesn't encounter surprises

### Implementation
```yaml
# .github/workflows/ci.yml
spatial-index-freshness:
  name: Spatial Index Freshness
  runs-on: ubuntu-latest
  needs: [test]  # Run after tests ensure dataset loads
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Verify spatial index freshness
      run: cargo run -p evefrontier-cli -- index-verify --data-dir data/
```

---

## R-004: CLI `index-verify` Command Design

### Question
What should the `index-verify` CLI command interface look like?

### Findings
Existing CLI commands follow this pattern:
- `evefrontier-cli download` - Download dataset
- `evefrontier-cli route <from> <to>` - Plan route
- `evefrontier-cli index-build` - Build spatial index

**Interface design:**

```
evefrontier-cli index-verify [OPTIONS]

OPTIONS:
    --data-dir <DIR>   Path to dataset directory
    --json             Output in JSON format
    --quiet            Only output on failure
    --strict           Fail if release tag doesn't match (not just checksum)

EXIT CODES:
    0 - Index is fresh (matches dataset)
    1 - Index is stale (doesn't match dataset)
    2 - Index is missing
    3 - Index format error (legacy v1 or corrupt)
    4 - Dataset not found
```

### Decision
Implement `index-verify` subcommand with JSON output support for CI parsing.

### Implementation
```rust
#[derive(Parser)]
struct IndexVerify {
    /// Path to dataset directory
    #[arg(long, env = "EVEFRONTIER_DATA_DIR")]
    data_dir: Option<PathBuf>,
    
    /// Output in JSON format
    #[arg(long)]
    json: bool,
    
    /// Only output on failure
    #[arg(long)]
    quiet: bool,
}

#[derive(Serialize)]
struct VerifyResult {
    fresh: bool,
    dataset_checksum: String,
    index_source_checksum: Option<String>,
    dataset_tag: Option<String>,
    index_source_tag: Option<String>,
    message: String,
}
```

---

## R-005: Backward Compatibility Strategy

### Question
How do we handle existing v1 spatial index files that don't have source metadata?

### Findings
Options:
1. **Fail on v1 files** (CHOSEN for CI, but warn locally)
2. **Treat v1 as "unknown freshness"**
3. **Auto-migrate v1 to v2 on load**

### Decision
- CI workflow: Fail if v1 index detected (forces rebuild)
- Local CLI: Warn about v1 but allow operations
- `index-build` always produces v2

Rationale:
- CI needs deterministic pass/fail
- Local development shouldn't be blocked by legacy files
- Clear migration path: run `index-build` to upgrade

---

## Summary of Decisions

| Topic | Decision | Rationale |
|-------|----------|-----------|
| Format extension | Version bump to v2 with metadata section | Clean, explicit versioning |
| Checksum method | Full file SHA-256 | Reliable, acceptable performance |
| CI integration | New job in CI workflow | Early detection, blocks PRs |
| CLI command | `index-verify` with JSON output | CI-friendly, scriptable |
| Legacy handling | Fail in CI, warn locally | Forces upgrade, doesn't block dev |
