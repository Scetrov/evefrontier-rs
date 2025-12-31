# EVE Frontier CLI, Lambda & Library — Usage

This document describes how to build and use the `evefrontier-cli` workspace and its library crate
`evefrontier-lib`. Lambda crates will reuse the same APIs once they are implemented. Refer to
[`docs/TODO.md`](TODO.md) for the remaining backlog.

## Build

1. Build the entire workspace:

   ```pwsh
   cargo build --workspace
   ```

2. Build only the library and CLI crates:

   ```pwsh
   cargo build -p evefrontier-lib -p evefrontier-cli
   ```

## Run the CLI

The CLI provides two primary subcommands:

- **`download`** — Ensures the dataset is downloaded and reports its location
- **`route`** — Computes a route between two systems using various algorithms and options

Preferred invocation for end users and scripts:

```bash
evefrontier-cli <subcommand> [args]
```

Developer invocation (build-and-run via cargo):

```bash
cargo run -p evefrontier-cli -- <subcommand> [args]
```

Note: The examples below use the installed/release binary invocation. For development, prefix
commands with `cargo run -p evefrontier-cli --`.

Global options accepted by all subcommands:

- `--data-dir <PATH>` — override the dataset directory or file.
- `--dataset <TAG>` — request a specific dataset release.
- `--no-logo` — suppress the ASCII banner.
- `--no-footer` — suppress the completion timing footer.

Route-only options (ignored by other subcommands):

- `--format <text|rich|json|basic|emoji|note>` — control route display (defaults to `text`).

### Examples

- Download the latest dataset to the default location resolved by the CLI and report the path:

  ```pwsh
  evefrontier-cli download
  ```

- Download a specific dataset tag into a custom directory (helpful for tests or fixtures):

  ```pwsh
  evefrontier-cli download --data-dir docs/fixtures --dataset e6c3
  ```

> [!NOTE]
> The `download` subcommand always emits plain text regardless of `--format`.

- Calculate a route between two systems using the downloaded dataset:

  ```pwsh
  evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6"
  ```

- Request structured JSON output suitable for downstream tooling:

  ```bash
  evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --format json
  ```

- Get in-game note format with clickable system links:

  ```bash
  evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --format note
  ```

- Use different pathfinding algorithms:

  ```bash
  # Breadth-first search (unweighted, gate-only)
  evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --algorithm bfs

  # Dijkstra (weighted distance optimization)
  evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --algorithm dijkstra

  # A* (default, uses coordinates as heuristic)
  evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --algorithm a-star
  ```

- Limit jump distance and avoid specific systems:

  ```bash
  evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --max-jump 80.0 --avoid "IFM-228"
  ```

- Use spatial-only routing (no gates):

  ```bash
  evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --avoid-gates
  ```

- Filter by maximum system temperature for spatial jumps:

  ```bash
  evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --max-temp 5000.0
  ```

  Prevents routing through systems with star temperature above the threshold via spatial jumps
  (ships would overheat). Gate jumps bypass this constraint entirely.

- Calculate a route using environment variable for dataset path:

  ```bash
  export EVEFRONTIER_DATA_DIR="$HOME/.local/share/evefrontier"
  evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6"
  ```

### `download`

Ensures the requested dataset is present on disk and reports the resolved path. The command
downloads the specified dataset release (or reuses the cached copy) and writes it to the resolved
location. When `--dataset` is omitted the downloader uses the latest release from
[`Scetrov/evefrontier_datasets`](https://github.com/Scetrov/evefrontier_datasets).

```pwsh
evefrontier-cli download --data-dir docs/fixtures
```

### `route`

Computes a route between two system names using the selected algorithm (default: A\* hybrid graph
combining gates + spatial jumps). If the dataset is not already present, the CLI downloads it
automatically before computing the route.

```pwsh
evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6"
```

### Routing options

The routing subcommands accept several flags that map directly to the library's route planner:

- `--algorithm <bfs|dijkstra|a-star>` — select the pathfinding algorithm. `a-star` (default) uses
  coordinates as a heuristic over a hybrid graph. `dijkstra` optimises weighted distance. `bfs`
  performs an unweighted gate-only traversal.
- `--max-jump <LIGHT-YEARS>` — limit the maximum distance of an individual jump. Direct edges that
  exceed the threshold are pruned, encouraging multi-hop routes when necessary.
- `--avoid <SYSTEM>` — avoid specific systems by name. Repeat the flag to provide more than one
  entry. Avoiding the start or destination results in a clear error.
- `--avoid-gates` — restrict the search to spatial traversal only (omit gate edges). If system
  coordinates are absent the spatial graph may be sparse.
- `--max-temp <KELVIN>` — constrain the maximum star temperature for **spatial jumps only**. Spatial
  jumps to systems with star temperature exceeding this threshold are blocked (ships would
  overheat). Gate jumps are unaffected by temperature. Systems without temperature data are treated
  as safe.

### `index-build`
### Fuel projection (optional)

When planning routes, you can optionally calculate fuel consumption by specifying a ship and fuel
quality. The CLI will display fuel cost for each hop and total fuel required for the route.

```bash
# Use default ship (Reflex) and fuel quality (10%) - no flags needed
evefrontier-cli route --from "Nod" --to "Brana"

# Specify a different ship
evefrontier-cli route --from "Nod" --to "Brana" --ship "Reflex"

# Adjust fuel quality (1-100, default 10)
evefrontier-cli route --from "Nod" --to "Brana" --fuel-quality 15

# Include cargo mass in calculations
evefrontier-cli route --from "Nod" --to "Brana" --cargo-mass 5000

# Enable dynamic mass recalculation (mass decreases as fuel burns)
evefrontier-cli route --from "Nod" --to "Brana" --dynamic-mass
```

**Fuel calculation:**

The fuel cost for a jump is calculated using:

```
fuel_cost = (total_mass_kg / 100,000) × (fuel_quality / 100) × distance_ly
```

Where `total_mass_kg` includes:
- Ship base mass
- Fuel currently loaded
- Cargo mass (if specified)

**Static vs. Dynamic mass:**

- **Static mode (default):** Total mass remains constant throughout the route. Fuel consumption
  increases with remaining fuel load.
- **Dynamic mode (`--dynamic-mass`):** After each jump, fuel mass decreases. Subsequent hops cost
  less fuel because the ship is lighter. Useful for calculating actual fuel remaining and detecting
  fuel shortfalls.

**List available ships:**

```bash
evefrontier-cli route --list-ships
```

This displays all available ships from the bundled ship data catalog, with their base mass and fuel
capacity.

### `index-build`

Precomputes a KD-tree spatial index for efficient neighbor queries during routing. The index file is
saved alongside the database with a `.spatial.bin` extension.

```bash
evefrontier-cli index-build --data-dir docs/fixtures/minimal_static_data.db
```

Output: `docs/fixtures/minimal_static_data.db.spatial.bin`

Options:

- `--force` — overwrite an existing spatial index file if present.

The spatial index accelerates Dijkstra and A\* routing algorithms by efficiently finding nearby
systems within a given radius. Without a pre-built index, the CLI will build one automatically (with
a warning) when spatial/hybrid routing is requested.

**When to rebuild the index:**

- After updating the dataset (new systems, changed coordinates)
- After modifying temperature data in the dataset
- When switching between dataset versions

The index includes per-system minimum external temperature, enabling temperature-aware filtering
during neighbor queries.

### `index-verify`

Verifies that the spatial index artifact is fresh (built from the current dataset version). This
command compares the source metadata embedded in the spatial index file against the current
dataset's checksum and release tag.

```bash
evefrontier-cli index-verify --data-dir docs/fixtures
```

Options:

- `--json` — output in JSON format (suitable for CI automation)
- `--quiet` — only output on failure (quiet mode for scripts)
- `--strict` — require release tag match in addition to checksum

Exit codes:

| Code | Status         | Description                           |
|------|----------------|---------------------------------------|
| 0    | SUCCESS        | Index is fresh (matches dataset)      |
| 1    | STALE          | Index doesn't match dataset           |
| 2    | MISSING        | Spatial index file not found          |
| 3    | FORMAT_ERROR   | Legacy v1 format or corrupt file      |
| 4    | DATASET_MISSING| Dataset file not found                |
| 5    | ERROR          | Unexpected error during verification  |

**Examples:**

```bash
# Basic verification
evefrontier-cli index-verify

# CI-friendly JSON output
evefrontier-cli index-verify --json

# Quiet mode (only output on failure)
evefrontier-cli index-verify --quiet || echo "Index is stale!"
```

### Regenerating the Spatial Index

When the spatial index becomes stale (e.g., after downloading a new dataset version), you need to
regenerate it to ensure routing accuracy. The CI pipeline validates freshness automatically.

**Steps to regenerate:**

1. **Download the latest dataset** (if needed):

   ```bash
   evefrontier-cli download
   ```

2. **Rebuild the spatial index**:

   ```bash
   evefrontier-cli index-build --force
   ```

3. **Verify freshness**:

   ```bash
   evefrontier-cli index-verify
   ```

**Automated regeneration in CI:**

The CI workflow builds and verifies the spatial index on each run. If CI reports a stale index,
follow these steps locally, commit the regenerated `.spatial.bin` file (if tracked), and push.

### Troubleshooting CI Failures

If the `spatial-index-freshness` CI job fails, it indicates the spatial index is out of sync with
the dataset.

**Common causes:**

1. **Dataset was updated without rebuilding the index**
   - Run `evefrontier-cli index-build --force` locally
   - Commit any changes to tracked index files

2. **Legacy v1 format index file**
   - Older index files don't include source metadata
   - Run `evefrontier-cli index-build --force` to upgrade to v2 format

3. **Index file missing**
   - Run `evefrontier-cli index-build` to create the index

**Debugging steps:**

```bash
# Check the current status
evefrontier-cli index-verify --json

# Rebuild with metadata
evefrontier-cli index-build --force

# Re-verify
evefrontier-cli index-verify
```

### Spatial Index Format v2

The v2 spatial index format (introduced with freshness verification) embeds source dataset metadata
directly in the index file, enabling automated freshness checks.

**Format structure:**

- **Header** (16 bytes): Magic (`EFSI`), version (2), flags, node count
- **Metadata section** (variable): SHA-256 dataset checksum, release tag, build timestamp
- **Compressed data**: KD-tree nodes serialized with postcard + zstd
- **Checksum** (32 bytes): SHA-256 of compressed data for integrity

**Feature flags** (byte 5 of header):

| Bit | Flag              | Description                        |
|-----|-------------------|------------------------------------|
| 0   | HAS_TEMPERATURE   | Index includes temperature data    |
| 1   | HAS_METADATA      | v2 format with source metadata     |

**Backward compatibility:**

- v2 loader can read v1 format files (no metadata section, version byte = 1)
- v1 files trigger `LegacyFormat` result from `index-verify`
- Use `index-build --force` to upgrade v1 files to v2 format

### Lambda Freshness Behavior

AWS Lambda deployments **do not perform runtime freshness verification**. Instead, freshness is
validated at build-time when bundling the Lambda package.

**Rationale:**

- Lambda artifacts include the dataset and spatial index bundled via `include_bytes!`
- Runtime verification would add latency to every cold start
- Build-time verification ensures artifacts are consistent before deployment

**CI Integration:**

The `spatial-index-freshness` CI job verifies the fixture dataset and index before release. For
production Lambda deployments:

1. CI builds the Lambda with bundled artifacts
2. CI runs `index-verify` against the same artifacts
3. Only fresh builds are promoted to deployment

**Local development:**

When testing Lambda handlers locally, ensure your local dataset and index match:

```bash
evefrontier-cli index-build --force
evefrontier-cli index-verify
```

## Configuration & data path resolution

The CLI resolves the data path in the following order:

1. CLI `--data-dir` flag (if provided)
2. `EVEFRONTIER_DATA_DIR` environment variable
3. XDG `directories::ProjectDirs` default location. Examples:
   - Linux: `~/.local/share/evefrontier/static_data.db`
   - macOS: `~/Library/Application Support/com.evefrontier.evefrontier/static_data.db`
   - Windows: `%APPDATA%\\evefrontier\\static_data.db`

If the dataset is absent in all locations, the library will attempt to download it automatically.

## Library API

Key library entrypoints (in `crates/evefrontier-lib`):

- `ensure_dataset(target_dir: Option<&Path>, release: DatasetRelease)` — resolves or downloads the
  dataset release identified by `release`. The optional path argument allows tests to point at
  fixture data or custom paths. `ensure_e6c3_dataset` is still available as a shorthand for
  `DatasetRelease::tag("e6c3")`.
- `load_starmap(db_path: &Path)` — loads systems and jumps into memory with schema detection for the
  `SolarSystems`/`Jumps` schema. Each `System` entry includes optional metadata (region,
  constellation, and security status when available) plus coordinates (when exposed by the dataset)
  so callers do not need to perform additional lookups.
- `build_gate_graph`, `build_spatial_graph`, and `build_hybrid_graph` — construct gate-only,
  spatial-only, or mixed graphs from the `Starmap` depending on the routing mode. These helpers
  return a `Graph` that tracks edge types and distances.
- `find_route` — compute unweighted routes using BFS given a `Graph` returned by one of the
  constructors above.
- `RouteSummary::from_plan` — convert a `RoutePlan` into rich structs suitable for CLI or Lambda
  responses. Use `RouteSummary::render` with `RouteRenderMode::{PlainText, RichText, InGameNote}` to
  obtain ready-to-emit text while JSON is handled via `serde`.

## Testing

Run unit tests across the workspace:

```pwsh
cargo test --workspace
```

The library test suite uses the bundled fixture located at `docs/fixtures/minimal_static_data.db`.
This fixture is pinned to the e6c3 dataset release and uses legacy system names (Nod, Brana, etc.)
for deterministic testing. The fixture is protected from accidental overwrites.

> [!NOTE]
> The test fixture uses system names from the e6c3 release (Nod, Brana, H:2L2S, etc.).
> The production dataset uses different names. See `docs/fixtures/README.md` for details.

### Local dataset overrides

For development and testing you can override the GitHub download by setting the
`EVEFRONTIER_DATASET_SOURCE` environment variable to a local path. The path may point to either a
`.db` file or a `.zip` archive containing the database. When set, `ensure_dataset` (and convenience
wrappers like `ensure_e6c3_dataset`) copy or extract the local file instead of contacting GitHub.

```pwsh
$env:EVEFRONTIER_DATASET_SOURCE = "docs/fixtures/minimal_static_data.db"
evefrontier-cli download --data-dir target/fixtures
```

## Library API

The `evefrontier-lib` crate provides a programmatic API for integrating EVE Frontier routing into
your own applications. This section demonstrates common usage patterns.

### Basic Usage

The typical workflow involves three steps: ensuring the dataset, loading the starmap, and planning
routes.

```rust
use evefrontier_lib::{
    ensure_e6c3_dataset, load_starmap, plan_route,
    RouteRequest, RouteAlgorithm, RouteConstraints,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Ensure dataset is available (downloads if needed)
    let dataset_path = ensure_e6c3_dataset(None)?;

    // 2. Load starmap into memory
    let starmap = load_starmap(&dataset_path)?;

    // 3. Plan a route
    let request = RouteRequest {
        start: "ER1-MM7".to_string(),
        goal: "ENQ-PB6".to_string(),
        algorithm: RouteAlgorithm::AStar,
        constraints: RouteConstraints::default(),
    };

    let plan = plan_route(&starmap, &request)?;

    println!("Found route with {} hops", plan.hop_count());
    println!("Total gates: {}, spatial jumps: {}", plan.gates, plan.jumps);

    Ok(())
}
```

### Using Different Algorithms

Three routing algorithms are available:

```rust
use evefrontier_lib::{RouteRequest, RouteAlgorithm};

// Breadth-first search (shortest hop count, unweighted)
let request_bfs = RouteRequest {
    start: "ER1-MM7".to_string(),
    goal: "ENQ-PB6".to_string(),
    algorithm: RouteAlgorithm::Bfs,
    constraints: Default::default(),
};

// Dijkstra (shortest distance in light-years)
let request_dijkstra = RouteRequest {
    start: "ER1-MM7".to_string(),
    goal: "ENQ-PB6".to_string(),
    algorithm: RouteAlgorithm::Dijkstra,
    constraints: Default::default(),
};

// A* with heuristic (default, usually fastest)
let request_astar = RouteRequest {
    start: "ER1-MM7".to_string(),
    goal: "ENQ-PB6".to_string(),
    algorithm: RouteAlgorithm::AStar,
    constraints: Default::default(),
};
```

### Applying Route Constraints

You can constrain routes by maximum jump distance, avoided systems, or temperature:

```rust
use evefrontier_lib::{RouteRequest, RouteAlgorithm, RouteConstraints};

let request = RouteRequest {
    start: "ER1-MM7".to_string(),
    goal: "ENQ-PB6".to_string(),
    algorithm: RouteAlgorithm::AStar,
    constraints: RouteConstraints {
        max_jump: Some(80.0),  // Max 80 ly per jump
        avoid_systems: vec!["IFM-228".to_string()],  // Avoid this system
        avoid_gates: false,  // Allow gate usage
        max_temperature: Some(50.0),  // Exclude hot systems
    },
};
```

### Error Handling

The library provides detailed error types with context:

```rust
use evefrontier_lib::{plan_route, Error};

match plan_route(&starmap, &request) {
    Ok(plan) => {
        println!("Route found!");
    }
    Err(Error::UnknownSystem { name, suggestions }) => {
        eprintln!("Unknown system: {}", name);
        if !suggestions.is_empty() {
            eprintln!("Did you mean: {:?}", suggestions);
        }
    }
    Err(Error::RouteNotFound { start, goal }) => {
        eprintln!("No route found between {} and {}", start, goal);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

### Formatting Output

Convert route plans to various output formats:

```rust
use evefrontier_lib::{RouteSummary, RouteRenderMode};

let plan = plan_route(&starmap, &request)?;

// Convert to summary with system names
let summary = RouteSummary::from_plan(&plan, &starmap)?;

// Render as plain text
let text = summary.render(RouteRenderMode::PlainText);
println!("{}", text);

// Or serialize to JSON
let json = serde_json::to_string_pretty(&summary)?;
println!("{}", json);
```

#### Enhanced format example (CLI)

```bash
evefrontier-cli --no-logo --format enhanced route --from "Nod" --to "Brana"
```

Sample output (colors may vary by terminal):

```
Route from Nod to Brana (3 jumps):
 STRT  ● Nod
   │ min  15.74K,  2 Planets
 GATE  ● J:35IA (gate, 119ly)
   │ min   3.69K,  8 Planets,  6 Moons
 JUMP  ● G:3OA0 (jump, 110ly)
   │ min   1.31K,  3 Planets,  1 Moon
 GOAL  ● Brana (gate, 143ly)
   │ min   0.32K,  2 Planets,  2 Moons

───────────────────────────────────────
  Total Distance:  373ly
  Via Gates:       262ly
  Via Jumps:       110ly
```

> Black hole systems (IDs 30000001–30000003) display a “Black Hole” badge on the status line.

### Using Custom Dataset Paths

For testing or using alternative datasets:

```rust
use evefrontier_lib::{ensure_dataset, DatasetRelease, load_starmap};
use std::path::Path;

// Use a specific dataset tag
let path = ensure_dataset(None, DatasetRelease::tag("e6c3"))?;

// Or point to a local fixture
let fixture_path = Path::new("docs/fixtures/minimal_static_data.db");
let starmap = load_starmap(fixture_path)?;
```

### Performance Considerations

- **Starmap Loading**: Loading the dataset into memory (`load_starmap`) is a one-time cost. Reuse
  the `Starmap` instance for multiple route computations.

- **Algorithm Selection**:
  - BFS: Fastest for short routes, unweighted
  - Dijkstra: Accurate distance optimization, slightly slower
  - A\*: Best balance of speed and accuracy for most use cases

- **Constraint Impact**: Each constraint (avoided systems, max jump, etc.) may increase route
  computation time. Use sparingly for best performance.

## MCP Server Integration

The workspace provides an MCP (Model Context Protocol) server that exposes EVE Frontier route planning
and system query functionality to AI assistants like Claude Desktop, VS Code, and Cursor.

The MCP server communicates via stdio using JSON-RPC 2.0 and provides:
- **Tools**: Interactive route planning, system info lookup, spatial queries, gate connections
- **Resources**: Dataset metadata, algorithm capabilities, spatial index status
- **Prompts**: (Coming in Phase 9) Pre-configured navigation and exploration templates

### Quick Start

Run the MCP server directly:

```bash
cargo run -p evefrontier-mcp
```

The server listens on stdin/stdout for JSON-RPC 2.0 messages. Example:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run -p evefrontier-mcp
```

### Configuration Examples

#### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or
`%APPDATA%\Claude\claude_desktop_config.json` (Windows):

```json
{
  "mcpServers": {
    "evefrontier": {
      "command": "cargo",
      "args": ["run", "-p", "evefrontier-mcp"],
      "cwd": "/path/to/evefrontier-rs"
    }
  }
}
```

#### VS Code (with MCP Extension)

Add to `.vscode/mcp.json`:

```json
{
  "mcpServers": {
    "evefrontier": {
      "command": "cargo",
      "args": ["run", "-p", "evefrontier-mcp"],
      "cwd": "${workspaceFolder}"
    }
  }
}
```

#### Cursor

Add to Cursor Settings → MCP Servers:

```json
{
  "evefrontier": {
    "command": "cargo",
    "args": ["run", "-p", "evefrontier-mcp"],
    "cwd": "/path/to/evefrontier-rs"
  }
}
```

### Available Tools

The MCP server exposes four tools for AI assistants:

#### `route_plan`

Plan a route between two star systems with optional constraints.

**Input Schema:**

```typescript
{
  origin: string;           // Starting system name (required)
  destination: string;      // Destination system name (required)
  algorithm?: "bfs" | "dijkstra" | "a-star"; // Routing algorithm (default: a-star)
  max_jump?: number;        // Maximum jump distance in light years
  max_temperature?: number; // Maximum system temperature in Kelvin
  avoid_systems?: string[]; // Systems to avoid
  avoid_gates?: boolean;    // Avoid jump gates (spatial only)
}
```

**Example Request:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "route_plan",
    "arguments": {
      "origin": "Nod",
      "destination": "Brana",
      "algorithm": "a-star",
      "max_jump": 80
    }
  }
}
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"success\":true,\"summary\":\"Route from Nod to Brana: 2 jumps, 45.2 ly\",\"route\":{...}}"
      }
    ]
  }
}
```

#### `system_info`

Get detailed information about a star system.

**Input Schema:**

```typescript
{
  system_name: string; // System name to query (required)
}
```

**Example:**

```json
{
  "name": "system_info",
  "arguments": { "system_name": "Nod" }
}
```

Returns system coordinates, temperature, planets, moons, and gate connections.

#### `systems_nearby`

Find star systems within a spatial radius.

**Input Schema:**

```typescript
{
  origin: string;           // Center system name (required)
  radius: number;           // Search radius in light years (required)
  max_temperature?: number; // Maximum system temperature filter
}
```

**Example:**

```json
{
  "name": "systems_nearby",
  "arguments": {
    "origin": "Nod",
    "radius": 50,
    "max_temperature": 5000
  }
}
```

Returns list of systems within range, sorted by distance.

#### `gates_from`

Get jump gate connections from a system.

**Input Schema:**

```typescript
{
  system_name: string; // System name to query (required)
}
```

**Example:**

```json
{
  "name": "gates_from",
  "arguments": { "system_name": "Nod" }
}
```

Returns list of gate-connected neighbor systems.

### Available Resources

The MCP server exposes three resources for context:

#### `evefrontier://dataset/info`

Dataset metadata including system count, gate count, schema version.

**Response:**

```json
{
  "system_count": 8,
  "gate_count": 12,
  "schema_version": "static_data.db",
  "release_tag": "v0.1.0",
  "checksum": "abc123..."
}
```

#### `evefrontier://algorithms`

Available routing algorithms and their capabilities.

**Response:**

```json
{
  "algorithms": [
    {
      "name": "bfs",
      "description": "Breadth-first search for unweighted gate routes",
      "constraints": ["gate_only", "no_max_jump", "fast"]
    },
    {
      "name": "dijkstra",
      "description": "Weighted routing supporting gate and spatial edges",
      "constraints": ["supports_max_jump", "supports_temperature", "gate_or_spatial"]
    },
    {
      "name": "a-star",
      "description": "Heuristic-guided routing prioritizing shortest spatial distance",
      "constraints": ["supports_max_jump", "supports_temperature", "heuristic_spatial"]
    }
  ],
  "default": "a-star"
}
```

#### `evefrontier://spatial-index/status`

Spatial index availability and initialization status.

**Response:**

```json
{
  "available": true,
  "path": "/path/to/static_data.db.spatial.bin",
  "initialized_at": "2025-12-31T12:34:56Z",
  "format_version": 2,
  "source_checksum": "abc123...",
  "source_release_tag": "v0.1.0"
}
```

### Protocol Details

The MCP server implements JSON-RPC 2.0 over stdio transport with the following methods:

- `initialize`: Protocol handshake (returns version, capabilities, server info)
- `tools/list`: List all available tools with schemas
- `tools/call`: Invoke a tool with arguments
- `resources/list`: List all available resources
- `resources/read`: Read resource content by URI
- `prompts/list`: List available prompt templates (Phase 9)

All responses follow the MCP specification format with `content` arrays containing
structured data as JSON strings.

### Error Handling

The server returns standard JSON-RPC error codes:

- `-32700`: Parse error (malformed JSON)
- `-32600`: Invalid request (non-2.0 protocol version)
- `-32601`: Method not found (unknown tool/resource)
- `-32602`: Invalid params (validation failure)
- `-32603`: Internal error (server-side failure)

**Example Error Response:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid parameters: origin cannot be empty"
  }
}
```

### Performance Considerations

- **Cold Start**: First request loads dataset (~100ms) and spatial index (~50ms)
- **Tool Latency**: Route planning: 10-100ms, system queries: <10ms
- **Memory**: ~50-100MB RAM for dataset + spatial index
- **Concurrency**: Single-threaded, processes one request at a time

### Debugging

Enable detailed logging by setting the `RUST_LOG` environment variable:

```bash
RUST_LOG=evefrontier_mcp=debug cargo run -p evefrontier-mcp
```

Logs are written to stderr to avoid corrupting the stdio protocol on stdout.

### Testing

Run integration tests that verify the JSON-RPC protocol:

```bash
cargo test -p evefrontier-mcp --test integration_test
```

Tests spawn the server and verify all protocol methods, tool invocations, and error handling.

## AWS Lambda Functions

The workspace provides three AWS Lambda functions for serverless route planning and navigation. Each
Lambda is a thin wrapper around `evefrontier-lib` with optimized cold-start performance via bundled
dataset and spatial index.

> [!TIP]
> For infrastructure setup and deployment instructions, see [DEPLOYMENT.md](./DEPLOYMENT.md). This
> section covers API usage assuming functions are already deployed.

### Lambda Function Overview

| Function                         | Endpoint       | Description                                                  |
| -------------------------------- | -------------- | ------------------------------------------------------------ |
| `evefrontier-lambda-route`       | `/route`       | Compute routes between systems with algorithm selection      |
| `evefrontier-lambda-scout-gates` | `/scout-gates` | Find gate-connected neighbors of a system                    |
| `evefrontier-lambda-scout-range` | `/scout-range` | Find systems within spatial range with temperature filtering |

All Lambda functions:

- Accept JSON requests and return JSON responses
- Use RFC 9457 Problem Details for structured error responses
- Support tracing for CloudWatch Logs integration
- Bundle the dataset and spatial index for zero-download cold starts
- Share a common runtime initialized once per Lambda container lifecycle

### Route Lambda

Computes routes between two systems using configurable algorithms and constraints.

#### Request Schema

```json
{
  "from": "ER1-MM7",
  "to": "ENQ-PB6",
  "algorithm": "a-star",
  "max_jump": 80.0,
  "avoid": ["IFM-228"],
  "avoid_gates": false,
  "max_temperature": 50.0
}
```

**Fields:**

- `from` (required): Starting system name
- `to` (required): Destination system name
- `algorithm` (optional): `"bfs"`, `"dijkstra"`, or `"a-star"` (default: `"a-star"`)
- `max_jump` (optional): Maximum jump distance in light-years
- `avoid` (optional): Array of system names to avoid
- `avoid_gates` (optional): If `true`, use only spatial jumps (default: `false`)
- `max_temperature` (optional): Maximum star temperature threshold in Kelvin

#### Response Schema

**Success (HTTP 200):**

```json
{
  "content_type": "application/json",
  "hops": 2,
  "gates": 2,
  "jumps": 0,
  "algorithm": "a-star",
  "route": ["ER1-MM7", "IFM-228", "ENQ-PB6"]
}
```

_Note: The `LambdaResponse` wrapper uses `#[serde(flatten)]`, so response fields are merged directly
into the top level._

**Error (HTTP 400/404/500):**

```json
{
  "type": "https://evefrontier.example/problems/unknown-system",
  "title": "Unknown System",
  "status": 404,
  "detail": "System 'InvalidName' not found in dataset. Did you mean: ER1-MM7, ENQ-PB6?",
  "instance": "/route/req-abc123"
}
```

#### Invocation Examples

**AWS SDK (Python):**

```python
import boto3
import json

lambda_client = boto3.client('lambda', region_name='us-east-1')

payload = {
    "from": "ER1-MM7",
    "to": "ENQ-PB6",
    "algorithm": "a-star",
    "max_jump": 80.0
}

response = lambda_client.invoke(
    FunctionName='evefrontier-lambda-route',
    InvocationType='RequestResponse',
    Payload=json.dumps(payload)
)

result = json.loads(response['Payload'].read())
print(f"Route: {result['data']['route']}")
print(f"Hops: {result['data']['hops']}")
```

**AWS SDK (JavaScript/Node.js):**

```javascript
const { LambdaClient, InvokeCommand } = require("@aws-sdk/client-lambda");

const client = new LambdaClient({ region: "us-east-1" });

const payload = {
  from: "ER1-MM7",
  to: "ENQ-PB6",
  algorithm: "a-star",
  max_jump: 80.0
};

const command = new InvokeCommand({
  FunctionName: "evefrontier-route",
  Payload: JSON.stringify(payload)
});

const response = await client.send(command);
const result = JSON.parse(Buffer.from(response.Payload).toString());
console.log(`Route: ${result.data.route}`);
```

**curl (via API Gateway):**

```bash
curl -X POST https://api.example.com/route \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "from": "ER1-MM7",
    "to": "ENQ-PB6",
    "algorithm": "a-star"
  }'
```

### Scout Gates Lambda

Returns gate-connected neighbors of a system.

#### Request Schema

```json
{
  "system": "ER1-MM7"
}
```

**Fields:**

- `system` (required): System name to find neighbors for

#### Response Schema

**Success (HTTP 200):**

```json
{
  "content_type": "application/json",
  "system": "ER1-MM7",
  "system_id": 30001178,
  "count": 4,
  "neighbors": [
    {
      "name": "IFM-228",
      "id": 30001177
    },
    {
      "name": "E85-NR6",
      "id": 30001179
    }
  ]
}
```

_Note: The `LambdaResponse` wrapper uses `#[serde(flatten)]`, so response fields are merged directly
into the top level._

#### Invocation Examples

**AWS SDK (Python):**

```python
payload = {"system": "ER1-MM7"}

response = lambda_client.invoke(
    FunctionName='evefrontier-lambda-scout-gates',
    InvocationType='RequestResponse',
    Payload=json.dumps(payload)
)

result = json.loads(response['Payload'].read())
neighbors = result['data']['neighbors']
print(f"Found {result['data']['count']} gate-connected systems:")
for neighbor in neighbors:
    print(f"  - {neighbor['name']} (ID: {neighbor['id']})")
```

**curl (via API Gateway):**

```bash
curl -X POST https://api.example.com/scout-gates \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{"system": "ER1-MM7"}'
```

### Scout Range Lambda

Returns systems within spatial range with optional temperature filtering.

#### Request Schema

```json
{
  "system": "ER1-MM7",
  "limit": 50,
  "radius": 100.0,
  "max_temperature": 50.0
}
```

**Fields:**

- `system` (required): Center system name
- `limit` (optional): Maximum number of results (default: 50)
- `radius` (optional): Maximum distance in light-years (no limit if omitted)
- `max_temperature` (optional): Maximum star temperature threshold in Kelvin

#### Response Schema

**Success (HTTP 200):**

```json
{
  "content_type": "application/json",
  "system": "ER1-MM7",
  "system_id": 30001178,
  "count": 3,
  "systems": [
    {
      "name": "IFM-228",
      "id": 30001177,
      "distance_ly": 25.4,
      "min_temp_k": 2.45
    },
    {
      "name": "ENQ-PB6",
      "id": 30001176,
      "distance_ly": 67.8,
      "min_temp_k": 22.1
    },
    {
      "name": "E85-NR6",
      "id": 30001179,
      "distance_ly": 88.3
    }
  ]
}
```

_Note: The `LambdaResponse` wrapper uses `#[serde(flatten)]`, so response fields are merged directly
into the top level._

**Notes:**

- Results are ordered by distance (closest first)
- `min_temp_k` field is included only if temperature data is available in the dataset
- KD-tree spatial index is used for efficient neighbor queries (sub-millisecond for typical
  datasets)

#### Invocation Examples

**AWS SDK (Python):**

```python
payload = {
    "system": "ER1-MM7",
    "radius": 100.0,
    "max_temperature": 50.0,
    "limit": 10
}

response = lambda_client.invoke(
    FunctionName='evefrontier-lambda-scout-range',
    InvocationType='RequestResponse',
    Payload=json.dumps(payload)
)

result = json.loads(response['Payload'].read())
systems = result['data']['systems']
print(f"Found {result['data']['count']} systems within range:")
for system in systems:
    temp_info = f", temp: {system['min_temp_k']}K" if 'min_temp_k' in system else ""
    print(f"  - {system['name']}: {system['distance_ly']:.1f} ly{temp_info}")
```

**curl (via API Gateway):**

```bash
curl -X POST https://api.example.com/scout-range \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "system": "ER1-MM7",
    "radius": 100.0,
    "max_temperature": 50.0
  }'
```

### Cold Start & Initialization

Lambda functions bundle the EVE Frontier dataset and spatial index at build time for optimal
cold-start performance. Initialization happens once per Lambda container lifecycle.

#### Initialization Sequence

1. **Tracing Setup**: JSON-formatted CloudWatch Logs integration
2. **Database Loading**: Zero-copy deserialization from bundled bytes
   (`rusqlite::deserialize_bytes`)
3. **Spatial Index Loading**: KD-tree deserialized from bundled compressed bytes
4. **Starmap Construction**: In-memory graph built from database tables

#### Cold Start Timing

Typical cold-start metrics (logged via tracing):

```json
{
  "level": "INFO",
  "message": "runtime initialized",
  "db_load_ms": 45,
  "index_load_ms": 12,
  "total_init_ms": 57,
  "timestamp": "2025-12-05T10:30:45.123Z"
}
```

- **Database load**: 30-60ms (depends on dataset size)
- **Spatial index load**: 10-20ms (compressed format enables fast decompression)
- **Total cold start**: <100ms for typical e6c3 dataset

**Warm invocations** reuse the initialized container and return in <10ms.

#### Memory Usage

- **Bundled assets**: ~20-50 MB (dataset + spatial index compressed)
- **Runtime memory**: ~80-150 MB (starmap, KD-tree, graph structures)
- **Total Lambda memory**: 512 MB recommended (256 MB minimum)

### Configuration & Secrets

#### Environment Variables

Currently, Lambda functions do not require environment configuration. All data is bundled at build
time.

**Future configuration options** (when implemented):

- `LOG_LEVEL`: Tracing verbosity (`debug`, `info`, `warn`, `error`)
- `DATASET_VERSION`: Override bundled dataset version (if dynamic loading enabled)

#### IAM Permissions

Lambda functions require minimal IAM permissions:

**Required:**

- `logs:CreateLogGroup`
- `logs:CreateLogStream`
- `logs:PutLogEvents`

**If using API Gateway:**

- Configure authentication (API keys, Cognito, IAM, or custom authorizers)

**Future requirements** (when secrets are needed):

- `secretsmanager:GetSecretValue` (if API tokens or credentials are externalized)

#### Deployment Considerations

1. **Build with bundled data**:

   ```bash
   cargo build --release -p evefrontier-lambda-route --features bundle-data
   cargo build --release -p evefrontier-lambda-scout-gates --features bundle-data
   cargo build --release -p evefrontier-lambda-scout-range --features bundle-data
   ```

2. **Package for AWS Lambda**:
   - Binaries must be named `bootstrap` for custom Lambda runtimes
   - Package in `.zip` with binary at root
   - Or use container images with AWS Lambda Runtime Interface

3. **Lambda Configuration**:
   - Runtime: `provided.al2` (custom Rust runtime)
   - Memory: 512 MB (recommended)
   - Timeout: 10 seconds (routes typically compute in <1s)
   - Architecture: `arm64` or `x86_64` (match build target)

4. **API Gateway Integration**:
   - Set up REST API or HTTP API endpoints
   - Configure CORS if needed for browser clients
   - Enable CloudWatch Logs for request tracing

**See `docs/RELEASE.md` (future)** for detailed deployment automation and Terraform templates.

## Development Scripts

The `scripts/` directory contains utility scripts for fixture management, database inspection, and
development tooling. All scripts are registered as Nx tasks.

### Available Script Tasks

```bash
# Verify test fixture integrity
pnpm nx run scripts:fixture-verify

# Show current fixture status
pnpm nx run scripts:fixture-status

# Record fixture metadata (after updates)
pnpm nx run scripts:fixture-record

# Inspect a database file
pnpm nx run scripts:inspect-db docs/fixtures/minimal_static_data.db

# Run all verification tasks
pnpm nx run scripts:verify-all
```

### Fixture Management

The test fixture at `docs/fixtures/minimal_static_data.db` is pinned and verified against recorded
metadata. This ensures deterministic test results across different environments.

**Verify fixture integrity:**

```bash
pnpm nx run scripts:fixture-verify
```

This command compares the current fixture against the recorded SHA-256 checksum and table row counts
in `docs/fixtures/minimal_static_data.meta.json`. It fails if the fixture has been modified.

**Record new metadata after updating the fixture:**

```bash
pnpm nx run scripts:fixture-record
```

Use this after intentionally modifying the fixture (e.g., adding new test systems).

### Database Inspection

The `inspect-db` task displays the schema and contents of any evefrontier SQLite database:

```bash
pnpm nx run scripts:inspect-db docs/fixtures/minimal_static_data.db
```

Output includes table names, system data, planets, moons, and jump gates.

### Python Environment

Scripts use Python stdlib only (no external dependencies). For future scripts requiring
dependencies, run:

```bash
pnpm nx run scripts:venv-setup
```

This creates a virtual environment at `scripts/.venv/` and installs packages from
`scripts/requirements.txt`.

For detailed script documentation, see `scripts/README.md`.
