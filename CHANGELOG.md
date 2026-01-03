# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

### Fixed

- **CLI** (`evefrontier-cli`)
  - GOAL step in `--format enhanced` now displays status line (min temp, planets, moons) like all
    other steps
  - Added "Black Hole" indicator for systems 30000001-30000003 which have no celestial bodies
  - Fuel information in enhanced format uses color coding: orange for hop cost, magenta for
    remaining fuel
  - Fixed padding alignment for planet/moon count labels to maintain consistent layout in enhanced
    output
  - Fixed panic when running dataset download under the CLI's Tokio runtime by running blocking
    dataset operations in a blocking region (`tokio::task::block_in_place`). This prevents dropping
    an internal reqwest runtime from inside an async context.
  - Add: show an estimation warning box in route footers when fuel or heat values are present
    to indicate values are approximate and may deviate by ±10%.
- **Library** (`evefrontier-lib`)
  - Fuel projections no longer consume fuel on gate hops; gate steps report zero fuel cost
  - Fix: avoid parsing checksum sidecar files (e.g., `*.sha256`) as ship CSVs. The downloader cache
    discovery now prefers `*_ship_data.csv` files and `ShipCatalog::from_path` resolves an adjacent
    `.csv` when given a `.sha256` sidecar.
  - Add: make ship CSV parsing tolerant of common header variants (e.g., `ShipName`, `Mass_kg`,
    `SpecificHeat_C`, `FuelCapacity_units`) and provide reasonable defaults for missing heat-related
    columns to preserve backward compatibility with older releases.
  - Tests: added unit tests covering cache discovery, sidecar resolution, and header-variant
    parsing.

### Added

- **MCP Server** (`evefrontier-mcp`)
  - **Ship Data & Fuel Calculations** (Library & CLI)
    - `ShipAttributes`, `ShipLoadout`, and `ShipCatalog` structs for ship management
    - Fuel cost calculator: `calculate_jump_fuel_cost()` with mass and distance-based formula
    - Add: `calculate_maximum_distance()` helper to compute maximum range from fuel load and quality
    - Docs: Add Rust doc comments for fuel APIs and clarify units/quality scaling
    - Tests: Add precise unit tests for `calculate_jump_fuel_cost()` and `calculate_maximum_distance()`
    - Route fuel projection: `calculate_route_fuel()` with static and dynamic mass modes
    - CLI flags: `--ship`, `--fuel-quality`, `--cargo-mass`, `--dynamic-mass` for route command
    - CLI convenience flag: `--list-ships` to display available ships from catalog
    - Ship data fixture (`docs/fixtures/ship_data.csv`) with Reflex and other ships
    - Default values: Reflex ship with 10% fuel quality applied automatically
    - Enhanced output mode displays fuel consumption per hop and total fuel required
    - Lambda support: Extended RouteRequest/RouteSummary with optional fuel projection fields
- **CLI** (`evefrontier-cli`)
  - Add `--avoid-critical-state` flag to `route` to conservatively avoid single spatial hops
    that would reach the canonical `CRITICAL` heat threshold; requires `--ship` and is
    documented in `docs/USAGE.md` and `docs/HEAT_MECHANICS.md`.
    - Dynamic mass recalculation mode where fuel weight decreases after each jump
    - 8 integration tests for ship catalog parsing, fuel calculation, and route aggregation
    - 3 specification tests for future ship data downloader features
    - Documentation in `docs/USAGE.md` with fuel calculation examples and formula reference
  - **Heat mechanics** (Library, CLI & Lambda)
      - Add `calculate_jump_heat()` and `HeatConfig` to `evefrontier-lib` for per-hop heat
        calculation
      - Add `HeatProjection` and `HeatSummary` types, included in CLI and Lambda outputs when a ship
        is provided
      - CLI: `route` command renders per-hop heat and route heat summary (enhanced and JSON formats)
      - Lambda: `route` handler attaches heat projections when `ship` is included in the request
      - Thresholds: warnings now use canonical absolute heat thresholds (Nominal=30.0 units,
        Overheated=90.0 units, Critical=150.0 units); per-ship `max_heat_tolerance` is no longer
        used or inferred from ship data
      - Tests: unit tests for formula, integration tests for per-hop and summary projections,
        CLI/Lambda integration tests
      - Documentation: `docs/USAGE.md` updated with heat quickstart examples and `docs/adrs/0015`
        augmented with implementation details
        - CLI: the `--heat-calibration` flag is not exposed to users; calibration is fixed to
          `1e-7` and not user-configurable. The previously-added `--cooling-mode` flag has been
          removed as it introduced unnecessary complexity; the library defaults to a conservative
          zone-based dissipation model.
        - Display & API changes: per-hop heat is shown in the CLI (two decimals) and very small non‑zero
          values are rendered as `"<0.01"` to avoid misleading `0.00` readings. The bracketed
          cumulative per-step heat value was removed from the CLI display; residual/wait information is
          still available in the JSON/Lambda response objects.
        - Lambda & schema: `RouteRequest` no longer accepts `heat_calibration`; calibration is fixed
          server-side to `1e-7` and cannot be overridden by clients. Fuel projection fields in Lambda
          responses are expressed as integers (e.g. `hop_cost`, `cumulative`, `remaining`, `total`) to
          maintain a stable, simple contract for API consumers. The JSON contract/specs have been
          updated accordingly.
        - Fuel rounding: fuel values are converted to integer units using **ceiling** (always round up).
          Tests and the JSON schema were updated to reflect this policy.
        - Tests: added in-game example tests (Reflex out/return and O1J-P35→UD6-P25) that validate
          the calibration and heat calculations

  - **Lambda Bundling & Ship Data**
    - Add support for bundling `ship_data.csv` into Lambda artifacts (feature: `bundle-ship-data`)
    - `evefrontier-lambda-shared::init_runtime()` now accepts bundled ship bytes and loads an
      in-memory `ShipCatalog` at cold start for fast ship lookups
    - `EVEFRONTIER_SHIP_DATA` env var can be used to provide a ship CSV at runtime when bundling is
      not used

  - **MCP Server** (`evefrontier-mcp`)
  - Model Context Protocol (MCP) server implementation for AI assistant integration
  - Stdio transport for local process communication (Claude Desktop, VS Code, Cursor)
  - JSON-RPC 2.0 message handling per MCP specification 2024-11-05
  - `McpServerState` with dataset loading, spatial index integration, and metadata access
  - Route planning tool (`route_plan`) with algorithm selection and constraint support
  - System query tools: `system_info`, `systems_nearby` (spatial KD-tree queries), `gates_from`
  - MCP resources: `dataset/info` (metadata), `algorithms` (catalog), `spatial-index/status`
  - MCP prompt templates: `route_planning`, `system_exploration`, `fleet_planning`,
    `safe_zone_finder`
  - Fuzzy system name matching with Jaro-Winkler suggestions
  - RFC 9457 ProblemDetails error responses for structured error handling
  - Temperature-aware spatial filtering for heat-constrained route planning
  - 54 tests passing (42 unit tests + 12 integration tests covering protocol, tools, resources,
    prompts)
  - Comprehensive usage documentation in `docs/USAGE.md` with configuration examples for Claude
    Desktop, VS Code, and Cursor
  - Tool schemas with full input/output documentation and example requests
  - JSON-RPC 2.0 protocol implementation with proper error codes and message handling
  - Prompt template system with argument validation and dynamic content generation

## [0.1.0-alpha.1] - 2025-12-30

### Added

- **Core Library** (`evefrontier-lib`)
  - Dataset downloader with GitHub release caching, atomic writes, and zip extraction
  - Starmap loader with runtime schema detection for multiple dataset formats
  - Graph builders for gate-only, spatial, and hybrid routing modes
  - Pathfinding algorithms: BFS (unweighted), Dijkstra (weighted), `A*` (heuristic-guided)
  - Route constraints: max jump distance, avoided systems, gate-free travel, temperature limits
  - KD-tree spatial index with postcard+zstd serialization and SHA-256 checksums
  - Temperature-aware nearest-neighbor queries with min_external_temp filtering
  - Fuzzy system name matching with Jaro-Winkler similarity suggestions
  - Output formatters: plain text, rich text, JSON, in-game note format

- **CLI** (`evefrontier-cli`)
  - `download` command for dataset acquisition with caching
  - `route` command with `--algorithm`, `--format`, `--max-jump`, `--avoid`, `--max-temp` flags
  - `index-build` command to precompute spatial index artifacts
  - `index-verify` command for spatial index freshness verification
  - Global options: `--data-dir`, `--format`, `--no-logo`, `--dataset`
  - Enhanced output format with system details (planets, moons, temperature)
  - Sci-fi styled CLI banner with color support

- **AWS Lambda Functions**
  - `evefrontier-lambda-route` - Route planning endpoint
  - `evefrontier-lambda-scout-gates` - Gate-connected neighbor lookup
  - `evefrontier-lambda-scout-range` - Spatial range queries with temperature filtering
  - `evefrontier-lambda-shared` - Common infrastructure (request validation, RFC 9457 errors)
  - Terraform module at `terraform/modules/evefrontier-lambda/`

- **Docker Microservices**
  - `evefrontier-service-route` - HTTP route planning service
  - `evefrontier-service-scout-gates` - HTTP gate lookup service
  - `evefrontier-service-scout-range` - HTTP spatial query service
  - `evefrontier-service-shared` - Axum 0.8 HTTP infrastructure, health checks, metrics
  - Multi-stage Dockerfiles with Distroless base images (~30MB containers)
  - Helm chart at `charts/evefrontier/` with deployments, services, ingress, probes
  - docker-compose.yml with Traefik reverse proxy for local development

- **Observability**
  - Prometheus metrics integration with `record_request_metrics()`
  - Structured JSON logging with RFC3339 timestamps
  - Health endpoints: `/health/live`, `/health/ready` with dependency checks
  - Grafana dashboard JSON and Prometheus alerting rules

- **CI/CD**
  - Release workflow with multi-arch builds (x86_64, aarch64)
  - Keyless cosign signing via GitHub OIDC
  - SBOM generation (CycloneDX format)
  - Docker release workflow with Trivy scanning
  - ADR governance workflow enforcing naming and immutability
  - Changelog guard requiring CHANGELOG.md updates
  - Nightly dependency outdated reports
  - Spatial index freshness CI verification

- **Documentation**
  - Comprehensive RELEASE.md with GPG signing and verification procedures
  - DEPLOYMENT.md for Lambda and Kubernetes deployment
  - ARCHITECTURE.md with Mermaid diagrams
  - OBSERVABILITY.md for metrics, logging, and alerting
  - USAGE.md with CLI examples and library API
  - 14 Architecture Decision Records (ADRs)

- **Testing**
  - 100+ unit and integration tests across all crates
  - Test fixture with real e6c3 dataset systems
  - CLI integration tests with assert_cmd
  - Lambda unit tests with shared test utilities

### Fixed

- Standardize GHCR container registry paths to `ghcr.io/scetrov/evefrontier-rs/*`
- Fix CLI integration tests for GitHub API rate limiting
- Fix temperature calculation bugs and formula accuracy
- Fix JSON output format pollution from tracing logs
- Fix race condition in dataset download tests

### Known Issues

- `kiddo 5.2.3` depends on yanked `cmov 0.3.1` (not a security vulnerability, just yanked)
  - Waiting for upstream fix; cargo audit passes with warning only
