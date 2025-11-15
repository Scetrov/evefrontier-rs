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

- Filter by maximum system temperature:

  ```bash
  evefrontier-cli route --from "Y:170N" --to "BetaTest" --max-temp 300.0
  ```

- Filter by minimum external temperature (at the outermost celestial body):

  ```bash
  evefrontier-cli route --from "Y:170N" --to "BetaTest" --min-temp 200.0
  ```
  Systems with computed `min_external_temp` below the threshold are excluded. Systems that
  do not expose this value are treated as allowed to avoid over-pruning.

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
- `--max-temp <KELVIN>` — constrain the maximum temperature of systems along the route. Systems that
  do not expose a temperature reading are treated as safe.
- `--min-temp <KELVIN>` — constrain the minimum external temperature (Kelvin) at the outermost
  celestial body in each system. Systems without a computed value are treated as allowed.

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
  fixture data or custom paths. `ensure_c3e6_dataset` is still available as a shorthand for
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
wrappers like `ensure_c3e6_dataset`) copy or extract the local file instead of contacting GitHub.

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
    ensure_c3e6_dataset, load_starmap, plan_route,
    RouteRequest, RouteAlgorithm, RouteConstraints,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Ensure dataset is available (downloads if needed)
    let dataset_path = ensure_c3e6_dataset(None)?;

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

