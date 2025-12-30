# Quickstart: Spatial Index Freshness CI Verification

**Feature Branch**: `009-spatial-index-freshness-ci`  
**Date**: 2025-12-30

## Overview

This feature adds source metadata tracking to spatial index files, enabling automated verification
that the index matches the current dataset version. This prevents releasing mismatched artifacts.

## Quick Reference

### Build Index with Source Tracking (v2 format)

```bash
# Build spatial index from current dataset
evefrontier-cli index-build --data-dir data/

# Force rebuild (even if index exists)
evefrontier-cli index-build --data-dir data/ --force
```

### Verify Index Freshness

```bash
# Basic verification
evefrontier-cli index-verify --data-dir data/

# JSON output for CI
evefrontier-cli index-verify --data-dir data/ --json

# Exit code only (for scripts)
evefrontier-cli index-verify --data-dir data/ --quiet
echo $?  # 0=fresh, 1=stale, 2=missing, 3=legacy, 4=no-dataset
```

### CI Integration

Add to `.github/workflows/ci.yml`:

```yaml
spatial-index-freshness:
  name: Spatial Index Freshness
  runs-on: ubuntu-latest
  needs: [test]
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        toolchain: '1.91.1'
    - name: Verify spatial index freshness
      run: cargo run -p evefrontier-cli --release -- index-verify --data-dir data/
```

## Common Workflows

### After Updating Dataset

1. Download new dataset:
   ```bash
   evefrontier-cli download
   ```

2. Rebuild spatial index:
   ```bash
   evefrontier-cli index-build --force
   ```

3. Verify freshness:
   ```bash
   evefrontier-cli index-verify
   ```

4. Commit both artifacts:
   ```bash
   git add data/static_data.db data/static_data.db.spatial.bin
   git commit -m "chore(data): update dataset and spatial index to e6c4"
   ```

### Troubleshooting CI Failures

**Error: "Spatial index is STALE"**
```bash
# Local fix
evefrontier-cli index-build --force
git add data/static_data.db.spatial.bin
git commit -m "fix: rebuild spatial index for current dataset"
git push
```

**Error: "Legacy format (v1)"**
```bash
# Upgrade to v2 format
evefrontier-cli index-build --force
git add data/static_data.db.spatial.bin
git commit -m "chore: upgrade spatial index to v2 format"
git push
```

**Error: "Spatial index not found"**
```bash
# Create index from scratch
evefrontier-cli index-build
git add data/static_data.db.spatial.bin
git commit -m "feat: add precomputed spatial index"
git push
```

## Library Usage

### Building with Metadata

```rust
use evefrontier_lib::{load_starmap, SpatialIndex};
use evefrontier_lib::spatial::{DatasetMetadata, compute_dataset_checksum};
use std::path::Path;

let db_path = Path::new("data/static_data.db");
let starmap = load_starmap(db_path)?;

// Compute source metadata
let checksum = compute_dataset_checksum(db_path)?;
let metadata = DatasetMetadata {
    checksum,
    release_tag: Some("e6c3".to_string()),
    build_timestamp: chrono::Utc::now().timestamp(),
};

// Build and save
let index = SpatialIndex::build_with_metadata(&starmap, metadata);
index.save(Path::new("data/static_data.db.spatial.bin"))?;
```

### Verifying Freshness

```rust
use evefrontier_lib::spatial::{verify_freshness, FreshnessResult};
use std::path::Path;

let result = verify_freshness(
    Path::new("data/static_data.db.spatial.bin"),
    Path::new("data/static_data.db"),
);

match result {
    FreshnessResult::Fresh { .. } => println!("Index is up to date"),
    FreshnessResult::Stale { expected_checksum, actual_checksum, .. } => {
        eprintln!("Index mismatch: {} vs {}", expected_checksum, actual_checksum);
        std::process::exit(1);
    }
    _ => std::process::exit(1),
}
```

## File Format Changes

### v1 Format (legacy)
- No source metadata
- Will fail CI freshness check
- Upgrade by running `index-build --force`

### v2 Format (new)
- Includes source dataset checksum (SHA-256)
- Includes release tag (if available)
- Includes build timestamp
- Required for CI freshness verification

## Exit Codes Reference

| Code | Status | Action Required |
|------|--------|-----------------|
| 0 | Fresh | None |
| 1 | Stale | Run `index-build --force` |
| 2 | Missing | Run `index-build` |
| 3 | Legacy v1 | Run `index-build --force` |
| 4 | No Dataset | Download dataset first |
| 5 | Error | Check error message |
