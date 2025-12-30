//! Precomputed KD-tree spatial index for efficient nearest-neighbour queries.
//!
//! This module implements ADR 0009: a static spatial index built at dataset preparation
//! time and serialized for fast cold-start loading. The index embeds per-system temperature
//! data to support temperature-aware filtering during spatial routing.
//!
//! # Overview
//!
//! The `SpatialIndex` provides O(log n) average-case nearest-neighbour and radius queries
//! using a KD-tree (k=3 for 3D Cartesian coordinates). Each indexed system stores:
//!
//! - System ID (i64)
//! - 3D coordinates (f32 for compactness)
//! - Minimum external temperature in Kelvin (Option<f32>)
//!
//! # Temperature-Aware Queries
//!
//! When routing, spatial jumps to systems above a temperature threshold should be
//! excluded. The index embeds `min_external_temp` per system so queries can filter
//! without additional database lookups:
//!
//! - Systems with `min_external_temp > threshold` are excluded
//! - Systems with `min_external_temp = None` pass (fail-open policy per ADR 0009)
//!
//! # Serialization Format
//!
//! ```text
//! Header (16 bytes):
//!   - Magic: b"EFSI" (4 bytes)
//!   - Version: u8 (1 byte)
//!   - Flags: u8 (1 byte) - bit 0: has_min_external_temp
//!   - Node count: u32 (4 bytes)
//!   - Reserved: 6 bytes
//!
//! Body:
//!   - postcard-serialized Vec<IndexNode>
//!   - zstd compressed
//!
//! Footer (32 bytes):
//!   - SHA-256 checksum of compressed body
//! ```
//!
//! # Example
//!
//! ```no_run
//! use evefrontier_lib::{load_starmap, SpatialIndex, NeighbourQuery};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let starmap = load_starmap(std::path::Path::new("static_data.db"))?;
//! let index = SpatialIndex::build(&starmap);
//!
//! // Query 10 nearest systems to a position, excluding those above 50K
//! let query = NeighbourQuery {
//!     k: 10,
//!     radius: Some(100.0),  // light-years
//!     max_temperature: Some(50.0),  // Kelvin
//! };
//! let point = [0.0, 0.0, 0.0];
//! let neighbors = index.nearest_filtered(point, &query);
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use kiddo::float::kdtree::KdTree;
use kiddo::SquaredEuclidean;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::db::{Starmap, SystemId, SystemPosition};
use crate::error::{Error, Result};

/// Magic bytes identifying a spatial index file.
const INDEX_MAGIC: &[u8; 4] = b"EFSI";

/// Current index format version (v1 without metadata).
const INDEX_VERSION: u8 = 1;

/// Index format version 2 (with source metadata).
pub const INDEX_VERSION_V2: u8 = 2;

/// Flag: index includes min_external_temp data.
const FLAG_HAS_TEMPERATURE: u8 = 0x01;

/// Flag: index includes source metadata section (v2 format).
pub const FLAG_HAS_METADATA: u8 = 0x02;

/// Header size in bytes.
const HEADER_SIZE: usize = 16;

/// Checksum size in bytes (SHA-256).
const CHECKSUM_SIZE: usize = 32;

/// zstd compression level (balanced speed/ratio).
const COMPRESSION_LEVEL: i32 = 3;

/// KD-tree bucket size (kiddo default).
const BUCKET_SIZE: usize = 32;

// =============================================================================
// Source Metadata Types (v2 format)
// =============================================================================

/// Metadata about the source dataset used to build a spatial index.
///
/// Embedded in the spatial index file (v2 format) to enable freshness verification.
/// The checksum provides cryptographic proof of the exact dataset version,
/// while the tag provides human-readable identification.
///
/// # Example
///
/// ```
/// use evefrontier_lib::spatial::DatasetMetadata;
///
/// let metadata = DatasetMetadata {
///     checksum: [0u8; 32],
///     release_tag: Some("e6c3".to_string()),
///     build_timestamp: 1735500000,
/// };
/// ```
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

/// Result of verifying spatial index freshness against the current dataset.
///
/// This enum represents all possible outcomes of comparing a spatial index's
/// embedded source metadata against the current dataset file.
///
/// # Serialization
///
/// Uses serde's tagged enum format with `status` as the discriminant:
/// - `{"status": "fresh", "checksum": "...", "release_tag": "..."}`
/// - `{"status": "stale", "expected_checksum": "...", ...}`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FreshnessResult {
    /// Index is fresh - source metadata matches current dataset.
    Fresh {
        /// The matching checksum (hex-encoded, 64 characters).
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

/// Structured output for the index-verify CLI command.
///
/// Supports both human-readable (default) and JSON (`--json` flag) formats.
/// Contains the verification result plus optional diagnostic information.
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

// =============================================================================
// Index Node and Query Types
// =============================================================================

/// Index node stored per system.
///
/// Uses f32 for coordinates to reduce serialized size (per ADR 0009 guidance).
/// Temperature is also f32 for consistency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexNode {
    /// System identifier.
    pub system_id: SystemId,
    /// 3D Cartesian coordinates in light-years.
    pub coords: [f32; 3],
    /// Minimum external temperature in Kelvin (computed at load time).
    pub min_external_temp: Option<f32>,
}

/// Query parameters for nearest-neighbour searches.
#[derive(Debug, Clone, Default)]
pub struct NeighbourQuery {
    /// Maximum number of results to return.
    pub k: usize,
    /// Optional radius constraint in light-years.
    pub radius: Option<f64>,
    /// Optional maximum temperature threshold in Kelvin.
    ///
    /// Systems with `min_external_temp > max_temperature` are excluded.
    /// Systems without temperature data pass (fail-open policy).
    pub max_temperature: Option<f64>,
}

impl NeighbourQuery {
    /// Create a simple k-nearest query without constraints.
    pub fn nearest(k: usize) -> Self {
        Self {
            k,
            radius: None,
            max_temperature: None,
        }
    }

    /// Create a query with radius constraint.
    pub fn within_radius(k: usize, radius: f64) -> Self {
        Self {
            k,
            radius: Some(radius),
            max_temperature: None,
        }
    }

    /// Create a query with temperature constraint.
    pub fn with_temperature(k: usize, max_temperature: f64) -> Self {
        Self {
            k,
            radius: None,
            max_temperature: Some(max_temperature),
        }
    }
}

// =============================================================================
// Freshness Verification Functions
// =============================================================================

/// Compute SHA-256 checksum of a dataset file using streaming reads.
///
/// Reads the file in 64KB chunks to avoid loading large databases into memory.
/// Returns the raw 32-byte checksum.
///
/// # Example
///
/// ```no_run
/// use evefrontier_lib::spatial::compute_dataset_checksum;
/// use std::path::Path;
///
/// let checksum = compute_dataset_checksum(Path::new("static_data.db"))?;
/// println!("Checksum: {:02x?}", checksum);
/// # Ok::<(), evefrontier_lib::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read.
pub fn compute_dataset_checksum(path: &Path) -> Result<[u8; 32]> {
    let file = File::open(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::DatasetNotFound {
                path: path.to_path_buf(),
            }
        } else {
            Error::Io(e)
        }
    })?;

    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];

    loop {
        let bytes_read = reader.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buf[..bytes_read]);
    }

    Ok(hasher.finalize().into())
}

/// Read the release tag from a `.db.release` marker file.
///
/// The marker file is created by the downloader and contains lines like:
/// ```text
/// requested=latest
/// resolved=e6c3
/// ```
///
/// Returns `Some(tag)` if the marker exists and contains a `resolved=` line,
/// `None` otherwise. Does not error on missing or malformed markers.
///
/// # Example
///
/// ```no_run
/// use evefrontier_lib::spatial::read_release_tag;
/// use std::path::Path;
///
/// let tag = read_release_tag(Path::new("static_data.db"));
/// println!("Release tag: {:?}", tag); // Some("e6c3") or None
/// ```
pub fn read_release_tag(db_path: &Path) -> Option<String> {
    let marker_path = db_path.with_extension("db.release");

    let content = std::fs::read_to_string(&marker_path).ok()?;

    for line in content.lines() {
        if let Some(tag) = line.strip_prefix("resolved=") {
            let trimmed = tag.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

/// Verify that a spatial index matches its source dataset.
///
/// This is the core function used by both the CLI `index-verify` command and CI jobs
/// to ensure the spatial index is in sync with the dataset.
///
/// # Returns
///
/// - `Fresh` - Index exists, is v2 format, and checksums match
/// - `Stale` - Index exists and is v2, but checksums differ (dataset changed)
/// - `LegacyFormat` - Index exists but is v1 format (no embedded checksum)
/// - `Missing` - Index file does not exist
/// - `DatasetMissing` - Dataset file does not exist
/// - `Error` - Error occurred during verification
///
/// # Example
///
/// ```no_run
/// use evefrontier_lib::spatial::{verify_freshness, FreshnessResult};
/// use std::path::Path;
///
/// let index_path = Path::new("static_data.db.spatial.bin");
/// let db_path = Path::new("static_data.db");
///
/// match verify_freshness(index_path, db_path) {
///     FreshnessResult::Fresh { .. } => println!("Index is fresh!"),
///     FreshnessResult::Stale { .. } => println!("Index needs rebuild"),
///     FreshnessResult::LegacyFormat { .. } => println!("Upgrade to v2 format"),
///     other => println!("Issue: {:?}", other),
/// }
/// ```
pub fn verify_freshness(index_path: &Path, db_path: &Path) -> FreshnessResult {
    // Check dataset exists
    if !db_path.exists() {
        return FreshnessResult::DatasetMissing {
            expected_path: db_path.display().to_string(),
        };
    }

    // Check index exists
    if !index_path.exists() {
        return FreshnessResult::Missing {
            expected_path: index_path.display().to_string(),
        };
    }

    // Try to load the index and get metadata
    let index = match SpatialIndex::load(index_path) {
        Ok(idx) => idx,
        Err(e) => {
            return FreshnessResult::Error {
                message: format!("failed to load index: {}", e),
            };
        }
    };

    // Check if index has metadata (v2 format)
    let index_metadata = match index.source_metadata() {
        Some(meta) => meta,
        None => {
            return FreshnessResult::LegacyFormat {
                index_path: index_path.display().to_string(),
                message: "Index is v1 format without embedded metadata. Rebuild with --metadata to enable freshness verification.".to_string(),
            };
        }
    };

    // Compute current dataset checksum
    let actual_checksum = match compute_dataset_checksum(db_path) {
        Ok(cs) => cs,
        Err(e) => {
            return FreshnessResult::Error {
                message: format!("failed to compute dataset checksum: {}", e),
            };
        }
    };

    // Read current release tag
    let actual_tag = read_release_tag(db_path);

    // Format checksums as hex for output
    let expected_checksum_hex = hex_encode(&index_metadata.checksum);
    let actual_checksum_hex = hex_encode(&actual_checksum);

    // Compare checksums
    if index_metadata.checksum == actual_checksum {
        FreshnessResult::Fresh {
            checksum: actual_checksum_hex,
            release_tag: actual_tag,
        }
    } else {
        FreshnessResult::Stale {
            expected_checksum: expected_checksum_hex,
            actual_checksum: actual_checksum_hex,
            expected_tag: index_metadata.release_tag.clone(),
            actual_tag,
        }
    }
}

/// Convert bytes to lowercase hex string.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// =============================================================================
// Spatial Index Implementation
// =============================================================================

/// Precomputed spatial index for efficient nearest-neighbour queries.
///
/// The index is built from a `Starmap` and can be serialized to disk for fast
/// loading at application startup (especially important for Lambda cold-starts).
pub struct SpatialIndex {
    /// KD-tree for spatial queries. Uses usize as item type (index into nodes vec).
    tree: KdTree<f32, usize, 3, BUCKET_SIZE, u32>,
    /// Indexed nodes containing system data.
    nodes: Vec<IndexNode>,
    /// Fast lookup from system ID to temperature.
    temp_lookup: HashMap<SystemId, Option<f32>>,
    /// Fast lookup from system ID to node index.
    id_to_index: HashMap<SystemId, usize>,
    /// Source dataset metadata (v2 format only).
    ///
    /// Present when index was built with `build_with_metadata()` or loaded from
    /// a v2 format file. None for v1 format files or indexes built without metadata.
    metadata: Option<DatasetMetadata>,
}

impl SpatialIndex {
    /// Build a spatial index from a starmap.
    ///
    /// Only systems with valid 3D positions are indexed. The `min_external_temp`
    /// from each system's metadata is embedded in the index for temperature-aware
    /// queries.
    pub fn build(starmap: &Starmap) -> Self {
        let mut nodes = Vec::new();
        let mut temp_lookup = HashMap::new();
        let mut id_to_index = HashMap::new();

        for system in starmap.systems.values() {
            let Some(position) = system.position else {
                continue;
            };

            let coords = position_to_coords(&position);
            let min_external_temp = system.metadata.min_external_temp.map(|t| t as f32);

            let node = IndexNode {
                system_id: system.id,
                coords,
                min_external_temp,
            };

            let index = nodes.len();
            id_to_index.insert(system.id, index);
            temp_lookup.insert(system.id, min_external_temp);
            nodes.push(node);
        }

        // Build the KD-tree
        let mut tree: KdTree<f32, usize, 3, BUCKET_SIZE, u32> = KdTree::new();
        for (index, node) in nodes.iter().enumerate() {
            tree.add(&node.coords, index);
        }

        info!(
            node_count = nodes.len(),
            systems_with_temp = temp_lookup.values().filter(|t| t.is_some()).count(),
            "built spatial index"
        );

        Self {
            tree,
            nodes,
            temp_lookup,
            id_to_index,
            metadata: None, // No metadata when built without build_with_metadata()
        }
    }

    /// Build a spatial index from a starmap with source metadata.
    ///
    /// The metadata is embedded in the index and written to disk when `save()` is called.
    /// This enables freshness verification via `verify_freshness()`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use evefrontier_lib::{load_starmap, SpatialIndex};
    /// use evefrontier_lib::spatial::{DatasetMetadata, compute_dataset_checksum, read_release_tag};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let db_path = Path::new("static_data.db");
    /// let starmap = load_starmap(db_path)?;
    ///
    /// let checksum = compute_dataset_checksum(db_path)?;
    /// let metadata = DatasetMetadata {
    ///     checksum,
    ///     release_tag: read_release_tag(db_path),
    ///     build_timestamp: std::time::SystemTime::now()
    ///         .duration_since(std::time::UNIX_EPOCH)
    ///         .unwrap()
    ///         .as_secs() as i64,
    /// };
    ///
    /// let index = SpatialIndex::build_with_metadata(&starmap, metadata);
    /// index.save(Path::new("static_data.db.spatial.bin"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build_with_metadata(starmap: &Starmap, metadata: DatasetMetadata) -> Self {
        let mut index = Self::build(starmap);
        index.metadata = Some(metadata);
        index
    }

    /// Returns the source dataset metadata embedded in this index.
    ///
    /// Returns `None` if:
    /// - Index was built without metadata (using `build()` instead of `build_with_metadata()`)
    /// - Index was loaded from a v1 format file
    /// - Index was loaded from a v2 file without metadata section
    pub fn source_metadata(&self) -> Option<&DatasetMetadata> {
        self.metadata.as_ref()
    }

    /// Number of indexed systems.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns true if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get the temperature for a system if indexed.
    pub fn temperature(&self, system_id: SystemId) -> Option<Option<f32>> {
        self.temp_lookup.get(&system_id).copied()
    }

    /// Get the position of a system if indexed.
    pub fn position(&self, system_id: SystemId) -> Option<[f32; 3]> {
        self.id_to_index
            .get(&system_id)
            .map(|&idx| self.nodes[idx].coords)
    }

    /// Find k nearest neighbours to a point.
    ///
    /// Returns (SystemId, distance) pairs sorted by distance.
    pub fn nearest(&self, point: [f64; 3], k: usize) -> Vec<(SystemId, f64)> {
        if k == 0 || self.nodes.is_empty() {
            return Vec::new();
        }

        let query_point = [point[0] as f32, point[1] as f32, point[2] as f32];
        let results = self.tree.nearest_n::<SquaredEuclidean>(&query_point, k);

        results
            .into_iter()
            .map(|neighbor| {
                let node = &self.nodes[neighbor.item];
                let distance = (neighbor.distance as f64).sqrt();
                (node.system_id, distance)
            })
            .collect()
    }

    /// Find all systems within a radius of a point.
    ///
    /// Returns (SystemId, distance) pairs sorted by distance.
    pub fn within_radius(&self, point: [f64; 3], radius: f64) -> Vec<(SystemId, f64)> {
        if radius <= 0.0 || self.nodes.is_empty() {
            return Vec::new();
        }

        let query_point = [point[0] as f32, point[1] as f32, point[2] as f32];
        let squared_radius = (radius * radius) as f32;
        let results = self
            .tree
            .within::<SquaredEuclidean>(&query_point, squared_radius);

        let mut neighbors: Vec<(SystemId, f64)> = results
            .into_iter()
            .map(|neighbor| {
                let node = &self.nodes[neighbor.item];
                let distance = (neighbor.distance as f64).sqrt();
                (node.system_id, distance)
            })
            .collect();

        neighbors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        neighbors
    }

    /// Find nearest neighbours with filtering.
    ///
    /// Applies temperature and radius constraints, returning up to k results.
    /// Over-fetches by 2x to account for filtered results.
    ///
    /// # Temperature Filtering (per ADR 0009)
    ///
    /// - If `max_temperature` is set, exclude systems where `min_external_temp > threshold`
    /// - Systems with `None` temperature pass (fail-open policy)
    pub fn nearest_filtered(
        &self,
        point: [f64; 3],
        query: &NeighbourQuery,
    ) -> Vec<(SystemId, f64)> {
        if query.k == 0 || self.nodes.is_empty() {
            return Vec::new();
        }

        let query_point = [point[0] as f32, point[1] as f32, point[2] as f32];

        // Over-fetch to account for filtering
        let fetch_count = query.k.saturating_mul(2).max(query.k + 10);

        let candidates = self
            .tree
            .nearest_n::<SquaredEuclidean>(&query_point, fetch_count);

        let mut results = Vec::with_capacity(query.k);

        for neighbor in candidates {
            let node = &self.nodes[neighbor.item];
            let distance = (neighbor.distance as f64).sqrt();

            // Apply radius filter
            if let Some(max_radius) = query.radius {
                if distance > max_radius {
                    continue;
                }
            }

            // Apply temperature filter (fail-open: None temps pass)
            if let Some(max_temp) = query.max_temperature {
                if let Some(temp) = node.min_external_temp {
                    if (temp as f64) > max_temp {
                        continue;
                    }
                }
                // None temp = pass through (fail-open policy)
            }

            results.push((node.system_id, distance));

            if results.len() >= query.k {
                break;
            }
        }

        results
    }

    /// Find all systems within a radius, filtered by temperature.
    pub fn within_radius_filtered(
        &self,
        point: [f64; 3],
        radius: f64,
        max_temperature: Option<f64>,
    ) -> Vec<(SystemId, f64)> {
        if radius <= 0.0 || self.nodes.is_empty() {
            return Vec::new();
        }

        let query_point = [point[0] as f32, point[1] as f32, point[2] as f32];
        let squared_radius = (radius * radius) as f32;
        let candidates = self
            .tree
            .within::<SquaredEuclidean>(&query_point, squared_radius);

        let mut results: Vec<(SystemId, f64)> = candidates
            .into_iter()
            .filter_map(|neighbor| {
                let node = &self.nodes[neighbor.item];
                let distance = (neighbor.distance as f64).sqrt();

                // Apply temperature filter
                if let Some(max_temp) = max_temperature {
                    if let Some(temp) = node.min_external_temp {
                        if (temp as f64) > max_temp {
                            return None;
                        }
                    }
                }

                Some((node.system_id, distance))
            })
            .collect();

        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Serialize the index to a file.
    ///
    /// Uses postcard for compact binary encoding and zstd for compression.
    /// Writes a versioned header and SHA-256 checksum for integrity verification.
    ///
    /// If the index was built with `build_with_metadata()`, writes v2 format with
    /// embedded source metadata. Otherwise writes v1 format for backward compatibility.
    pub fn save(&self, path: &Path) -> Result<()> {
        let version = if self.metadata.is_some() {
            INDEX_VERSION_V2
        } else {
            INDEX_VERSION
        };

        info!(
            path = %path.display(),
            nodes = self.nodes.len(),
            version = version,
            "saving spatial index"
        );

        // Serialize nodes with postcard
        let serialized =
            postcard::to_allocvec(&self.nodes).map_err(|e| Error::SpatialIndexSerialize {
                message: format!("postcard serialization failed: {}", e),
            })?;

        // Compress with zstd
        let compressed =
            zstd::encode_all(serialized.as_slice(), COMPRESSION_LEVEL).map_err(|e| {
                Error::SpatialIndexSerialize {
                    message: format!("zstd compression failed: {}", e),
                }
            })?;

        // Build flags
        let has_temp = self.nodes.iter().any(|n| n.min_external_temp.is_some());
        let mut flags = if has_temp { FLAG_HAS_TEMPERATURE } else { 0 };
        if self.metadata.is_some() {
            flags |= FLAG_HAS_METADATA;
        }

        let node_count = self.nodes.len() as u32;

        // Build header
        let mut header = [0u8; HEADER_SIZE];
        header[0..4].copy_from_slice(INDEX_MAGIC);
        header[4] = version;
        header[5] = flags;
        header[6..10].copy_from_slice(&node_count.to_le_bytes());
        // bytes 10-15 reserved

        // Prepare metadata section if v2 format
        let metadata_section = if let Some(ref meta) = self.metadata {
            // Serialize metadata: checksum(32) + tag_len(u16) + tag_bytes + timestamp(i64)
            let tag_bytes = meta
                .release_tag
                .as_ref()
                .map(|s| s.as_bytes())
                .unwrap_or(&[]);
            let tag_len = tag_bytes.len() as u16;

            let mut section = Vec::with_capacity(32 + 2 + tag_bytes.len() + 8);
            section.extend_from_slice(&meta.checksum);
            section.extend_from_slice(&tag_len.to_le_bytes());
            section.extend_from_slice(tag_bytes);
            section.extend_from_slice(&meta.build_timestamp.to_le_bytes());
            section
        } else {
            Vec::new()
        };

        // Compute checksum over compressed data only (consistent with v1)
        let checksum = Sha256::digest(&compressed);

        // Write file
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(&header)?;
        if !metadata_section.is_empty() {
            writer.write_all(&metadata_section)?;
        }
        writer.write_all(&compressed)?;
        writer.write_all(&checksum)?;
        writer.flush()?;

        let file_size = HEADER_SIZE + metadata_section.len() + compressed.len() + CHECKSUM_SIZE;
        info!(
            file_size = file_size,
            compressed_size = compressed.len(),
            version = version,
            has_metadata = self.metadata.is_some(),
            "spatial index saved"
        );

        Ok(())
    }

    /// Load a spatial index from a file.
    ///
    /// Validates the header, decompresses the body, and verifies the checksum.
    /// Supports both v1 format (no metadata) and v2 format (with embedded metadata).
    pub fn load(path: &Path) -> Result<Self> {
        debug!(path = %path.display(), "loading spatial index");

        let file = File::open(path).map_err(|e| Error::SpatialIndexLoad {
            path: path.to_path_buf(),
            message: format!("failed to open file: {}", e),
        })?;
        let mut reader = BufReader::new(file);

        // Read and validate header
        let mut header = [0u8; HEADER_SIZE];
        reader
            .read_exact(&mut header)
            .map_err(|e| Error::SpatialIndexLoad {
                path: path.to_path_buf(),
                message: format!("failed to read header: {}", e),
            })?;

        if &header[0..4] != INDEX_MAGIC {
            return Err(Error::SpatialIndexLoad {
                path: path.to_path_buf(),
                message: "invalid magic bytes".to_string(),
            });
        }

        let version = header[4];
        if version != INDEX_VERSION && version != INDEX_VERSION_V2 {
            return Err(Error::SpatialIndexLoad {
                path: path.to_path_buf(),
                message: format!(
                    "unsupported version {} (expected {} or {})",
                    version, INDEX_VERSION, INDEX_VERSION_V2
                ),
            });
        }

        let flags = header[5];
        let has_metadata = (flags & FLAG_HAS_METADATA) != 0;
        let node_count = u32::from_le_bytes(header[6..10].try_into().unwrap());

        debug!(
            version = version,
            flags = flags,
            has_metadata = has_metadata,
            node_count = node_count,
            "parsed spatial index header"
        );

        // Read metadata section if v2 format with metadata flag
        let metadata = if version == INDEX_VERSION_V2 && has_metadata {
            // Read checksum (32 bytes)
            let mut checksum = [0u8; 32];
            reader
                .read_exact(&mut checksum)
                .map_err(|e| Error::SpatialIndexLoad {
                    path: path.to_path_buf(),
                    message: format!("failed to read metadata checksum: {}", e),
                })?;

            // Read tag length (u16)
            let mut tag_len_bytes = [0u8; 2];
            reader
                .read_exact(&mut tag_len_bytes)
                .map_err(|e| Error::SpatialIndexLoad {
                    path: path.to_path_buf(),
                    message: format!("failed to read metadata tag length: {}", e),
                })?;
            let tag_len = u16::from_le_bytes(tag_len_bytes) as usize;

            // Read tag bytes
            let release_tag = if tag_len > 0 {
                let mut tag_bytes = vec![0u8; tag_len];
                reader
                    .read_exact(&mut tag_bytes)
                    .map_err(|e| Error::SpatialIndexLoad {
                        path: path.to_path_buf(),
                        message: format!("failed to read metadata tag: {}", e),
                    })?;
                Some(String::from_utf8_lossy(&tag_bytes).into_owned())
            } else {
                None
            };

            // Read timestamp (i64)
            let mut timestamp_bytes = [0u8; 8];
            reader
                .read_exact(&mut timestamp_bytes)
                .map_err(|e| Error::SpatialIndexLoad {
                    path: path.to_path_buf(),
                    message: format!("failed to read metadata timestamp: {}", e),
                })?;
            let build_timestamp = i64::from_le_bytes(timestamp_bytes);

            Some(DatasetMetadata {
                checksum,
                release_tag,
                build_timestamp,
            })
        } else {
            None
        };

        // Calculate compressed data size
        let file_metadata = std::fs::metadata(path)?;
        let metadata_section_size = if metadata.is_some() {
            let tag_len = metadata
                .as_ref()
                .and_then(|m| m.release_tag.as_ref())
                .map(|s| s.len())
                .unwrap_or(0);
            32 + 2 + tag_len + 8 // checksum + tag_len + tag_bytes + timestamp
        } else {
            0
        };
        let compressed_size =
            file_metadata.len() as usize - HEADER_SIZE - metadata_section_size - CHECKSUM_SIZE;

        let mut compressed = vec![0u8; compressed_size];
        reader
            .read_exact(&mut compressed)
            .map_err(|e| Error::SpatialIndexLoad {
                path: path.to_path_buf(),
                message: format!("failed to read compressed data: {}", e),
            })?;

        // Read and verify checksum
        let mut stored_checksum = [0u8; CHECKSUM_SIZE];
        reader
            .read_exact(&mut stored_checksum)
            .map_err(|e| Error::SpatialIndexLoad {
                path: path.to_path_buf(),
                message: format!("failed to read checksum: {}", e),
            })?;

        let computed_checksum = Sha256::digest(&compressed);
        if computed_checksum.as_slice() != stored_checksum {
            return Err(Error::SpatialIndexLoad {
                path: path.to_path_buf(),
                message: "checksum mismatch - file may be corrupted".to_string(),
            });
        }

        // Decompress
        let decompressed =
            zstd::decode_all(compressed.as_slice()).map_err(|e| Error::SpatialIndexLoad {
                path: path.to_path_buf(),
                message: format!("zstd decompression failed: {}", e),
            })?;

        // Deserialize nodes
        let nodes: Vec<IndexNode> =
            postcard::from_bytes(&decompressed).map_err(|e| Error::SpatialIndexLoad {
                path: path.to_path_buf(),
                message: format!("postcard deserialization failed: {}", e),
            })?;

        if nodes.len() != node_count as usize {
            warn!(
                expected = node_count,
                actual = nodes.len(),
                "node count mismatch in spatial index"
            );
        }

        // Rebuild tree and lookups
        let mut tree: KdTree<f32, usize, 3, BUCKET_SIZE, u32> = KdTree::new();
        let mut temp_lookup = HashMap::new();
        let mut id_to_index = HashMap::new();

        for (index, node) in nodes.iter().enumerate() {
            tree.add(&node.coords, index);
            temp_lookup.insert(node.system_id, node.min_external_temp);
            id_to_index.insert(node.system_id, index);
        }

        info!(
            node_count = nodes.len(),
            systems_with_temp = temp_lookup.values().filter(|t| t.is_some()).count(),
            version = version,
            has_metadata = metadata.is_some(),
            "loaded spatial index"
        );

        Ok(Self {
            tree,
            nodes,
            temp_lookup,
            id_to_index,
            metadata,
        })
    }

    /// Load a spatial index from a byte slice.
    ///
    /// This is useful for loading from bundled data (e.g., `include_bytes!` in Lambda).
    /// The byte slice must contain the complete index with header, compressed data,
    /// and checksum.
    ///
    /// # Example
    ///
    /// ```text
    /// static INDEX_BYTES: &[u8] = include_bytes!("../data/spatial.bin");
    /// let index = SpatialIndex::load_from_bytes(INDEX_BYTES)?;
    /// ```
    pub fn load_from_bytes(bytes: &[u8]) -> Result<Self> {
        use std::io::Cursor;
        Self::load_from_reader(Cursor::new(bytes))
    }

    /// Load a spatial index from any `Read + Seek` source.
    ///
    /// This is the underlying implementation shared by `load` (file) and
    /// `load_from_bytes` (bundled data). Supports both v1 and v2 formats.
    pub fn load_from_reader<R: std::io::Read>(mut reader: R) -> Result<Self> {
        // Read and validate header
        let mut header = [0u8; HEADER_SIZE];
        reader
            .read_exact(&mut header)
            .map_err(|e| Error::SpatialIndexDeserialize {
                message: format!("failed to read header: {}", e),
            })?;

        if &header[0..4] != INDEX_MAGIC {
            return Err(Error::SpatialIndexDeserialize {
                message: "invalid magic bytes".to_string(),
            });
        }

        let version = header[4];
        if version != INDEX_VERSION && version != INDEX_VERSION_V2 {
            return Err(Error::SpatialIndexDeserialize {
                message: format!(
                    "unsupported version {} (expected {} or {})",
                    version, INDEX_VERSION, INDEX_VERSION_V2
                ),
            });
        }

        let flags = header[5];
        let has_metadata = (flags & FLAG_HAS_METADATA) != 0;
        let node_count = u32::from_le_bytes(header[6..10].try_into().unwrap());

        debug!(
            version = version,
            flags = flags,
            has_metadata = has_metadata,
            node_count = node_count,
            "parsed spatial index header from bytes"
        );

        // Read metadata section if v2 format with metadata flag
        let metadata = if version == INDEX_VERSION_V2 && has_metadata {
            // Read checksum (32 bytes)
            let mut checksum = [0u8; 32];
            reader
                .read_exact(&mut checksum)
                .map_err(|e| Error::SpatialIndexDeserialize {
                    message: format!("failed to read metadata checksum: {}", e),
                })?;

            // Read tag length (u16)
            let mut tag_len_bytes = [0u8; 2];
            reader
                .read_exact(&mut tag_len_bytes)
                .map_err(|e| Error::SpatialIndexDeserialize {
                    message: format!("failed to read metadata tag length: {}", e),
                })?;
            let tag_len = u16::from_le_bytes(tag_len_bytes) as usize;

            // Read tag bytes
            let release_tag = if tag_len > 0 {
                let mut tag_bytes = vec![0u8; tag_len];
                reader
                    .read_exact(&mut tag_bytes)
                    .map_err(|e| Error::SpatialIndexDeserialize {
                        message: format!("failed to read metadata tag: {}", e),
                    })?;
                Some(String::from_utf8_lossy(&tag_bytes).into_owned())
            } else {
                None
            };

            // Read timestamp (i64)
            let mut timestamp_bytes = [0u8; 8];
            reader.read_exact(&mut timestamp_bytes).map_err(|e| {
                Error::SpatialIndexDeserialize {
                    message: format!("failed to read metadata timestamp: {}", e),
                }
            })?;
            let build_timestamp = i64::from_le_bytes(timestamp_bytes);

            Some(DatasetMetadata {
                checksum,
                release_tag,
                build_timestamp,
            })
        } else {
            None
        };

        // Read remaining data (compressed + checksum)
        let mut remaining = Vec::new();
        reader
            .read_to_end(&mut remaining)
            .map_err(|e| Error::SpatialIndexDeserialize {
                message: format!("failed to read data: {}", e),
            })?;

        if remaining.len() < CHECKSUM_SIZE {
            return Err(Error::SpatialIndexDeserialize {
                message: "data too short for checksum".to_string(),
            });
        }

        let checksum_start = remaining.len() - CHECKSUM_SIZE;
        let compressed = &remaining[..checksum_start];
        let stored_checksum = &remaining[checksum_start..];

        // Verify checksum
        let computed_checksum = Sha256::digest(compressed);
        if computed_checksum.as_slice() != stored_checksum {
            return Err(Error::SpatialIndexDeserialize {
                message: "checksum mismatch - data may be corrupted".to_string(),
            });
        }

        // Decompress
        let decompressed =
            zstd::decode_all(compressed).map_err(|e| Error::SpatialIndexDeserialize {
                message: format!("zstd decompression failed: {}", e),
            })?;

        // Deserialize nodes
        let nodes: Vec<IndexNode> =
            postcard::from_bytes(&decompressed).map_err(|e| Error::SpatialIndexDeserialize {
                message: format!("postcard deserialization failed: {}", e),
            })?;

        if nodes.len() != node_count as usize {
            warn!(
                expected = node_count,
                actual = nodes.len(),
                "node count mismatch in spatial index"
            );
        }

        // Rebuild tree and lookups
        let mut tree: KdTree<f32, usize, 3, BUCKET_SIZE, u32> = KdTree::new();
        let mut temp_lookup = HashMap::new();
        let mut id_to_index = HashMap::new();

        for (index, node) in nodes.iter().enumerate() {
            tree.add(&node.coords, index);
            temp_lookup.insert(node.system_id, node.min_external_temp);
            id_to_index.insert(node.system_id, index);
        }

        info!(
            node_count = nodes.len(),
            systems_with_temp = temp_lookup.values().filter(|t| t.is_some()).count(),
            version = version,
            has_metadata = metadata.is_some(),
            "loaded spatial index from bytes"
        );

        Ok(Self {
            tree,
            nodes,
            temp_lookup,
            id_to_index,
            metadata,
        })
    }
}

impl std::fmt::Debug for SpatialIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpatialIndex")
            .field("node_count", &self.nodes.len())
            .field(
                "systems_with_temp",
                &self.temp_lookup.values().filter(|t| t.is_some()).count(),
            )
            .finish()
    }
}

/// Derive the spatial index path from a database path.
///
/// The index file is stored alongside the database with a `.spatial.bin` extension.
/// For example, `static_data.db` -> `static_data.db.spatial.bin`.
pub fn spatial_index_path(db_path: &Path) -> std::path::PathBuf {
    let mut path = db_path.as_os_str().to_owned();
    path.push(".spatial.bin");
    std::path::PathBuf::from(path)
}

/// Attempt to load a spatial index for a database, returning None if not found.
pub fn try_load_spatial_index(db_path: &Path) -> Option<SpatialIndex> {
    let index_path = spatial_index_path(db_path);
    if !index_path.exists() {
        return None;
    }

    match SpatialIndex::load(&index_path) {
        Ok(index) => Some(index),
        Err(e) => {
            warn!(
                path = %index_path.display(),
                error = %e,
                "failed to load spatial index, will rebuild"
            );
            None
        }
    }
}

fn position_to_coords(pos: &SystemPosition) -> [f32; 3] {
    [pos.x as f32, pos.y as f32, pos.z as f32]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_node(id: i64, x: f32, y: f32, z: f32, temp: Option<f32>) -> IndexNode {
        IndexNode {
            system_id: id,
            coords: [x, y, z],
            min_external_temp: temp,
        }
    }

    #[test]
    fn test_build_empty_starmap() {
        let starmap = Starmap::default();
        let index = SpatialIndex::build(&starmap);
        assert!(index.is_empty());
    }

    #[test]
    fn test_nearest_basic() {
        // Create a simple index with 3 nodes
        let nodes = vec![
            test_node(1, 0.0, 0.0, 0.0, Some(10.0)),
            test_node(2, 1.0, 0.0, 0.0, Some(20.0)),
            test_node(3, 2.0, 0.0, 0.0, Some(30.0)),
        ];

        let mut tree: KdTree<f32, usize, 3, BUCKET_SIZE, u32> = KdTree::new();
        let mut temp_lookup = HashMap::new();
        let mut id_to_index = HashMap::new();

        for (index, node) in nodes.iter().enumerate() {
            tree.add(&node.coords, index);
            temp_lookup.insert(node.system_id, node.min_external_temp);
            id_to_index.insert(node.system_id, index);
        }

        let index = SpatialIndex {
            tree,
            nodes,
            temp_lookup,
            id_to_index,
            metadata: None,
        };

        // Query from origin
        let results = index.nearest([0.0, 0.0, 0.0], 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 1); // closest
        assert_eq!(results[1].0, 2); // second closest
    }

    #[test]
    fn test_temperature_filtering() {
        let nodes = vec![
            test_node(1, 0.0, 0.0, 0.0, Some(10.0)),
            test_node(2, 1.0, 0.0, 0.0, Some(50.0)), // Hot system
            test_node(3, 2.0, 0.0, 0.0, None),       // No temp data
            test_node(4, 3.0, 0.0, 0.0, Some(20.0)),
        ];

        let mut tree: KdTree<f32, usize, 3, BUCKET_SIZE, u32> = KdTree::new();
        let mut temp_lookup = HashMap::new();
        let mut id_to_index = HashMap::new();

        for (index, node) in nodes.iter().enumerate() {
            tree.add(&node.coords, index);
            temp_lookup.insert(node.system_id, node.min_external_temp);
            id_to_index.insert(node.system_id, index);
        }

        let index = SpatialIndex {
            tree,
            nodes,
            temp_lookup,
            id_to_index,
            metadata: None,
        };

        // Query with max_temp = 30K (should exclude system 2)
        let query = NeighbourQuery {
            k: 10,
            radius: None,
            max_temperature: Some(30.0),
        };

        let results = index.nearest_filtered([0.0, 0.0, 0.0], &query);

        // Should include 1 (10K), 3 (None - passes), 4 (20K)
        // Should exclude 2 (50K)
        assert_eq!(results.len(), 3);
        let ids: Vec<_> = results.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&3)); // None temp passes (fail-open)
        assert!(ids.contains(&4));
        assert!(!ids.contains(&2)); // Excluded - too hot
    }

    #[test]
    fn test_radius_filtering() {
        let nodes = vec![
            test_node(1, 0.0, 0.0, 0.0, None),
            test_node(2, 5.0, 0.0, 0.0, None),
            test_node(3, 15.0, 0.0, 0.0, None), // Outside radius
        ];

        let mut tree: KdTree<f32, usize, 3, BUCKET_SIZE, u32> = KdTree::new();
        let mut temp_lookup = HashMap::new();
        let mut id_to_index = HashMap::new();

        for (index, node) in nodes.iter().enumerate() {
            tree.add(&node.coords, index);
            temp_lookup.insert(node.system_id, node.min_external_temp);
            id_to_index.insert(node.system_id, index);
        }

        let index = SpatialIndex {
            tree,
            nodes,
            temp_lookup,
            id_to_index,
            metadata: None,
        };

        let results = index.within_radius([0.0, 0.0, 0.0], 10.0);
        assert_eq!(results.len(), 2); // 1 and 2 within radius
        let ids: Vec<_> = results.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(!ids.contains(&3));
    }
}
