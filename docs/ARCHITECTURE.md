# EVE Frontier Architecture

This document provides visual documentation of the EVE Frontier Rust workspace architecture,
including component relationships, data flows, and key operational sequences.

All diagrams use [Mermaid](https://mermaid.js.org/) syntax and render natively in GitHub and VS
Code.

## Table of Contents

- [EVE Frontier Architecture](#eve-frontier-architecture)
  - [Table of Contents](#table-of-contents)
  - [Design Patterns](#design-patterns)
    - [Strategy Pattern (Routing Algorithms)](#strategy-pattern-routing-algorithms)
    - [Predicate Functions (Edge Filtering)](#predicate-functions-edge-filtering)
    - [Module Organization (Single Responsibility)](#module-organization-single-responsibility)
  - [Component Overview](#component-overview)
    - [Component Descriptions](#component-descriptions)
  - [Module Dependencies](#module-dependencies)
    - [Module Responsibilities](#module-responsibilities)
  - [Data Flows](#data-flows)
    - [Dataset Download Flow](#dataset-download-flow)
    - [Starmap Load Flow](#starmap-load-flow)
    - [Route Planning Flow](#route-planning-flow)
  - [Sequence Diagrams](#sequence-diagrams)
    - [CLI Route Command](#cli-route-command)
    - [Lambda Cold-Start](#lambda-cold-start)
  - [See Also](#see-also)

---

## Design Patterns

The codebase employs several design patterns to maintain SOLID principles and clean architecture:

### Strategy Pattern (Routing Algorithms)

The `routing/planner.rs` module implements the Strategy pattern for algorithm selection:

```rust
pub trait RoutePlanner: Send + Sync {
    fn algorithm(&self) -> RouteAlgorithm;
    fn find_path(&self, graph: &Graph, starmap: Option<&Starmap>,
                 start: SystemId, goal: SystemId,
                 constraints: &SearchConstraints) -> Option<Vec<SystemId>>;
    fn requires_spatial_index(&self) -> bool { false }
}
```

Three implementations (`BfsPlanner`, `DijkstraPlanner`, `AStarPlanner`) encapsulate algorithm-specific
logic. The `select_planner()` factory function returns the appropriate planner based on the request.

**Benefits:**
- **Open/Closed Principle**: New algorithms can be added without modifying `plan_route()`
- **Single Responsibility**: Each planner handles only its algorithm
- **Testability**: Planners can be unit tested in isolation

### Predicate Functions (Edge Filtering)

The `path.rs` module uses composable predicate functions for edge filtering:

```rust
fn edge_meets_distance_limit(edge: &Edge, max_jump: Option<f64>) -> bool;
fn edge_meets_gate_policy(edge: &Edge, avoid_gates: bool) -> bool;
fn system_meets_avoidance(target: SystemId, avoided: &HashSet<SystemId>) -> bool;
fn system_meets_temperature(edge: &Edge, starmap: Option<&Starmap>,
                            target: SystemId, max_temperature: Option<f64>) -> bool;
fn hop_meets_heat_safety(edge: &Edge, target: SystemId, ctx: &HeatSafetyContext) -> bool;
```

**Benefits:**
- **Composability**: Predicates combine cleanly in `PathConstraints::allows()`
- **Testability**: Each predicate can be unit tested independently
- **Extensibility**: New constraints can be added as new predicates

### Module Organization (Single Responsibility)

Large modules are split into focused submodules:

- **`ship/`**: `attributes.rs`, `fuel.rs`, `heat.rs`, `catalog.rs`, `constants.rs`
- **`routing/`**: `mod.rs` (orchestration), `planner.rs` (algorithms)

---

## Component Overview

High-level view of the Rust workspace crates and their relationships with external systems.

```mermaid
graph LR
    subgraph External["External Systems"]
        GH[("GitHub Releases<br/>(evefrontier_datasets)")]
        DB[("SQLite Database<br/>(static_data.db)")]
        IDX[("Spatial Index<br/>(.spatial.bin)")]
    end

    subgraph Library["evefrontier-lib"]
        LIB[Core Library]
    end

    subgraph Consumers["Consumer Crates"]
        CLI[evefrontier-cli]

        subgraph Lambdas["AWS Lambda Functions"]
            SHARED[evefrontier-lambda-shared]
            ROUTE[evefrontier-lambda-route]
            GATES[evefrontier-lambda-scout-gates]
            RANGE[evefrontier-lambda-scout-range]
        end
    end

    GH -->|download| LIB
    DB -->|load| LIB
    IDX -->|load| LIB

    LIB --> CLI
    LIB --> SHARED
    SHARED --> ROUTE
    SHARED --> GATES
    SHARED --> RANGE
```

### Component Descriptions

| Component                        | Type    | Description                                                                             |
| -------------------------------- | ------- | --------------------------------------------------------------------------------------- |
| `evefrontier-lib`                | Library | Core business logic: dataset handling, graph construction, routing algorithms           |
| `evefrontier-cli`                | Binary  | Command-line interface with Clap argument parsing and output formatting                 |
| `evefrontier-lambda-shared`      | Library | Shared Lambda infrastructure: tracing, RFC 9457 problem details, runtime initialization |
| `evefrontier-lambda-route`       | Binary  | Lambda handler for route planning endpoint                                              |
| `evefrontier-lambda-scout-gates` | Binary  | Lambda handler for gate-connected neighbors query                                       |
| `evefrontier-lambda-scout-range` | Binary  | Lambda handler for spatial range queries                                                |

---

## Module Dependencies

Internal module structure of `evefrontier-lib` showing how modules depend on each other.

```mermaid
graph TD
    subgraph "evefrontier-lib"
        error[error.rs<br/>Error types]

        github[github.rs<br/>GitHub downloader]
        dataset[dataset.rs<br/>Path resolution]
        db[db.rs<br/>SQLite loader]

        spatial[spatial.rs<br/>KD-tree index]
        graph[graph.rs<br/>Graph builders]
        path[path.rs<br/>Pathfinding + Predicates]

        subgraph routing["routing/"]
            routing_mod[mod.rs<br/>Route orchestration]
            planner[planner.rs<br/>Strategy pattern]
        end

        subgraph ship["ship/"]
            ship_mod[mod.rs<br/>Re-exports]
            attributes[attributes.rs<br/>ShipAttributes]
            fuel[fuel.rs<br/>Fuel calculations]
            heat[heat.rs<br/>Heat calculations]
            catalog[catalog.rs<br/>Ship catalog]
        end

        output[output.rs<br/>Formatting]
        temperature[temperature.rs<br/>Temp calculations]
    end

    dataset --> github
    dataset --> db
    db --> graph
    graph --> path
    path --> routing_mod
    planner --> path

    spatial --> graph
    spatial --> routing_mod

    temperature --> graph
    temperature --> routing_mod

    error --> github
    error --> dataset
    error --> db
    error --> routing_mod

    routing_mod --> output
    routing_mod --> planner

    ship_mod --> fuel
    ship_mod --> heat
    ship_mod --> attributes
    ship_mod --> catalog
    fuel --> path
    heat --> path
```

### Module Responsibilities

| Module               | Responsibility                               | Key Exports                                                     |
| -------------------- | -------------------------------------------- | --------------------------------------------------------------- |
| `error.rs`           | Centralized error handling                   | `Error`, `Result`                                               |
| `github.rs`          | Download datasets from GitHub releases       | `download_dataset_with_tag`, `DatasetRelease`                   |
| `dataset.rs`         | Resolve and ensure dataset paths             | `ensure_dataset`, `ensure_e6c3_dataset`, `DatasetPaths`         |
| `db.rs`              | Load SQLite into Starmap                     | `load_starmap`, `load_starmap_from_connection`, `Starmap`       |
| `graph.rs`           | Build adjacency graphs                       | `build_gate_graph`, `build_spatial_graph`, `build_hybrid_graph` |
| `path.rs`            | Pathfinding algorithms + edge predicates     | `find_route_bfs`, `find_route_dijkstra`, `PathConstraints`      |
| `routing/mod.rs`     | High-level route orchestration               | `plan_route`, `RouteRequest`, `RoutePlan`                       |
| `routing/planner.rs` | Strategy pattern for algorithm selection     | `RoutePlanner` trait, `BfsPlanner`, `DijkstraPlanner`, `AStarPlanner` |
| `ship/mod.rs`        | Ship attributes and calculations             | `ShipAttributes`, `ShipLoadout`, `ShipCatalog`                  |
| `ship/fuel.rs`       | Fuel consumption calculations                | `calculate_jump_fuel_cost`, `calculate_route_fuel`              |
| `ship/heat.rs`       | Heat generation calculations                 | `calculate_jump_heat`, `HeatConfig`                             |
| `spatial.rs`         | KD-tree spatial index                        | `SpatialIndex`, `SpatialIndex::build`, `try_load_spatial_index` |
| `output.rs`          | Output formatting                            | `RouteSummary`, `RouteStep`, `RouteOutputKind`                  |
| `temperature.rs`     | Temperature constraint helpers               | Temperature filtering predicates                                |

---

## Data Flows

### Dataset Download Flow

How the system ensures dataset availability, with caching and atomic writes.

```mermaid
flowchart TD
    START([ensure_dataset called]) --> CHECK{Dataset exists<br/>at target path?}

    CHECK -->|Yes| RETURN([Return DatasetPaths])
    CHECK -->|No| RESOLVE[Resolve release tag]

    RESOLVE --> FETCH[Fetch from GitHub<br/>Releases API]
    FETCH --> DOWNLOAD[Download asset<br/>to temp file]
    DOWNLOAD --> EXTRACT{Is ZIP archive?}

    EXTRACT -->|Yes| UNZIP[Extract .db file]
    EXTRACT -->|No| COPY[Use file directly]

    UNZIP --> ATOMIC[Atomic rename<br/>to target path]
    COPY --> ATOMIC

    ATOMIC --> CACHE[Store in OS<br/>cache directory]
    CACHE --> RETURN
```

### Starmap Load Flow

How SQLite data transforms into the in-memory Starmap with schema detection.

```mermaid
flowchart TD
    START([load_starmap called]) --> OPEN[Open SQLite<br/>connection]

    OPEN --> DETECT{Detect schema}
    DETECT -->|SolarSystems table| NEW[Use new schema<br/>SolarSystems + Jumps]
    DETECT -->|mapSolarSystems| LEGACY[Use legacy schema<br/>mapSolarSystems]

    NEW --> QUERY[Query systems<br/>and jump gates]
    LEGACY --> QUERY

    QUERY --> BUILD[Build Starmap struct]
    BUILD --> INDEX[Create name→ID<br/>lookup index]
    INDEX --> FUZZY[Initialize fuzzy<br/>matching index]

    FUZZY --> RETURN([Return Starmap])
```

### Route Planning Flow

How a route request transforms into a computed path using the Strategy pattern.

```mermaid
flowchart TD
    START([plan_route called]) --> VALIDATE[Resolve system names<br/>& validate constraints]

    VALIDATE --> SELECT_GRAPH{Select graph type}

    SELECT_GRAPH -->|BFS| GATE[build_gate_graph]
    SELECT_GRAPH -->|Dijkstra| HYBRID[build_hybrid_graph]
    SELECT_GRAPH -->|A*| SPATIAL[build_spatial_graph<br/>or hybrid]

    GATE --> SELECT_PLANNER[select_planner<br/>Strategy pattern]
    HYBRID --> SELECT_PLANNER
    SPATIAL --> SELECT_PLANNER

    SELECT_PLANNER --> BFS_P{BfsPlanner}
    SELECT_PLANNER --> DIJ_P{DijkstraPlanner}
    SELECT_PLANNER --> ASTAR_P{AStarPlanner}

    BFS_P --> FIND[planner.find_path]
    DIJ_P --> FIND
    ASTAR_P --> FIND

    FIND --> VALIDATE_EDGES[Validate route edges<br/>for heat safety]
    VALIDATE_EDGES --> PLAN[Construct RoutePlan]

    PLAN --> RETURN([Return RoutePlan])
```

---

## Sequence Diagrams

### CLI Route Command

Time-ordered sequence of a user running `evefrontier-cli route "Nod" "Brana"`.

```mermaid
sequenceDiagram
    actor User
    participant CLI as evefrontier-cli
    participant Dataset as dataset.rs
    participant DB as db.rs
    participant Routing as routing.rs
    participant Output as output.rs

    User->>CLI: route "Nod" "Brana"
    CLI->>CLI: Parse arguments (Clap)

    CLI->>Dataset: ensure_dataset()
    Dataset-->>CLI: DatasetPaths

    CLI->>DB: load_starmap(path)
    DB-->>CLI: Starmap

    CLI->>Routing: plan_route(starmap, request)
    Routing->>Routing: Resolve system names
    Routing->>Routing: Build graph
    Routing->>Routing: Run pathfinder
    Routing-->>CLI: RoutePlan

    CLI->>Output: format_route(plan)
    Output-->>CLI: Formatted string

    CLI-->>User: Display route
```

### Lambda Cold-Start

Time-ordered sequence of Lambda initialization with bundled data.

```mermaid
sequenceDiagram
    participant AWS as AWS Lambda
    participant Handler as Lambda Handler
    participant Runtime as LambdaRuntime
    participant SQLite as rusqlite
    participant Spatial as SpatialIndex

    AWS->>Handler: First invocation
    Handler->>Runtime: get_runtime()

    alt Not initialized
        Runtime->>Runtime: init_runtime()
        Runtime->>Runtime: include_bytes!(db)
        Runtime->>SQLite: Connection::open_in_memory()
        SQLite->>SQLite: deserialize_bytes()
        Runtime->>Runtime: load_starmap_from_connection()

        Runtime->>Runtime: include_bytes!(index)
        Runtime->>Spatial: load_from_bytes()
        Spatial->>Spatial: Decompress (zstd)
        Spatial->>Spatial: Deserialize (postcard)

        Runtime->>Runtime: Store in OnceLock
    end

    Runtime-->>Handler: &LambdaRuntime
    Handler->>Handler: Process request
    Handler-->>AWS: Response

    Note over AWS,Spatial: Subsequent invocations<br/>reuse initialized runtime
```

---

## See Also

- **[Usage Guide](USAGE.md)** - CLI commands, library API examples, Lambda invocation
- **[Deployment Guide](DEPLOYMENT.md)** - AWS Lambda deployment with Terraform
- **[ADR 0002: Workspace Structure](adrs/0002-workspace-structure.md)** - Library + CLI architecture
  decision
- **[ADR 0006: Software Components](adrs/0006-software-components.md)** - Toolchain and component
  documentation
- **[ADR 0009: KD-tree Spatial Index](adrs/0009-kd-tree-spatial-index.md)** - Spatial index design
- **[README](../README.md)** - Project overview and quick start

---

_Last updated: 2025-01-12_
