# EVE Frontier Rust Workspace

[![Crates.io](https://img.shields.io/crates/v/evefrontier-lib.svg)](https://crates.io/crates/evefrontier-lib)
[![Docs.rs](https://docs.rs/evefrontier-lib/badge.svg)](https://docs.rs/evefrontier-lib)
[![Build Status](https://github.com/Scetrov/evefrontier-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Scetrov/evefrontier-rs/actions/workflows/ci.yml)
[![GitHub Release](https://img.shields.io/github/v/release/Scetrov/evefrontier-rs)](https://github.com/Scetrov/evefrontier-rs/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Maintenance](https://img.shields.io/badge/maintenance-active-green.svg)](#)

A comprehensive Rust workspace for working with EVE Frontier static datasets, providing pathfinding
and navigation tools for the game world.

## 📦 Workspace Structure

This repository contains multiple crates organized as a Cargo workspace:

### Core Library

- **`evefrontier-lib`** — Reusable library providing:
  - Dataset downloading and caching
  - Starmap loading with schema detection
  - Graph construction for gate, spatial, and hybrid routing
  - Pathfinding algorithms (BFS, Dijkstra, A\*)
  - KD-tree spatial indexing for efficient neighbor queries
  - Temperature-aware routing constraints

### Applications

- **`evefrontier-cli`** — Command-line interface exposing:
  - `download` — Download and cache dataset releases
  - `route` — Compute routes between systems with advanced options
  - `index-build` — Precompute spatial index for faster queries

### AWS Lambda Crates

- **Lambda function crates:**
  - **`evefrontier-lambda-route`** — Route planning endpoint
  - **`evefrontier-lambda-scout-gates`** — Gate-connected neighbors query
  - **`evefrontier-lambda-scout-range`** — Systems within jump range query
- **Shared/infrastructure crate:**
  - **`evefrontier-lambda-shared`** — Common Lambda infrastructure (runtime, error handling,
    tracing)

### Docker Microservices

- **Service crates (Docker/Kubernetes deployment):**
  - **`evefrontier-service-route`** — HTTP route planning service
  - **`evefrontier-service-scout-gates`** — HTTP gate neighbors service  
  - **`evefrontier-service-scout-range`** — HTTP range query service
- **Shared/infrastructure crate:**
  - **`evefrontier-service-shared`** — Common HTTP infrastructure (axum handlers, health checks,
    request/response types)

> [!NOTE]
> The workspace contains 10 crates in total: the core library (`evefrontier-lib`), CLI
> (`evefrontier-cli`), 3 Lambda function crates, 1 shared Lambda infrastructure crate, 3 Docker
> microservice crates, and 1 shared HTTP infrastructure crate.

### Documentation

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — System architecture diagrams and data flows
- [`docs/INITIAL_SETUP.md`](docs/INITIAL_SETUP.md) — Configuration and data path resolution
- [`docs/USAGE.md`](docs/USAGE.md) — Comprehensive usage examples
- [`docs/adrs/`](docs/adrs/) — Architectural Decision Records

## Getting started

1. Ensure the Rust toolchain pinned in [`.rust-toolchain`](.rust-toolchain) is installed:

```bash
rustup toolchain install $(cat .rust-toolchain)
rustup override set $(cat .rust-toolchain)
```

2. Build the workspace:

```bash
cargo build --workspace
# Or use Nx for orchestrated builds with caching:
pnpm nx run-many --target=build --all
```

3. Run tests:

```bash
cargo test --workspace
# Or with Nx task orchestration:
pnpm nx run-many --target=test --all
```

4. Run the CLI (it will download the dataset automatically on first use):

```bash
# Download the dataset
cargo run -p evefrontier-cli -- download

# Compute a route
cargo run -p evefrontier-cli -- route --from "ER1-MM7" --to "ENQ-PB6"

# Or install globally
cargo install --path crates/evefrontier-cli
evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6"
```

The CLI automatically downloads the latest dataset on first use. Use `--data-dir` to specify a
custom location, or set `EVEFRONTIER_DATA_DIR`. See [`docs/INITIAL_SETUP.md`](docs/INITIAL_SETUP.md)
for data path resolution details.

## Developer Tooling (pnpm 10 + Nx)

This project uses **Nx** for task orchestration with intelligent caching and dependency management.

### Setup

Requires Node 24 (per `.nvmrc`) and pnpm 10+:

```bash
nvm use  # Switches to Node 24
npm i -g pnpm@10
pnpm install
```

### Nx Task Orchestration

Nx automatically runs builds before tests, caches task outputs, and runs tasks in the correct order:

```bash
# Run tests (automatically builds first)
pnpm nx run evefrontier-lib:test

# Run tests for all projects
pnpm nx run-many --target=test --all

# Run only affected projects' tests
pnpm nx affected --target=test

# Run clippy across all crates
pnpm nx run-many --target=clippy --all
```

### Available Scripts

Use package.json scripts for consistency:

```bash
pnpm run build    # Build all crates
pnpm run test     # Test all crates
pnpm run clippy   # Lint all crates
pnpm run lint:md  # Lint markdown files
```

### Troubleshooting

If Nx daemon issues occur:

```bash
NX_DAEMON=false pnpm nx run-many --target=test --all
pnpm nx reset
```

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for complete Nx documentation and task orchestration
details.

## CLI Usage

### Output Formats

The CLI supports multiple output formats for the `route` subcommand via the `--format` flag:

- **JSON** for machine-readable output and integrations:

```bash
evefrontier-cli --format json route --from "ER1-MM7" --to "ENQ-PB6"
```

- **Basic** for minimal path-only output with +/|/- prefixes:

```bash
evefrontier-cli --format basic route --from "ER1-MM7" --to "ENQ-PB6"
```

- **Note** for in-game EVE notes with clickable system links:

```bash
evefrontier-cli --format note route --from "ER1-MM7" --to "ENQ-PB6"
```

- **Text** (default) for human-readable output, or **Rich** for Markdown-style formatting

### Routing Options

The `route` subcommand supports advanced pathfinding options:

- **Algorithm selection** (`--algorithm <bfs|dijkstra|a-star>`):

```bash
evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --algorithm dijkstra
```

- **Maximum jump distance** (`--max-jump <LIGHT-YEARS>`):

```bash
evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --max-jump 80.0
```

- **System avoidance** (`--avoid <SYSTEM>`, repeatable):

```bash
evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --avoid "IFM-228"
```

- **Gate-free routing** (`--avoid-gates`):

```bash
evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --avoid-gates
```

- **Temperature limit for spatial jumps** (`--max-temp <KELVIN>`):

```bash
evefrontier-cli route --from "ER1-MM7" --to "ENQ-PB6" --max-temp 5000.0
```

Prevents spatial jumps to systems with high external temperatures. Gate jumps are unaffected.

See [`docs/USAGE.md`](docs/USAGE.md) for comprehensive documentation and additional examples.

### Spatial Index

Precompute a KD-tree spatial index for faster neighbor queries:

```bash
evefrontier-cli index-build
# Creates {database_path}.spatial.bin
```

The index enables efficient nearest-neighbor and radius queries with temperature filtering.

## Library API Highlights

- `ensure_dataset` — resolves the dataset path using CLI arguments, environment variables, or
  platform-specific defaults, downloads the requested dataset release (latest by default), and
  caches it under the OS cache directory. Returns both database and spatial index paths.
- `load_starmap` — loads systems and jumps from the SQLite database with basic schema detection.
- `plan_route` — converts system names into IDs, validates routing options, and plans a route using
  BFS, Dijkstra, or A\* while applying distance, avoidance, gate, and temperature constraints.
  Lower-level helpers such as `build_graph`/`find_route_bfs` remain available when needed.
- `build_spatial_index` / `load_spatial_index` — create and load KD-tree spatial indexes for
  efficient neighbor queries with temperature awareness.

Example:

```rust
use evefrontier_lib::{ensure_dataset, load_starmap, plan_route, RouteRequest, RouteConstraints, RouteAlgorithm, DatasetRelease};
use rusqlite::Connection;

let paths = ensure_dataset(None, DatasetRelease::latest())?;
let conn = Connection::open(&paths.database)?;
let starmap = load_starmap(&conn)?;

let request = RouteRequest {
    start: "ER1-MM7".to_string(),
    goal: "ENQ-PB6".to_string(),
    algorithm: RouteAlgorithm::AStar,
    constraints: RouteConstraints {
        max_jump: Some(80.0),
        ..Default::default()
    },
};

let plan = plan_route(&starmap, &request)?;
println!("Route: {} jumps", plan.jumps.len());
```

See [`docs/TODO.md`](docs/TODO.md) for the comprehensive backlog covering the downloader, advanced
pathfinding options, deployment infrastructure, and additional tooling.

## AWS Lambda Deployment

The workspace includes AWS Lambda functions for serverless deployment:

- **Route planning** (`evefrontier-lambda-route`) — POST endpoint accepting start/end systems
- **Gate neighbors** (`evefrontier-lambda-scout-gates`) — GET gate-connected systems
- **Range neighbors** (`evefrontier-lambda-scout-range`) — GET systems within jump range

All Lambda functions use the same `evefrontier-lib` core and include bundled datasets for fast cold
starts.

See [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md) for Terraform deployment instructions.

## Docker & Kubernetes Deployment

The workspace also includes containerized microservices for Docker Compose and Kubernetes:

### Docker Compose (Local Development)

```bash
# Start all services with Traefik reverse proxy
docker compose up -d

# Test the services
curl http://localhost/api/v1/route -X POST \
  -H "Content-Type: application/json" \
  -d '{"origin": "Nod", "destination": "Brana"}'
```

### Kubernetes with Helm

```bash
# Install from local chart
helm install evefrontier ./charts/evefrontier

# With custom values
helm install evefrontier ./charts/evefrontier -f values.yaml
```

The microservices use:
- **Base image**: `gcr.io/distroless/cc-debian12` (~20MB runtime)
- **Security**: Non-root user, read-only filesystem, dropped capabilities
- **Ingress**: Traefik IngressRoute with optional rate limiting and CORS

See [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md) for comprehensive Docker/Kubernetes documentation.

## Contributing

Please review [`CONTRIBUTING.md`](CONTRIBUTING.md) and the ADRs before submitting changes. All code
changes must add an entry to [`CHANGELOG.md`](CHANGELOG.md).
