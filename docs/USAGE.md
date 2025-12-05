# EveFrontier CLI, Lambda & Library — Usage

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

Note: The examples below use the installed/release binary invocation. For development, prefix commands with `cargo run -p evefrontier-cli --`.

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

> Note: The `download` subcommand always emits plain text regardless of `--format`.

- Calculate a route between two systems using an existing dataset path:

  ```pwsh
  evefrontier-cli route --from "Y:170N" --to "BetaTest" --data-dir docs/fixtures/minimal_static_data.db
  ```

- Request structured JSON output suitable for downstream tooling:

  ```bash
  evefrontier-cli route --from "Y:170N" --to "BetaTest" --format json
  ```

- Get in-game note format with clickable system links:

  ```bash
  evefrontier-cli route --from "Y:170N" --to "BetaTest" --format note
  ```

- Use different pathfinding algorithms:

  ```bash
  # Breadth-first search (unweighted, gate-only)
  evefrontier-cli route --from "Y:170N" --to "BetaTest" --algorithm bfs

  # Dijkstra (weighted distance optimization)
  evefrontier-cli route --from "Y:170N" --to "BetaTest" --algorithm dijkstra

  # A* (default, uses coordinates as heuristic)
  evefrontier-cli route --from "Y:170N" --to "BetaTest" --algorithm a-star
  ```

- Limit jump distance and avoid specific systems:

  ```bash
  evefrontier-cli route --from "Y:170N" --to "BetaTest" --max-jump 5.0 --avoid "AlphaTest"
  ```

- Use spatial-only routing (no gates):

  ```bash
  evefrontier-cli route --from "Y:170N" --to "BetaTest" --avoid-gates
  ```

- Filter by maximum system temperature for spatial jumps:

  ```bash
  evefrontier-cli route --from "Y:170N" --to "BetaTest" --max-temp 5000.0
  ```

  Prevents routing through systems with star temperature above the threshold via spatial jumps
  (ships would overheat). Gate jumps bypass this constraint entirely.

- Calculate a route using environment variable for dataset path:

  ```bash
  export EVEFRONTIER_DATA_DIR="$HOME/.local/share/evefrontier"
  evefrontier-cli route --from "Y:170N" --to "BetaTest"
  ```

### `download`

Ensures the requested dataset is present on disk and reports the resolved path. The command downloads
the specified dataset release (or reuses the cached copy) and writes it to the resolved location. When
`--dataset` is omitted the downloader uses the latest release from
[`Scetrov/evefrontier_datasets`](https://github.com/Scetrov/evefrontier_datasets).

```pwsh
evefrontier-cli download --data-dir docs/fixtures
```

### `route`

Computes a route between two system names using the selected algorithm (default: A* hybrid
graph combining gates + spatial jumps). If the dataset is not already present, the CLI
downloads it automatically before computing the route.

```pwsh
evefrontier-cli route --from "Y:170N" --to "BetaTest" --data-dir docs/fixtures/minimal_static_data.db
```

### Routing options

The routing subcommands accept several flags that map directly to the library's route planner:

- `--algorithm <bfs|dijkstra|a-star>` — select the pathfinding algorithm. `a-star` (default)
  uses coordinates as a heuristic over a hybrid graph. `dijkstra` optimises weighted distance.
  `bfs` performs an unweighted gate-only traversal.
- `--max-jump <LIGHT-YEARS>` — limit the maximum distance of an individual jump. Direct edges that
  exceed the threshold are pruned, encouraging multi-hop routes when necessary.
- `--avoid <SYSTEM>` — avoid specific systems by name. Repeat the flag to provide more than one
  entry. Avoiding the start or destination results in a clear error.
- `--avoid-gates` — restrict the search to spatial traversal only (omit gate edges). If
  system coordinates are absent the spatial graph may be sparse.
- `--max-temp <KELVIN>` — constrain the maximum star temperature for **spatial jumps only**. 
  Spatial jumps to systems with star temperature exceeding this threshold are blocked (ships 
  would overheat). Gate jumps are unaffected by temperature. Systems without temperature data 
  are treated as safe.

### `index-build`

Precomputes a KD-tree spatial index for efficient neighbor queries during routing. The index
file is saved alongside the database with a `.spatial.bin` extension.

```bash
evefrontier-cli index-build --data-dir docs/fixtures/minimal_static_data.db
```

Output: `docs/fixtures/minimal_static_data.db.spatial.bin`

Options:

- `--force` — overwrite an existing spatial index file if present.

The spatial index accelerates Dijkstra and A* routing algorithms by efficiently finding
nearby systems within a given radius. Without a pre-built index, the CLI will build one
automatically (with a warning) when spatial/hybrid routing is requested.

**When to rebuild the index:**

- After updating the dataset (new systems, changed coordinates)
- After modifying temperature data in the dataset
- When switching between dataset versions

The index includes per-system minimum external temperature, enabling temperature-aware
filtering during neighbor queries.

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
  `SolarSystems`/`Jumps` schema. Each `System` entry includes optional metadata (region, constellation,
  and security status when available) plus coordinates (when exposed by the dataset) so callers do
  not need to perform additional lookups.
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
You can reuse the same file when running the CLI by passing `--data-dir docs/fixtures/minimal_static_data.db`.

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
        start: "Nod".to_string(),
        goal: "Brana".to_string(),
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
    start: "Nod".to_string(),
    goal: "Brana".to_string(),
    algorithm: RouteAlgorithm::Bfs,
    constraints: Default::default(),
};

// Dijkstra (shortest distance in light-years)
let request_dijkstra = RouteRequest {
    start: "Nod".to_string(),
    goal: "Brana".to_string(),
    algorithm: RouteAlgorithm::Dijkstra,
    constraints: Default::default(),
};

// A* with heuristic (default, usually fastest)
let request_astar = RouteRequest {
    start: "Nod".to_string(),
    goal: "Brana".to_string(),
    algorithm: RouteAlgorithm::AStar,
    constraints: Default::default(),
};
```

### Applying Route Constraints

You can constrain routes by maximum jump distance, avoided systems, or temperature:

```rust
use evefrontier_lib::{RouteRequest, RouteAlgorithm, RouteConstraints};

let request = RouteRequest {
    start: "Nod".to_string(),
    goal: "Brana".to_string(),
    algorithm: RouteAlgorithm::AStar,
    constraints: RouteConstraints {
        max_jump: Some(80.0),  // Max 80 ly per jump
        avoid_systems: vec!["H:2L2S".to_string()],  // Avoid this system
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

- **Starmap Loading**: Loading the dataset into memory (`load_starmap`) is a one-time cost.
  Reuse the `Starmap` instance for multiple route computations.

- **Algorithm Selection**:
  - BFS: Fastest for short routes, unweighted
  - Dijkstra: Accurate distance optimization, slightly slower
  - A*: Best balance of speed and accuracy for most use cases

- **Constraint Impact**: Each constraint (avoided systems, max jump, etc.) may increase route
  computation time. Use sparingly for best performance.

## AWS Lambda Functions

The workspace provides three AWS Lambda functions for serverless route planning and navigation. Each
Lambda is a thin wrapper around `evefrontier-lib` with optimized cold-start performance via bundled
dataset and spatial index.

### Lambda Function Overview

| Function | Endpoint | Description |
|----------|----------|-------------|
| `evefrontier-lambda-route` | `/route` | Compute routes between systems with algorithm selection |
| `evefrontier-lambda-scout-gates` | `/scout-gates` | Find gate-connected neighbors of a system |
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
  "from": "Nod",
  "to": "Brana",
  "algorithm": "a-star",
  "max_jump": 80.0,
  "avoid": ["H:2L2S"],
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
  "data": {
    "hops": 3,
    "gates": 2,
    "jumps": 1,
    "algorithm": "a-star",
    "route": ["Nod", "D:2NAS", "Brana"]
  }
}
```

**Error (HTTP 400/404/500):**
```json
{
  "type": "https://evefrontier.example/problems/unknown-system",
  "title": "Unknown System",
  "status": 404,
  "detail": "System 'InvalidName' not found in dataset. Did you mean: Nod, Brana?",
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
    "from": "Nod",
    "to": "Brana",
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
  from: "Nod",
  to: "Brana",
  algorithm: "a-star",
  max_jump: 80.0
};

const command = new InvokeCommand({
  FunctionName: "evefrontier-route",
  Payload: JSON.stringify(payload),
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
    "from": "Nod",
    "to": "Brana",
    "algorithm": "a-star"
  }'
```

### Scout Gates Lambda

Returns gate-connected neighbors of a system.

#### Request Schema

```json
{
  "system": "Nod"
}
```

**Fields:**
- `system` (required): System name to find neighbors for

#### Response Schema

**Success (HTTP 200):**
```json
{
  "content_type": "application/json",
  "data": {
    "system": "Nod",
    "system_id": 30011392,
    "count": 2,
    "neighbors": [
      {
        "name": "D:2NAS",
        "id": 30011393
      },
      {
        "name": "G:3OA0",
        "id": 30011394
      }
    ]
  }
}
```

#### Invocation Examples

**AWS SDK (Python):**
```python
payload = {"system": "Nod"}

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
  -d '{"system": "Nod"}'
```

### Scout Range Lambda

Returns systems within spatial range with optional temperature filtering.

#### Request Schema

```json
{
  "system": "Nod",
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
  "data": {
    "system": "Nod",
    "system_id": 30011392,
    "count": 3,
    "systems": [
      {
        "name": "D:2NAS",
        "id": 30011393,
        "distance_ly": 25.4,
        "min_temp_k": 30.0
      },
      {
        "name": "Brana",
        "id": 30011395,
        "distance_ly": 67.8,
        "min_temp_k": 45.2
      },
      {
        "name": "G:3OA0",
        "id": 30011394,
        "distance_ly": 88.3
      }
    ]
  }
}
```

**Notes:**
- Results are ordered by distance (closest first)
- `min_temp_k` field is included only if temperature data is available in the dataset
- KD-tree spatial index is used for efficient neighbor queries (sub-millisecond for typical datasets)

#### Invocation Examples

**AWS SDK (Python):**
```python
payload = {
    "system": "Nod",
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
    "system": "Nod",
    "radius": 100.0,
    "max_temperature": 50.0
  }'
```

### Cold Start & Initialization

Lambda functions bundle the EVE Frontier dataset and spatial index at build time for optimal
cold-start performance. Initialization happens once per Lambda container lifecycle.

#### Initialization Sequence

1. **Tracing Setup**: JSON-formatted CloudWatch Logs integration
2. **Database Loading**: Zero-copy deserialization from bundled bytes (`rusqlite::deserialize_bytes`)
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

Currently, Lambda functions do not require environment configuration. All data is bundled at build time.

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

