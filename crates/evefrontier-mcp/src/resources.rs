//! MCP resource implementations for dataset metadata
//!
//! This module defines the three resources exposed by the MCP server:
//! - evefrontier://dataset/info: Dataset metadata and schema
//! - evefrontier://algorithms: Available routing algorithms
//! - evefrontier://spatial-index/status: Spatial index build status

use crate::server::McpServerState;
use crate::Result;
use serde::Serialize;
use serde_json::json;

/// Dataset information resource
///
/// Returns metadata about the loaded EVE Frontier dataset including
/// system count, jump count, schema version, and build timestamp.
pub struct DatasetInfoResource;

impl DatasetInfoResource {
    /// Handle a dataset info resource read
    pub async fn read(state: &McpServerState) -> Result<String> {
        let info = state.dataset_info();
        serde_json::to_string(&info).map_err(|e| crate::Error::internal(e.to_string()))
    }
}

/// Algorithms resource
///
/// Returns a list of available routing algorithms and their
/// constraints/capabilities.
pub struct AlgorithmsResource;

impl AlgorithmsResource {
    /// Handle an algorithms resource read
    pub async fn read() -> Result<String> {
        #[derive(Serialize)]
        struct Algorithm<'a> {
            name: &'a str,
            description: &'a str,
            constraints: &'a [&'a str],
        }

        let algorithms = vec![
            Algorithm {
                name: "bfs",
                description: "Breadth-first search for unweighted gate routes",
                constraints: &["gate_only", "no_max_jump", "fast"],
            },
            Algorithm {
                name: "dijkstra",
                description: "Weighted routing supporting gate and spatial edges",
                constraints: &[
                    "supports_max_jump",
                    "supports_temperature",
                    "gate_or_spatial",
                ],
            },
            Algorithm {
                name: "a-star",
                description: "Heuristic-guided routing prioritizing shortest spatial distance",
                constraints: &[
                    "supports_max_jump",
                    "supports_temperature",
                    "heuristic_spatial",
                ],
            },
        ];

        let payload = json!({
            "algorithms": algorithms,
            "default": "a-star",
        });

        serde_json::to_string(&payload).map_err(|e| crate::Error::internal(e.to_string()))
    }
}

/// Spatial index status resource
///
/// Returns information about the spatial index including its version,
/// build timestamp, and whether it's currently loaded.
pub struct SpatialIndexStatusResource;

impl SpatialIndexStatusResource {
    /// Handle a spatial index status resource read
    pub async fn read(state: &McpServerState) -> Result<String> {
        let index_path = format!("{}.spatial.bin", state.database_path.display());
        let payload = json!({
            "available": state.spatial_index_available,
            "index_path": index_path,
            "dataset_path": state.database_path,
            "schema_version": state.schema_version,
            "initialized_at": state.initialized_at.to_rfc3339(),
        });

        serde_json::to_string(&payload).map_err(|e| crate::Error::internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn test_resource_stubs_defined() {
        // Verify resource structures are properly defined
        let _info = DatasetInfoResource;
        let _algorithms = AlgorithmsResource;
        let _status = SpatialIndexStatusResource;
    }

    fn test_state() -> McpServerState {
        McpServerState {
            database_path: PathBuf::from("/tmp/static_data.db"),
            initialized_at: Utc::now(),
            system_count: 8,
            gate_count: 12,
            schema_version: "e6c3".to_string(),
            spatial_index_available: true,
        }
    }

    #[tokio::test]
    async fn test_dataset_info_resource() {
        let state = test_state();
        let json = DatasetInfoResource::read(&state).await.unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["system_count"], 8);
        assert_eq!(value["gate_count"], 12);
        assert_eq!(value["schema_version"], "e6c3");
        assert!(value["database_path"].is_string());
    }

    #[tokio::test]
    async fn test_algorithms_resource() {
        let json = AlgorithmsResource::read().await.unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["default"], "a-star");
        let algorithms = value["algorithms"].as_array().unwrap();
        assert_eq!(algorithms.len(), 3);
        assert_eq!(algorithms[0]["name"], "bfs");
    }

    #[tokio::test]
    async fn test_spatial_index_status_resource() {
        let state = test_state();
        let json = SpatialIndexStatusResource::read(&state).await.unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["available"], true);
        assert_eq!(value["schema_version"], "e6c3");
        assert!(value["index_path"]
            .as_str()
            .unwrap()
            .contains("static_data.db"));
    }
}
