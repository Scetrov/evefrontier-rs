# Data Model: MCP Server Integration

**Date**: 2025-12-31  
**Feature**: 016-mcp-server-integration

## Overview

This document defines the data structures, tool schemas, and resource specifications for the EVE
Frontier MCP server implementation. All types are designed to work with the existing
`evefrontier-lib` data model.

## Core Entities

### McpServerState

Shared state for all MCP tool handlers. Loaded once at server initialization.

```rust
/// Server state shared across all MCP tool invocations.
pub struct McpServerState {
    /// Loaded starmap with all systems and gates.
    pub starmap: Arc<Starmap>,
    /// Pre-built spatial index for range queries. Lazily loaded.
    pub spatial_index: Option<Arc<SpatialIndex>>,
    /// Dataset metadata for resource responses.
    pub dataset_info: DatasetInfo,
}

/// Metadata about the loaded dataset.
#[derive(Debug, Clone, Serialize)]
pub struct DatasetInfo {
    pub system_count: usize,
    pub gate_count: usize,
    pub schema_version: String,
    pub database_path: PathBuf,
    pub loaded_at: DateTime<Utc>,
}
```

### Tool Input Types

All tool inputs derive `Deserialize` + `JsonSchema` for automatic validation.

#### RoutePlanInput

```rust
/// Input for the route_plan tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RoutePlanInput {
    /// Starting system name (required)
    pub origin: String,
    
    /// Goal system name (required)
    pub destination: String,
    
    /// Routing algorithm: "bfs", "dijkstra", or "a-star" (default: "a-star")
    #[serde(default)]
    pub algorithm: Option<String>,
    
    /// Maximum jump distance in light-years (optional)
    pub max_jump: Option<f64>,
    
    /// Maximum system temperature in Kelvin (optional)
    pub max_temperature: Option<f64>,
    
    /// System names to exclude from route (optional)
    #[serde(default)]
    pub avoid_systems: Vec<String>,
    
    /// Use spatial-only routing, ignore jump gates (default: false)
    #[serde(default)]
    pub avoid_gates: bool,
}
```

#### SystemInfoInput

```rust
/// Input for the system_info tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SystemInfoInput {
    /// System name to query (supports fuzzy matching)
    pub system_name: String,
}
```

#### SystemsNearbyInput

```rust
/// Input for the systems_nearby tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SystemsNearbyInput {
    /// Center system name (required)
    pub system_name: String,
    
    /// Search radius in light-years (required)
    pub radius_ly: f64,
    
    /// Maximum system temperature in Kelvin (optional)
    pub max_temperature: Option<f64>,
    
    /// Maximum number of results (default: 20, max: 100)
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize { 20 }
```

#### GatesFromInput

```rust
/// Input for the gates_from tool.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GatesFromInput {
    /// System name to query gates from
    pub system_name: String,
}
```

### Tool Output Types

#### RoutePlanOutput

```rust
/// Output from the route_plan tool.
#[derive(Debug, Clone, Serialize)]
pub struct RoutePlanOutput {
    /// Whether a route was found
    pub success: bool,
    
    /// Human-readable summary
    pub summary: String,
    
    /// Detailed route information (if found)
    pub route: Option<RouteDetails>,
    
    /// Error details (if not found)
    pub error: Option<RouteError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteDetails {
    /// Algorithm used
    pub algorithm: String,
    
    /// Starting system
    pub origin: SystemSummary,
    
    /// Goal system
    pub destination: SystemSummary,
    
    /// Number of hops
    pub hop_count: usize,
    
    /// Total distance in light-years (for spatial routes)
    pub total_distance_ly: Option<f64>,
    
    /// Number of gate jumps
    pub gate_jumps: usize,
    
    /// Number of spatial jumps
    pub spatial_jumps: usize,
    
    /// Ordered list of systems in the route
    pub waypoints: Vec<Waypoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Waypoint {
    pub system_name: String,
    pub system_id: u64,
    pub position: Position3D,
    pub min_temperature_k: f64,
    /// Type of jump to reach this system from previous
    pub edge_type: Option<String>, // "gate" or "spatial"
    /// Distance from previous system in light-years
    pub distance_ly: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteError {
    pub code: String,
    pub message: String,
    pub suggestions: Vec<String>,
}
```

#### SystemInfoOutput

```rust
/// Output from the system_info tool.
#[derive(Debug, Clone, Serialize)]
pub struct SystemInfoOutput {
    pub found: bool,
    pub system: Option<SystemDetails>,
    pub error: Option<SystemError>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemDetails {
    pub system_id: u64,
    pub name: String,
    pub position: Position3D,
    pub min_external_temperature_k: f64,
    pub planet_count: usize,
    pub moon_count: usize,
    pub connected_gates: Vec<GateConnection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GateConnection {
    pub destination_system: String,
    pub destination_id: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Position3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
```

#### SystemsNearbyOutput

```rust
/// Output from the systems_nearby tool.
#[derive(Debug, Clone, Serialize)]
pub struct SystemsNearbyOutput {
    pub center_system: String,
    pub radius_ly: f64,
    pub count: usize,
    pub systems: Vec<NearbySystem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NearbySystem {
    pub name: String,
    pub system_id: u64,
    pub distance_ly: f64,
    pub min_temperature_k: f64,
}
```

#### GatesFromOutput

```rust
/// Output from the gates_from tool.
#[derive(Debug, Clone, Serialize)]
pub struct GatesFromOutput {
    pub system_name: String,
    pub gate_count: usize,
    pub gates: Vec<GateConnection>,
}
```

## MCP Resources

### evefrontier://dataset/info

```json
{
  "system_count": 8247,
  "gate_count": 12456,
  "schema_version": "e6c3",
  "database_path": "/home/user/.cache/evefrontier_datasets/static_data.db",
  "loaded_at": "2025-12-31T12:00:00Z"
}
```

### evefrontier://algorithms

```json
{
  "algorithms": [
    {
      "name": "bfs",
      "description": "Breadth-first search for minimum hop count (ignores distances)"
    },
    {
      "name": "dijkstra",
      "description": "Shortest path by total distance in light-years"
    },
    {
      "name": "a-star",
      "description": "Heuristic-guided search using spatial coordinates (default, typically fastest)"
    }
  ],
  "default": "a-star"
}
```

### evefrontier://spatial-index/status

```json
{
  "available": true,
  "version": 2,
  "system_count": 8247,
  "source_checksum": "abc123...",
  "build_timestamp": "2025-12-30T10:00:00Z"
}
```

## Validation Rules

### System Names

- Must be non-empty string
- Fuzzy matching enabled (Levenshtein distance ≤ 3)
- Case-insensitive comparison
- On mismatch, return top 3 suggestions

### Numeric Constraints

- `radius_ly`: Must be > 0 and ≤ 1000 (prevents accidental full-dataset queries)
- `max_jump`: Must be > 0 and ≤ 500
- `max_temperature`: Must be > 0 (Kelvin)
- `limit`: Must be ≥ 1 and ≤ 100

### Algorithm Names

- Must be one of: "bfs", "dijkstra", "a-star"
- Case-insensitive
- Default: "a-star"

## State Transitions

The MCP server is stateless from the client's perspective. Each tool invocation:

1. Receives validated input (deserialization + schema validation)
2. Accesses shared read-only state (`Arc<Starmap>`, `Arc<SpatialIndex>`)
3. Calls `evefrontier-lib` functions
4. Returns serialized output

No mutations to server state occur during tool execution.

## Error Taxonomy

| Error Code | Description | Example |
|------------|-------------|---------|
| `UNKNOWN_SYSTEM` | System name not found | `{"code": "UNKNOWN_SYSTEM", "message": "Unknown system: 'Nodd'", "suggestions": ["Nod"]}` |
| `ROUTE_NOT_FOUND` | No path exists with constraints | `{"code": "ROUTE_NOT_FOUND", "message": "No route from Nod to Brana with max_temp=100K"}` |
| `INVALID_ALGORITHM` | Unknown algorithm name | `{"code": "INVALID_ALGORITHM", "message": "Unknown algorithm: 'dfs'"}` |
| `INVALID_PARAMETER` | Validation failure | `{"code": "INVALID_PARAMETER", "message": "radius_ly must be > 0"}` |
| `SPATIAL_INDEX_UNAVAILABLE` | Index not loaded | `{"code": "SPATIAL_INDEX_UNAVAILABLE", "message": "Spatial index required for nearby search"}` |
