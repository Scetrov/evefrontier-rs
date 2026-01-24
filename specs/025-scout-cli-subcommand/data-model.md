# Data Model: Scout CLI Subcommand

**Date**: 2026-01-24

## Response Structures

### Gate Neighbors Response

```rust
use serde::Serialize;

/// A gate-connected neighbor system.
#[derive(Debug, Clone, Serialize)]
pub struct GateNeighbor {
    /// System name.
    pub name: String,
    /// System ID.
    pub id: i64,
}

/// Result of a gate neighbors query.
#[derive(Debug, Clone, Serialize)]
pub struct ScoutGatesResult {
    /// The queried system name.
    pub system: String,
    /// The queried system ID.
    pub system_id: i64,
    /// Number of gate-connected neighbors.
    pub count: usize,
    /// List of neighboring systems.
    pub neighbors: Vec<GateNeighbor>,
}
```

### Range Query Response

```rust
use serde::Serialize;

/// A system within spatial range.
#[derive(Debug, Clone, Serialize)]
pub struct RangeNeighbor {
    /// System name.
    pub name: String,
    /// System ID.
    pub id: i64,
    /// Distance from origin in light-years.
    pub distance_ly: f64,
    /// Minimum external temperature in Kelvin (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_temp_k: Option<f64>,
}

/// Result of a range query.
#[derive(Debug, Clone, Serialize)]
pub struct ScoutRangeResult {
    /// The queried system name.
    pub system: String,
    /// The queried system ID.
    pub system_id: i64,
    /// Query parameters.
    pub query: RangeQueryParams,
    /// Number of systems found.
    pub count: usize,
    /// List of nearby systems ordered by distance.
    pub systems: Vec<RangeNeighbor>,
}

/// Query parameters for range search (echoed in response).
#[derive(Debug, Clone, Serialize)]
pub struct RangeQueryParams {
    /// Maximum number of results requested.
    pub limit: usize,
    /// Maximum distance in light-years (if specified).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,
    /// Maximum temperature filter in Kelvin (if specified).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_temperature: Option<f64>,
}
```

## CLI Argument Structures

```rust
use clap::{Args, Subcommand};

/// Scout command arguments (contains subcommand).
#[derive(Args, Debug, Clone)]
pub struct ScoutCommandArgs {
    #[command(subcommand)]
    pub subcommand: ScoutSubcommand,
}

/// Scout subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum ScoutSubcommand {
    /// List gate-connected neighbors of a system.
    Gates(ScoutGatesArgs),
    /// Find systems within spatial range of a system.
    Range(ScoutRangeArgs),
}

/// Arguments for gate neighbors query.
#[derive(Args, Debug, Clone)]
pub struct ScoutGatesArgs {
    /// System name to query (case-insensitive, fuzzy matched).
    pub system: String,
}

/// Arguments for range query.
#[derive(Args, Debug, Clone)]
pub struct ScoutRangeArgs {
    /// System name to query (case-insensitive, fuzzy matched).
    pub system: String,

    /// Maximum number of results to return.
    #[arg(long, short = 'n', default_value = "10", value_parser = clap::value_parser!(usize).range(1..=100))]
    pub limit: usize,

    /// Maximum distance in light-years from the origin system.
    #[arg(long, short = 'r')]
    pub radius: Option<f64>,

    /// Maximum star temperature in Kelvin (filters out hotter systems).
    #[arg(long = "max-temp", short = 't')]
    pub max_temp: Option<f64>,
}
```

## Validation Rules

| Field | Constraint | Error Message |
|-------|------------|---------------|
| `system` | Non-empty after trim | "System name cannot be empty" |
| `limit` | 1 ≤ limit ≤ 100 | "Limit must be between 1 and 100" |
| `radius` | > 0.0 if specified | "Radius must be a positive number" |
| `max_temp` | > 0.0 if specified | "Temperature must be a positive number" |

## JSON Output Examples

### Gates Response

```json
{
  "system": "Nod",
  "system_id": 30000001,
  "count": 3,
  "neighbors": [
    { "name": "Brana", "id": 30000002 },
    { "name": "D:2NAS", "id": 30000003 },
    { "name": "G:3OA0", "id": 30000004 }
  ]
}
```

### Range Response

```json
{
  "system": "Nod",
  "system_id": 30000001,
  "query": {
    "limit": 5,
    "radius": 50.0,
    "max_temperature": 300
  },
  "count": 5,
  "systems": [
    { "name": "Brana", "id": 30000002, "distance_ly": 12.4, "min_temp_k": 285.0 },
    { "name": "D:2NAS", "id": 30000003, "distance_ly": 23.1, "min_temp_k": 290.0 },
    { "name": "G:3OA0", "id": 30000004, "distance_ly": 34.7, "min_temp_k": 275.0 },
    { "name": "H:2L2S", "id": 30000005, "distance_ly": 41.2, "min_temp_k": 298.0 },
    { "name": "J:35IA", "id": 30000006, "distance_ly": 48.9, "min_temp_k": 280.0 }
  ]
}
```
