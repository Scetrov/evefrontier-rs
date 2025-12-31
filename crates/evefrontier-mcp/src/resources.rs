//! MCP resource implementations for dataset metadata
//!
//! This module defines the three resources exposed by the MCP server:
//! - evefrontier://dataset/info: Dataset metadata and schema
//! - evefrontier://algorithms: Available routing algorithms
//! - evefrontier://spatial-index/status: Spatial index build status

/// Dataset information resource
///
/// Returns metadata about the loaded EVE Frontier dataset including
/// system count, jump count, schema version, and build timestamp.
pub struct DatasetInfoResource;

impl DatasetInfoResource {
    /// Handle a dataset info resource read
    ///
    /// TODO: Implement in Phase 2
    pub async fn read() -> crate::Result<String> {
        Ok("{}".to_string())
    }
}

/// Algorithms resource
///
/// Returns a list of available routing algorithms and their
/// constraints/capabilities.
pub struct AlgorithmsResource;

impl AlgorithmsResource {
    /// Handle an algorithms resource read
    ///
    /// TODO: Implement in Phase 2
    pub async fn read() -> crate::Result<String> {
        Ok("{}".to_string())
    }
}

/// Spatial index status resource
///
/// Returns information about the spatial index including its version,
/// build timestamp, and whether it's currently loaded.
pub struct SpatialIndexStatusResource;

impl SpatialIndexStatusResource {
    /// Handle a spatial index status resource read
    ///
    /// TODO: Implement in Phase 2
    pub async fn read() -> crate::Result<String> {
        Ok("{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_stubs_defined() {
        // Verify resource structures are properly defined
        let _info = DatasetInfoResource;
        let _algorithms = AlgorithmsResource;
        let _status = SpatialIndexStatusResource;
    }
}
