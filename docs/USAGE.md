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
- `--max-spatial-neighbours <N>` — maximum number of nearest neighbours considered per system when building spatial/hybrid graphs (defaults to `0`, meaning unlimited). Larger values increase search options but can increase planning time; set to `0` to consider all neighbours.
- `--optimize <distance|fuel>` — optimization objective for weighted planners (`distance` is default). Use `fuel` to prefer routes that minimize estimated fuel consumption (requires `--ship` and will fall back to distance with a warning if ship/loadout is not provided).

### Examples

- Download the latest dataset to the default location resolved by the CLI and report the path:

  ```pwsh
  evefrontier-cli download
  ```

- Download a specific dataset tag into a custom directory (helpful for tests or fixtures):

  ```pwsh
  evefrontier-cli download --data-dir docs/fixtures --dataset e6c3
  ```

> [!NOTE] The `download` subcommand always emits plain text regardless of `--format`.

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

When a `ship_data.csv` file is present in the dataset cache, `evefrontier-cli download` will print
the resolved path to the ship data alongside the dataset path for convenience.

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
- `--avoid-critical-state` — conservative heat-aware planning. This behavior is **enabled by default** when a ship is present, and you can opt out using `--no-avoid-critical-state` (CLI) or `avoid_critical_state=false` (API). When explicitly provided (`--avoid-critical-state`) the CLI will require `--ship` and will error if a ship is not supplied; when omitted the planner will only apply heat-aware avoidance if a ship is available or defaults are in use.
- `--max-spatial-neighbours <N>` — tune the spatial graph fan-out (default: `250`). Increasing this allows the planner to consider more long-range spatial links (may increase runtime and memory use); set to `0` for no truncation (unlimited neighbours) if you explicitly want that behaviour.
- `--optimize <distance|fuel>` — select the optimization target for weighted planners (`dijkstra`, `a-star`). `distance` selects shortest-distance routing; `fuel` selects routes that minimize estimated fuel consumption. Note: `--optimize fuel` **requires** `--ship` (and appropriate `--fuel-quality`, `--cargo-mass`, and `--dynamic-mass` flags when desired). If `--ship` is omitted the CLI will warn and fall back to distance optimization. The CLI default optimization is now **fuel** to provide more fuel-efficient out-of-the-box routes.

### Example: avoid critical heat hops (requires `--ship`)

```bash
# Plan a route avoiding spatial hops that would reach CRITICAL instant temperature
evefrontier-cli route --from "Nod" --to "Brana" --avoid-gates --avoid-critical-state --ship "Reflex"
```

- When `--avoid-critical-state` is active, the planner will conservatively omit any spatial jump that would cause the instantaneous temperature (local ambient + computed hop temperature delta) to meet or exceed the `CRITICAL` threshold. This check is performed per-hop and does not model residual cumulative heat across multiple hops (future work).
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

**Heat display:**

- The CLI displays **per-hop heat** (in game units) and emits warnings when canonical thresholds are
  exceeded (`HEAT_OVERHEATED`, `HEAT_CRITICAL`). Per-ship tolerances are not available from the
  canonical dataset and are not used for warnings.

Fuel-aware routing note:

- When `--optimize fuel` is used, the planner attempts to minimize estimated fuel consumption. In practice the planner treats gate transitions as zero-fuel hops (i.e., gates do not consume fuel) and computes hop fuel using the same formula as shown above. This means fuel optimization may prefer gate-based routes even if they are longer in distance. `--dynamic-mass` affects fuel projections for `--optimize fuel` when present.

Ship-capability per-hop limits 🔒

- When a ship (`--ship`) and `--fuel-quality`/loadout are provided, the planner **automatically computes a conservative per-hop `max_jump`** based on the ship's current fuel and (optionally) heat safety settings. This computed value is combined with any explicit `--max-jump` by taking the **minimum** so callers retain control.

- What is considered:
  - **Fuel-based limit**: derived from current fuel load and fuel quality (how many light-years can be covered with available fuel). 
  - **Heat-based limit**: applied **only** when `--avoid-critical-state` is enabled; an instantaneous hop that would reach or exceed the canonical `HEAT_CRITICAL` threshold is conservatively excluded.

- Practical effects:
  - The planner prunes spatial edges that exceed the effective per-hop `max_jump`, ensuring that only physically-capable single-jump edges are considered. This reduces the risk that `--max-spatial-neighbours` truncation hides valid ship-capable jumps.
  - **Gate transitions are unaffected** by per-hop `max_jump` and remain available when allowed.

 - Tuning:
 - Tuning:
  - Set `--max-spatial-neighbours` to a positive integer to limit the fan-out and reduce planning time/memory. Set it to `0` (the default) to disable truncation and consider all spatial neighbours — useful when ship-based per-hop limits could otherwise be hidden.
  - Note: for very large datasets an *unlimited* neighbour request without an explicit `--max-jump` radius can be extremely slow (it may require generating O(n^2) edges). In that case the planner will cap neighbours per-node and emit a warning; prefer passing `--max-spatial-neighbours` or `--max-jump` to control performance explicitly.
  - You may explicitly set `--max-jump` to override behavior (the explicit value is still combined with the computed ship limit via `min`).

- Debugging: the computed ship-based `max_jump` is logged at debug level (`tracing::debug!`) so you can inspect runtime decisions with `RUST_LOG=debug`.

Debugging example 🔍

To see the computed ship-based per-hop `max_jump` and other planner decisions, set `RUST_LOG=debug` and run a route command. Example:

```bash
RUST_LOG=debug ./target/debug/evefrontier-cli route --from "INN-6L4" --to "A3V-125" --ship "Reflex" --fuel-quality 10 --optimize fuel
```

Look for log lines like:

```
DEBUG planner: computed ship-based max_jump: fuel=Some(123.4), heat=Some(200.0), effective=Some(123.4)
```

These messages show the fuel-based and heat-based maxima and the final effective limit used by the planner.
- Very small non-zero hop heat values are shown as `"<0.01"` in the CLI to avoid misleading
  `0.00` readings. The CLI no longer shows a bracketed cumulative per-step heat value; residual heat
  and recommended wait times (when applicable) are included in the `json` output and Lambda
  responses for programmatic consumers.
 - Calibration is fixed at `1e-7` and is not user-configurable.

**Canonical thresholds:**

- Nominal: 30.0
- Overheated (warn): 90.0
- Critical (error): 150.0

These thresholds are applied uniformly; the per-ship `max_heat_tolerance` field was removed and is
not used for warnings.
 
Styled labels: the CLI renders warnings as short, high-visibility labels — ` OVERHEATED ` (black on
yellow) and ` CRITICAL ` (black on red) — which are also present as plain label strings in JSON
responses for programmatic consumers.

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

**Fuel units & rounding:**

- Fuel values shown in the CLI and returned in Lambda/JSON responses are integer units. Internally
  computed fuel floats are converted to integers by **ceiling (always round up)** before rendering
  or serializing. Tests and the JSON schema reflect this policy.

**Lambda notes:**

- When invoking the Lambda `route` handler, the `RouteRequest` no longer accepts a
  `heat_calibration` field — calibration is fixed server-side to `1e-7` and cannot be overridden
  by clients. The Lambda `RouteResponse` includes `FuelProjection` and `FuelSummary` fields as
  integers (e.g. `hop_cost`, `cumulative`, `remaining`, `total`) to maintain a stable, simple
  contract for API consumers.

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
evefrontier-cli index-build --data-dir docs/fixtures/minimal/static_data.db
```

Output: `docs/fixtures/minimal/static_data.db.spatial.bin`

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

| Code | Status          | Description                          |
| ---- | --------------- | ------------------------------------ |
| 0    | SUCCESS         | Index is fresh (matches dataset)     |
| 1    | STALE           | Index doesn't match dataset          |
| 2    | MISSING         | Spatial index file not found         |
| 3    | FORMAT_ERROR    | Legacy v1 format or corrupt file     |
| 4    | DATASET_MISSING | Dataset file not found               |
| 5    | ERROR           | Unexpected error during verification |

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

| Bit | Flag            | Description                     |
| --- | --------------- | ------------------------------- |
| 0   | HAS_TEMPERATURE | Index includes temperature data |
| 1   | HAS_METADATA    | v2 format with source metadata  |

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

### Ship Data & Lambda Bundling

Ship data (`ship_data.csv`) can be bundled into Lambda artifacts to provide an in-memory
`ShipCatalog` at cold start. This enables fast ship lookups for fuel projection and validation
without performing disk I/O at runtime.

- To include ship data in a Lambda build, enable the `bundle-ship-data` Cargo feature for the Lambda
  crate (e.g., `evefrontier-lambda-route`) and ensure `data/ship_data.csv` is present at build time.
- When bundled, `evefrontier-lambda-shared::init_runtime()` will parse the CSV and expose the
  catalog via `LambdaRuntime::ship_catalog()` for handlers to use.
- If bundling is not used, set the `EVEFRONTIER_SHIP_DATA` environment variable to the path of a
  `ship_data.csv` file to provide ship data at runtime.

Note: Failure to parse a bundled ship CSV will be logged during cold start, and the Lambda will
continue to operate without ship-based fuel projections.

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

The library test suite uses the bundled fixture located at `docs/fixtures/minimal/static_data.db`.
This fixture is pinned to the e6c3 dataset release and uses legacy system names (Nod, Brana, etc.)
for deterministic testing. The fixture is protected from accidental overwrites.

> [!NOTE] The test fixture uses system names from the e6c3 release (Nod, Brana, H:2L2S, etc.). The
> production dataset uses different names. See `docs/fixtures/README.md` for details.

### Local dataset overrides

For development and testing you can override the GitHub download by setting the
`EVEFRONTIER_DATASET_SOURCE` environment variable to a local path. The path may point to either a
`.db` file or a `.zip` archive containing the database. When set, `ensure_dataset` (and convenience
wrappers like `ensure_e6c3_dataset`) copy or extract the local file instead of contacting GitHub.

```pwsh
$env:EVEFRONTIER_DATASET_SOURCE = "docs/fixtures/minimal/static_data.db"
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

An additional footer line shows the **parameters applied** to the routing algorithm in a human-friendly form. Example:

- Parameters: Algorithm: a-star • Optimize: Fuel • Ship: Reflex • Fuel quality: 10% • Avoid critical state: Yes • Max spatial neighbors: 250 • Avoid gates: No

> Black hole systems (IDs 30000001–30000003) display a “Black Hole” badge on the status line.

### Using Custom Dataset Paths

For testing or using alternative datasets:

```rust
use evefrontier_lib::{ensure_dataset, DatasetRelease, load_starmap};
use std::path::Path;

// Use a specific dataset tag
let path = ensure_dataset(None, DatasetRelease::tag("e6c3"))?;

// Or point to a local fixture
let fixture_path = Path::new("docs/fixtures/minimal/static_data.db");
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

## MCP Server (stdio)

The CLI can run a Model Context Protocol (MCP) server over stdio using the `mcp` subcommand. This
mode is useful for integrating the EVE Frontier dataset with AI assistants (Claude Desktop, VS Code
Copilot, Cursor) or any client that speaks JSON-RPC over `stdin`/`stdout`.

Key points:

- Protocol: JSON-RPC 2.0 over newline-delimited messages on `stdout`/`stdin`.
- Logs: All logs and diagnostic output are written to `stderr` only so `stdout` remains a clean
  JSON-RPC channel.
- Dataset resolution: `--data-dir <PATH>` overrides `EVEFRONTIER_DATA_DIR`; if not provided the CLI
  will attempt to download or locate a dataset via the usual resolver.

### Basic usage

Run the MCP server using an explicit dataset fixture (recommended for development and tests):

```bash
# Run the MCP server (stdio transport)
evefrontier-cli mcp --data-dir ./docs/fixtures/minimal/static_data.db
```

When using an environment variable to set the dataset location:

```bash
export EVEFRONTIER_DATA_DIR="$HOME/.local/share/evefrontier/static_data.db"
evefrontier-cli mcp
```

Control logging verbosity via `RUST_LOG` (logs appear on stderr):

```bash
RUST_LOG=info evefrontier-cli mcp --data-dir ./docs/fixtures/minimal/static_data.db
```

The server responds to the standard MCP `initialize` handshake. The `initialize` response includes
capability keys for `tools` and `resources` (currently present as empty objects); tool discovery is
performed via `tools/list` and actual invocation via `tools/call`.

### Client configuration examples

Claude Desktop (example `claude_desktop_config.json`):

```json
{
  "name": "EVE Frontier (CLI MCP)",
  "command": "/path/to/evefrontier-cli",
  "args": ["mcp"],
  "env": {
    "EVEFRONTIER_DATA_DIR": "/absolute/path/to/static_data.db",
    "RUST_LOG": "info"
  }
}
```

VS Code extension / launch config (example `launch.json` snippet):

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "name": "Run EVE Frontier MCP",
      "type": "extensionHost",
      "request": "launch",
      "runtimeExecutable": "/path/to/evefrontier-cli",
      "args": ["mcp"],
      "env": {
        "EVEFRONTIER_DATA_DIR": "/absolute/path/to/static_data.db",
        "RUST_LOG": "info"
      }
    }
  ]
}
```

> NOTE: Different AI clients may have different ways to configure an external process. The essential
> properties are the executable path, `mcp` argument, and environment variables.

### Troubleshooting

- If the client fails to parse responses, ensure the CLI is launched **without** banners or messages
  on `stdout`. The MCP mode suppresses the ASCII banner and routes all logs to `stderr`.
- If the server cannot find the dataset, confirm the `--data-dir` path or set
  `EVEFRONTIER_DATA_DIR`. The CLI will attempt to download the dataset if `--data-dir` is not
  explicit.
- If the client disconnects unexpectedly, the server handles EOF and exits gracefully. Client-side
  tools must keep `stdout` open until finished reading JSON-RPC responses.

If you'd like, I can add a short example showing the JSON `initialize` exchange and a minimal Claude
Desktop configuration file in the `docs/` directory.

---
