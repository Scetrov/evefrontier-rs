# CLI Contract: index-verify Command

**Feature Branch**: `009-spatial-index-freshness-ci`  
**Date**: 2025-12-30

## Command Specification

### Synopsis

```
evefrontier-cli index-verify [OPTIONS]
```

### Description

Verify that the spatial index artifact is fresh (built from the current dataset version). This
command compares the source metadata embedded in the spatial index file against the current
dataset's checksum and release tag.

### Options

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--data-dir` | `-d` | PATH | auto | Path to dataset directory |
| `--json` | | FLAG | false | Output in JSON format |
| `--quiet` | `-q` | FLAG | false | Only output on failure |
| `--strict` | | FLAG | false | Require release tag match (not just checksum) |

### Exit Codes

| Code | Status | Description |
|------|--------|-------------|
| 0 | SUCCESS | Index is fresh (matches dataset) |
| 1 | STALE | Index doesn't match dataset |
| 2 | MISSING | Spatial index file not found |
| 3 | FORMAT_ERROR | Legacy v1 format or corrupt file |
| 4 | DATASET_MISSING | Dataset file not found |
| 5 | ERROR | Unexpected error during verification |

### Output Formats

#### Human-Readable (default)

**Success case:**
```
✓ Spatial index is fresh
  Dataset:  e6c3 (a1b2c3d4...)
  Index:    built 2025-12-30 14:32:00 UTC
```

**Stale case:**
```
✗ Spatial index is STALE
  Dataset checksum:  a1b2c3d4e5f6...
  Index source:      7890abcdef12...
  
  Run 'evefrontier-cli index-build' to regenerate
```

**Legacy format case:**
```
✗ Spatial index uses legacy format (v1)
  Index file: data/static_data.db.spatial.bin
  
  Run 'evefrontier-cli index-build --force' to upgrade to v2
```

**Missing case:**
```
✗ Spatial index not found
  Expected: data/static_data.db.spatial.bin
  
  Run 'evefrontier-cli index-build' to create
```

#### JSON (`--json`)

**Success case:**
```json
{
  "result": {
    "status": "fresh",
    "checksum": "a1b2c3d4e5f6...",
    "release_tag": "e6c3"
  },
  "is_fresh": true,
  "recommended_action": null,
  "diagnostics": {
    "dataset_path": "data/static_data.db",
    "index_path": "data/static_data.db.spatial.bin",
    "dataset_size": 52428800,
    "index_size": 1048576,
    "index_version": 2,
    "verification_time_ms": 127
  }
}
```

**Stale case:**
```json
{
  "result": {
    "status": "stale",
    "expected_checksum": "a1b2c3d4e5f6...",
    "actual_checksum": "7890abcdef12...",
    "expected_tag": "e6c3",
    "actual_tag": "e6c2"
  },
  "is_fresh": false,
  "recommended_action": "evefrontier-cli index-build",
  "diagnostics": {
    "dataset_path": "data/static_data.db",
    "index_path": "data/static_data.db.spatial.bin",
    "dataset_size": 52428800,
    "index_size": 1048576,
    "index_version": 2,
    "verification_time_ms": 127
  }
}
```

### Output Conventions

- **Checksums**: All checksum values are lowercase hex-encoded (64 characters for SHA-256)
- **Timestamps**: ISO 8601 format in UTC (human output) or Unix epoch seconds (JSON `build_timestamp`)
- **Paths**: Absolute paths in JSON output, relative paths in human output when shorter

### Examples

```bash
# Basic verification (uses auto-detected data directory)
evefrontier-cli index-verify

# Verify with explicit data directory
evefrontier-cli index-verify --data-dir ./data

# CI-friendly JSON output
evefrontier-cli index-verify --json

# Quiet mode for scripts (only output on failure)
evefrontier-cli index-verify --quiet || echo "Index is stale!"

# Strict mode (require release tag match)
evefrontier-cli index-verify --strict
```

### CI Integration Example

```yaml
# .github/workflows/ci.yml
spatial-index-freshness:
  name: Spatial Index Freshness
  runs-on: ubuntu-latest
  needs: [test]
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Verify spatial index freshness
      run: |
        cargo run -p evefrontier-cli -- index-verify --data-dir data/ --json > verify-result.json
        cat verify-result.json
        cargo run -p evefrontier-cli -- index-verify --data-dir data/
```
