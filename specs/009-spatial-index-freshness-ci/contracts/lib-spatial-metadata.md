# Library API Contract: Spatial Index Source Metadata

**Feature Branch**: `009-spatial-index-freshness-ci`  
**Date**: 2025-12-30

## API Specification

### Module: `evefrontier_lib::spatial`

#### New Types

```rust
/// Metadata about the source dataset used to build a spatial index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetMetadata {
    /// SHA-256 checksum of the source dataset file.
    pub checksum: [u8; 32],
    /// Release tag from the `.db.release` marker file.
    pub release_tag: Option<String>,
    /// Unix timestamp when the index was built.
    pub build_timestamp: i64,
}

/// Result of verifying spatial index freshness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FreshnessResult {
    Fresh { checksum: String, release_tag: Option<String> },
    Stale { expected_checksum: String, actual_checksum: String, ... },
    LegacyFormat { index_path: String, message: String },
    Missing { expected_path: String },
    DatasetMissing { expected_path: String },
    Error { message: String },
}
```

#### Modified Functions

##### `SpatialIndex::build`

**Current signature:**
```rust
pub fn build(starmap: &Starmap) -> Self
```

**New signature (backward compatible):**
```rust
pub fn build(starmap: &Starmap) -> Self

/// Build with source metadata for freshness tracking.
pub fn build_with_metadata(starmap: &Starmap, metadata: DatasetMetadata) -> Self
```

##### `SpatialIndex::save`

**Current signature:**
```rust
pub fn save(&self, path: &Path) -> Result<()>
```

**New signature (unchanged, but writes v2 format):**
```rust
pub fn save(&self, path: &Path) -> Result<()>
```

If `self.source_metadata` is `Some`, writes v2 format with metadata.
If `self.source_metadata` is `None`, writes v2 format with empty metadata section.

##### `SpatialIndex::load`

**Current signature:**
```rust
pub fn load(path: &Path) -> Result<Self>
```

**New behavior:**
- Reads both v1 and v2 format files
- For v1 files: `source_metadata()` returns `None`
- For v2 files: `source_metadata()` returns `Some(DatasetMetadata)` if present

#### New Functions

##### `SpatialIndex::source_metadata`

```rust
impl SpatialIndex {
    /// Returns the source dataset metadata embedded in this index.
    ///
    /// Returns `None` if:
    /// - Index was built without metadata (in-memory only)
    /// - Index was loaded from a v1 format file
    /// - Index was loaded from a v2 file without metadata section
    pub fn source_metadata(&self) -> Option<&DatasetMetadata> {
        self.metadata.as_ref()
    }
}
```

##### `compute_dataset_checksum`

```rust
/// Compute SHA-256 checksum of a dataset file.
///
/// Uses streaming reads to avoid loading the entire file into memory.
/// Suitable for files up to several GB.
///
/// # Errors
/// Returns error if file cannot be opened or read.
pub fn compute_dataset_checksum(path: &Path) -> Result<[u8; 32]>
```

##### `read_release_tag`

```rust
/// Read the release tag from a `.db.release` marker file.
///
/// # Arguments
/// * `db_path` - Path to the dataset `.db` file
///
/// # Returns
/// * `Some(tag)` if marker file exists and contains resolved tag
/// * `None` if marker file doesn't exist or is malformed
pub fn read_release_tag(db_path: &Path) -> Option<String>
```

##### `verify_freshness`

```rust
/// Verify that a spatial index is fresh (built from the current dataset).
///
/// # Arguments
/// * `index_path` - Path to the spatial index file
/// * `dataset_path` - Path to the dataset file
///
/// # Returns
/// `FreshnessResult` indicating whether the index matches the dataset.
///
/// # Example
/// ```rust
/// use evefrontier_lib::spatial::{verify_freshness, FreshnessResult};
///
/// let result = verify_freshness(
///     Path::new("data/static_data.db.spatial.bin"),
///     Path::new("data/static_data.db")
/// );
///
/// match result {
///     FreshnessResult::Fresh { .. } => println!("Index is up to date"),
///     FreshnessResult::Stale { .. } => println!("Index needs rebuild"),
///     _ => println!("Verification issue"),
/// }
/// ```
pub fn verify_freshness(index_path: &Path, dataset_path: &Path) -> FreshnessResult
```

### Usage Examples

#### Building Index with Metadata

```rust
use evefrontier_lib::{load_starmap, SpatialIndex, DatasetMetadata};
use evefrontier_lib::spatial::{compute_dataset_checksum, read_release_tag};
use std::time::{SystemTime, UNIX_EPOCH};

fn build_index_with_tracking(db_path: &Path) -> Result<()> {
    // Load starmap
    let starmap = load_starmap(db_path)?;
    
    // Compute source metadata
    let checksum = compute_dataset_checksum(db_path)?;
    let release_tag = read_release_tag(db_path);
    let build_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    
    let metadata = DatasetMetadata {
        checksum,
        release_tag,
        build_timestamp,
    };
    
    // Build and save index
    let index = SpatialIndex::build_with_metadata(&starmap, metadata);
    index.save(&db_path.with_extension("db.spatial.bin"))?;
    
    Ok(())
}
```

#### Verifying Freshness

```rust
use evefrontier_lib::spatial::{verify_freshness, FreshnessResult};

fn check_index(data_dir: &Path) -> bool {
    let db_path = data_dir.join("static_data.db");
    let index_path = data_dir.join("static_data.db.spatial.bin");
    
    match verify_freshness(&index_path, &db_path) {
        FreshnessResult::Fresh { checksum, release_tag } => {
            println!("✓ Index is fresh (tag: {:?})", release_tag);
            true
        }
        FreshnessResult::Stale { expected_checksum, actual_checksum, .. } => {
            println!("✗ Index is stale");
            println!("  Expected: {}", expected_checksum);
            println!("  Actual:   {}", actual_checksum);
            false
        }
        FreshnessResult::LegacyFormat { message, .. } => {
            println!("✗ Legacy format: {}", message);
            false
        }
        FreshnessResult::Missing { expected_path } => {
            println!("✗ Index not found: {}", expected_path);
            false
        }
        _ => false,
    }
}
```
