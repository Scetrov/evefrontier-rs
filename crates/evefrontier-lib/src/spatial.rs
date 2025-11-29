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

/// Current index format version.
const INDEX_VERSION: u8 = 1;

/// Flag: index includes min_external_temp data.
const FLAG_HAS_TEMPERATURE: u8 = 0x01;

/// Header size in bytes.
const HEADER_SIZE: usize = 16;

/// Checksum size in bytes (SHA-256).
const CHECKSUM_SIZE: usize = 32;

/// zstd compression level (balanced speed/ratio).
const COMPRESSION_LEVEL: i32 = 3;

/// KD-tree bucket size (kiddo default).
const BUCKET_SIZE: usize = 32;

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
        }
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
    pub fn save(&self, path: &Path) -> Result<()> {
        info!(
            path = %path.display(),
            nodes = self.nodes.len(),
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

        // Compute checksum
        let checksum = Sha256::digest(&compressed);

        // Build header
        let has_temp = self.nodes.iter().any(|n| n.min_external_temp.is_some());
        let flags = if has_temp { FLAG_HAS_TEMPERATURE } else { 0 };
        let node_count = self.nodes.len() as u32;

        let mut header = [0u8; HEADER_SIZE];
        header[0..4].copy_from_slice(INDEX_MAGIC);
        header[4] = INDEX_VERSION;
        header[5] = flags;
        header[6..10].copy_from_slice(&node_count.to_le_bytes());
        // bytes 10-15 reserved

        // Write file
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(&header)?;
        writer.write_all(&compressed)?;
        writer.write_all(&checksum)?;
        writer.flush()?;

        let file_size = HEADER_SIZE + compressed.len() + CHECKSUM_SIZE;
        info!(
            file_size = file_size,
            compressed_size = compressed.len(),
            "spatial index saved"
        );

        Ok(())
    }

    /// Load a spatial index from a file.
    ///
    /// Validates the header, decompresses the body, and verifies the checksum.
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
        if version != INDEX_VERSION {
            return Err(Error::SpatialIndexLoad {
                path: path.to_path_buf(),
                message: format!(
                    "unsupported version {} (expected {})",
                    version, INDEX_VERSION
                ),
            });
        }

        let _flags = header[5];
        let node_count = u32::from_le_bytes(header[6..10].try_into().unwrap());

        // Read compressed data (everything except header and checksum)
        let metadata = std::fs::metadata(path)?;
        let compressed_size = metadata.len() as usize - HEADER_SIZE - CHECKSUM_SIZE;

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
            "loaded spatial index"
        );

        Ok(Self {
            tree,
            nodes,
            temp_lookup,
            id_to_index,
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
        };

        let results = index.within_radius([0.0, 0.0, 0.0], 10.0);
        assert_eq!(results.len(), 2); // 1 and 2 within radius
        let ids: Vec<_> = results.iter().map(|(id, _)| *id).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
        assert!(!ids.contains(&3));
    }
}
