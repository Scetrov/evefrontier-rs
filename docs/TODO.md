# Project TODOs

This TODO list captures the remaining work required to implement the EVE Frontier CLI, library, AWS
Lambda functions, and supporting infrastructure described throughout the documentation and ADRs.
Tasks are grouped by domain; checkboxes track completion status.

## 📋 ADR Alignment Status

**Last Updated:** 2025-12-30 (see `docs/adr-alignment-report_2025-12-30.md` for detailed analysis)

**Overall Alignment:** ✅ **93%** (14/15 ADRs fully implemented; 1 deferred with clear plan)

- ✅ **ADRs 0001-0014:** Fully or substantially implemented
- ⚠️ **ADR 0015 (Fuel Calculations):** Proposed; deferred pending:
  - Community validation of fuel formula
  - Heat mechanic research and confirmation
  - Ship data CSV availability from evefrontier_datasets repository

**Recommendations:**

- Create ADR 0016 documenting web-based starmap explorer architecture (currently deferred)
- Create ADR documenting NX orchestration strategy (implicit in current structure)
- Create "Heat Mechanics Research" ADR once formula validated (prerequisite for ADR 0015 Phase 2)

For detailed alignment report, see `docs/adr-alignment-report_2025-12-30.md`

## High Priority - Container Repository

- [x] Container repository put images into the repo via `ghcr.io/scetrov/evefrontier-rs/*` instead
      of using `rslater-cs` or `scetrov/` (without evefrontier-rs).
- [x] All instances of incorrect GHCR references should be corrected

## 🔥 High Priority - Dependency Issues

- [ ] **BLOCKED**: `kiddo 5.2.3` depends on yanked `cmov 0.3.1`
  - Issue: `cmov 0.3.1` was yanked after `kiddo 5.2.3` release (Dec 2024)
  - `cmov 0.4.3` exists but is a major version bump (API may have changed)
  - Waiting for `kiddo` maintainer to release 5.2.4+ with updated `cmov` dependency
  - **Workaround**: Removed `--deny warnings` flag from audit; default `cargo audit` behavior fails
    on security advisories but allows yanked/unmaintained warnings
  - **Tracking**: Monitor [Kiddo](https://github.com/sdd/kiddo/issues) for `cmov` update
  - **Impact**: Pre-commit hooks fail on security vulnerabilities (RustSec advisories) but allow
    yanked dependencies as warnings

## 🔥 High Priority - Test & Documentation Fixes

- [x] **CRITICAL**: Ensure test fixture protection is maintained
  - The test fixture now uses 8 real systems from e6c3 (Nod, Brana, D:2NAS, G:3OA0, H:2L2S, J:35IA,
    Y:3R7E, E1J-M5G)
  - Tests use real system names and pass in CI with actual production data structure
  - All tests updated to reference real systems: Nod, Brana, H:2L2S, etc.
  - Documentation updated in `README.md` and `docs/USAGE.md` to use real system names
- [x] Rebuild test fixture database using real e6c3 dataset (completed with
      `extract_fixture_from_dataset.py`)
  - Fixture includes Nod, Brana, and all systems connected by gates or within 80 ly of Brana
  - 8 systems, 12 jump gates, 26 planets, 43 moons extracted from real e6c3 data
  - Schema bug fixed: loader now correctly handles `constellationId`/`regionId` (camelCase) from
    e6c3
- [x] Add `.gitignore` entry for `*.db.release` marker files (ephemeral download metadata;
      implemented as a global pattern)
- [x] Add fixture protection to prevent accidental overwrites:
  - [x] Document in `docs/fixtures/README.md` that fixture should not be replaced with downloads
    - [x] Library guard rejects download targets that resolve to
          `docs/fixtures/minimal_static_data.db`
- [x] Document `.vscode/mcp.json` in README or CONTRIBUTING guide (GitHub Copilot MCP server config;
      clarify if required or optional for developers)
- [x] Add CI validation step that runs example commands from README/USAGE docs to ensure they
      continue working

## Workspace & Tooling

- [x] Establish the Cargo workspace layout with `crates/evefrontier-lib`, `crates/evefrontier-cli`,
      and Lambda crates (for example `crates/evefrontier-lambda-route`,
      `crates/evefrontier-lambda-scout-gates`, `crates/evefrontier-lambda-scout-range`). Ensure
      shared code lives in the library crate.
  - `evefrontier-lambda-shared` crate contains common Lambda infrastructure
  - Three Lambda crates added: `evefrontier-lambda-route`, `evefrontier-lambda-scout-gates`,
    `evefrontier-lambda-scout-range`
  - Library crate extended with `load_starmap_from_connection()` and
    `SpatialIndex::load_from_bytes()` for Lambda use
- [x] Configure Nx to orchestrate Rust, lint, and security tasks (align with
      [ADR 0006](adrs/0006-software-components.md) and `.github/copilot-instructions.md`). Define
      tasks for `build`, `test`, `lint`, `clippy`, `audit`, and dependency update reporting.
  - Created nx.json with target defaults and caching for all specified tasks
  - Created project.json for all 6 Rust crates with executor configurations
  - Wired Nx targets into pre-commit hooks (Cargo.toml) and CI workflows (.github/workflows/ci.yml)
  - Fixed root project recursion issue with --exclude evefrontier-rs flag
- [x] Scaffold Node/Nx workspace: add `package.json`, `pnpm-lock.yaml`, `nx.json`, and project
      targets for Rust crates and scripts (ADR 0006 & 0007 alignment).
  - Created package.json with pnpm 10.0.0 engines and Nx scripts
  - Created pnpm-workspace.yaml (required for pnpm 10)
  - Generated pnpm-lock.yaml with 211 packages
  - Configured nx.json with cacheable targets and input patterns
  - Added 6 project.json files for each Rust crate
- [x] Add `package.json`, `pnpm-lock.yaml`, and Nx project configuration. Document developer
      commands in `CONTRIBUTING.md` and `docs/USAGE.md`.
  - Documented pnpm 10 setup instructions in CONTRIBUTING.md (Tooling requirements section)
  - Added Developer Tooling section to README.md with Nx usage and troubleshooting
  - Included daemon troubleshooting (NX_DAEMON=false, pnpm nx reset)
- [x] Add CI workflow enforcing ADR filename pattern (`^\\d{4}-.+\\.md$`) and immutability (reject
      edits to historical ADRs except via explicit override label) per
      [ADR 0001](adrs/0001-use-nygard-adr.md).
  - Created `.github/workflows/adr-governance.yml` with comprehensive validation
  - Enforces pattern: `^docs/adrs/\d{4}-[a-z0-9-]+\.md$`
  - Blocks edits to existing ADRs unless `allow-adr-edits` label present
  - Provides detailed error messages with examples and guidance
  - Added ADR governance section to CONTRIBUTING.md with complete procedures
  - Updated PR template with ADR immutability checklist
- [x] Create reproducible toolchain pins for Node (`.nvmrc` or Volta config) and confirm
      `.rust-toolchain` matches the intended compiler release. - Created `.nvmrc` with Node 20
      (LTS) - Confirmed `.rust-toolchain` specifies Rust 1.91.1 - Updated all CI workflows to use
      Rust 1.91.1 (matching `.rust-toolchain`) - Documented version pinning in `CONTRIBUTING.md`
- [x] Introduce automation scripts under `scripts/` (e.g., dataset fixture sync, release helpers)
      and register them as Nx tasks if applicable.
  - Created `scripts/project.json` with 10 Nx targets for Python and Node scripts
  - Created `scripts/requirements.txt` with stdlib-only dependencies
  - Created `scripts/README.md` documenting all available tasks
  - Fixed `inspect_db.py` bug (now handles schema variations gracefully)
  - Updated `CONTRIBUTING.md` and `docs/USAGE.md` with script task documentation

## Library (`evefrontier-lib`)

- [x] Implement `ensure_e6c3_dataset` to download the latest dataset release from GitHub, cache it
      in the OS cache directory under `evefrontier_datasets/`, perform atomic writes, and optionally
      extract `.zip` archives ([ADR 0003](adrs/0003-downloader-caching.md)).
- [x] Support injecting a pre-existing dataset path (for tests) and allow callers to override the
      cache location.
- [x] Implement `load_starmap` with runtime schema detection
      ([ADR 0004](adrs/0004-schema-detection.md)) for both `SolarSystems` / `Jumps` and legacy
      `mapSolarSystems` datasets.
- [x] Define the `Starmap` data model, including system metadata, adjacency lists, and a `HashMap`
      for name-to-ID lookups.
- [x] Build graph construction helpers (`graph.rs`) that transform the `Starmap` into search graphs
      for gate, spatial, or hybrid routing modes.
- [x] Implement pathfinding algorithms in `path.rs`: breadth-first search for unweighted graphs,
      Dijkstra for weighted routes, and A\* for heuristic-guided searches. Support filters such as
      maximum jump distance, gate-only routes, spatial routes, avoided systems, and temperature
      constraints.
- [x] Provide serialization helpers for CLI/Lambda outputs (plain text, rich text, JSON, in-game
      note format) with appropriate structs and enums.
- [x] Add robust error handling via a shared `Error` type (using `thiserror`) and bubble errors to
      callers with actionable messages. - Comprehensive `Error` enum in `error.rs` with thiserror
      derive - All error variants have descriptive messages and context (paths, tags, system
      names) - Transparent wrapping of underlying errors (SQLite, IO, HTTP, ZIP) - Helper functions
      for formatted error suggestions
- [x] Write unit tests covering schema detection, dataset loading, graph construction, and routing
      behavior using fixtures in `docs/fixtures/`. - 8 test files with 43+ integration tests - Tests
      cover: dataset_download, dataset_fixture_guard, dataset_normalize, fuzzy_matching, graph,
      load_starmap, output, routing - All tests use minimal_static_data.db fixture from
      docs/fixtures/ - Comprehensive edge case and error path coverage
- [x] Document the public API in `docs/USAGE.md` and Rustdoc comments. - Enhanced lib.rs with
      comprehensive module documentation and usage examples - Added "Library API" section to
      docs/USAGE.md with code examples - Covers common patterns: dataset loading, routing, error
      handling, output formatting - Includes examples for all three routing algorithms and
      constraint usage
- [x] Implement KD-tree spatial index module (per ADR 0009): build, serialize (e.g., postcard +
      zstd), load, query nearest systems. - `spatial.rs` module with kiddo v4.2, postcard+zstd
      serialization, SHA-256 checksum - Index format: EFSI magic, version 1, feature flags,
      compressed tree data - `build_spatial_index()`, `SpatialIndex::save()`, `load_spatial_index()`
      functions
- [x] Integrate KD-tree spatial index into spatial/hybrid routing path selection logic. -
      `build_spatial_graph_indexed()` and `build_hybrid_graph_indexed()` in graph.rs - Auto-build
      fallback with `tracing::warn!` when index not provided - `select_graph()` in routing.rs uses
      indexed builders for Dijkstra/A\*
- [x] Provide tests/benchmarks for KD-tree build and query performance. - 8 integration tests in
      `tests/spatial_index.rs` - Covers: build, serialize/deserialize, checksum validation, queries,
      temperature filtering
- [x] Make KD-tree temperature-aware for neighbor queries - `IndexNode` stores `min_external_temp`
      per system - `NeighbourQuery` accepts `max_temperature: Option<f64>` filter -
      `nearest_filtered()` and `within_radius_filtered()` apply predicates - (v2 future work:
      subtree temperature aggregates for branch pruning)

## Ship Data & Fuel Calculations (ADR 0015)

- [ ] Create `ship.rs` module with `ShipAttributes`, `ShipLoadout`, and `ShipCatalog` structs
- [ ] Implement CSV parsing for `ship_data.csv` with validation (base_mass_kg, fuel_capacity,
      cargo_capacity, specific_heat)
- [ ] Add `calculate_jump_fuel_cost()` function implementing the fuel formula:
      `(total_mass_kg / 10^5) × (fuel_quality / 100) × distance_ly`
- [ ] Add `calculate_route_fuel()` for full route projection with static/dynamic mass modes
- [ ] Implement `ShipLoadout::total_mass_kg()` combining hull + fuel + cargo mass
- [ ] Extend GitHub downloader (`github.rs`) to fetch `ship_data.csv` from release assets
- [ ] Add ship data caching alongside the database in `evefrontier_datasets/` cache directory
- [ ] Extend `RouteStep` with optional `FuelProjection` struct (hop_cost, cumulative, remaining)
- [ ] Extend `RouteSummary` with total_fuel, ship_name, and fuel_warning fields
- [ ] Update output formatters to include fuel information in status line when ship specified
- [ ] Add `--ship`, `--fuel-quality`, `--cargo-mass`, `--fuel-load` CLI options to `route`
      subcommand
- [ ] Add `--dynamic-mass` flag to enable per-hop mass recalculation
- [ ] Add `--list-ships` convenience option to display available ships
- [ ] Extend Lambda request/response schemas with ship, loadout, and fuel parameters
- [ ] Bundle `ship_data.csv` with Lambda deployment artifacts
- [ ] Add ship data fixture for testing (subset of ships)
- [ ] Write unit tests for fuel calculation formula with known test cases (static and dynamic modes)
- [ ] Write integration tests for CLI fuel projection output
- [ ] Update `USAGE.md` with fuel projection examples including cargo and dynamic mass
- [ ] Future: Research and implement heat impact calculations (requires separate ADR)

## CLI (`evefrontier-cli`)

- [x] Implement the CLI skeleton with Clap, including global `--data-dir`, `--format`, `--no-logo`,
      and other shared options. Respect the data path resolution order defined in
      `docs/INITIAL_SETUP.md`.
- [x] Implement the `download` subcommand that wraps `ensure_e6c3_dataset`, reports the resolved
      path, and exits with appropriate codes.
- [x] Implement unified `route` subcommand (replacing earlier `search` / `path`) exposing all
      routing functionality via flags: `--algorithm`, `--format`, `--max-jump`, `--avoid`,
      `--avoid-gates`, `--max-temp`.
- [x] Provide friendly error messages for unknown systems and route failures.
- [x] Add integration tests for CLI behavior (using `assert_cmd` or similar) with the fixture
      dataset.
- [x] Update `README.md` and `docs/USAGE.md` with CLI examples that match the implemented behavior.
- [x] Add `index-build` (or `build-index`) subcommand to precompute KD-tree spatial index
      artifact. - `evefrontier-cli index-build` command implemented - Saves to
      `{database_path}.spatial.bin` - Supports `--force` flag to overwrite existing index
- [x] Surface friendly errors when spatial index missing but requested. - Auto-build fallback with
      warning instead of hard error - Uses `tracing::warn!` to inform user before building on-demand

## AWS Lambda crates

- [x] Scaffold Lambda crates (e.g., `evefrontier-lambda-route`, `evefrontier-lambda-scout-gates`,
      `evefrontier-lambda-scout-range`) that depend on the library crate.
  - Created `evefrontier-lambda-shared` crate with common infrastructure
  - Created three Lambda crates: `route`, `scout-gates`, `scout-range`
  - All Lambda crates use `include_bytes!` for bundled dataset
- [x] Implement shared bootstrap logic to download or locate the dataset at cold start and share it
      across invocations (per `.github/copilot-instructions.md`).
  - `LambdaRuntime` in `evefrontier-lambda-shared` provides singleton initialization
  - Uses rusqlite `deserialize_bytes()` for zero-copy database loading
  - `SpatialIndex::load_from_bytes()` added for bundled spatial index
  - Cold-start timing logged via tracing (`db_load_ms`, `index_load_ms`, `total_init_ms`)
- [x] Define request/response models using `serde` and ensure JSON serialization matches API
      contracts.
  - `RouteRequest`, `ScoutGatesRequest`, `ScoutRangeRequest` with `Validate` trait
  - `LambdaResponse<T>` wrapper with `content_type: "application/json"`
  - RFC 9457 `ProblemDetails` for structured error responses
- [x] Wire Lambda handlers (using `lambda_runtime` or `aws_lambda_events`) to call library APIs and
      return results with structured errors.
  - All three handlers wired with `lambda_runtime::run(service_fn(handler))`
  - Handlers return `Result<Response, Error>` with `Response` enum for success/error
  - Route handler: parses request, validates, calls `plan_route()`, returns route with system names
  - Scout-gates handler: returns gate-connected neighbors from adjacency list
  - Scout-range handler: uses `SpatialIndex::nearest_filtered()` with radius/temperature filters
- [x] Provide infrastructure notes or SAM/CDK templates (if required) for deployment under `docs/`.
  - Created Terraform module at `terraform/modules/evefrontier-lambda/` (per ADR 0013)
  - Module includes: Lambda functions, HTTP API Gateway v2, IAM roles, CloudWatch Logs
  - Example configuration at `terraform/examples/complete/`
  - Comprehensive deployment guide in `docs/DEPLOYMENT.md`
- [x] Add Lambda-focused tests (unit tests and, if possible, integration tests using
      `lambda_runtime::run` mocks).
  - Added 44 unit tests across three Lambda crates
  - Route Lambda: 15 tests covering request parsing, validation, route planning, response
    serialization
  - Scout-Gates Lambda: 13 tests covering request parsing, validation, gate lookup, response
    serialization
  - Scout-Range Lambda: 16 tests covering request parsing, validation, spatial queries, response
    serialization
  - Added test utilities module in `evefrontier-lambda-shared` for shared fixtures
  - All tests use `docs/fixtures/minimal_static_data.db` fixture
- [x] Integrate KD-tree spatial index loading at cold start if artifact bundled.
  - `SpatialIndex::load_from_bytes()` and `load_from_reader()` added to library
  - Lambda runtime loads spatial index from bundled bytes at cold start

## Testing & Quality

- [x] Ensure `cargo fmt`, `cargo clippy --all-targets --all-features`, and `cargo test --workspace`
      run cleanly in CI ([ADR 0007](adrs/0007-devsecops-practices.md)). Pre-commit hooks configured
      with rusty-hook to run all CI checks locally.
- [x] Hook Rust build, test, lint, and clippy tasks into Nx orchestration per
      [ADR 0006](adrs/0006-software-components.md) and [ADR 0007](adrs/0007-devsecops-practices.md).
  - Added `parallel: false` to all Rust task targetDefaults in nx.json to allow Cargo to manage its
    own parallelism
  - Added `dependsOn: ["build"]` to all test targets across 6 crates to ensure builds complete
    before tests
  - Added `dependsOn: ["build"]` to clippy target in nx.json's targetDefaults (not in individual
    project.json files) to ensure compilation before linting
  - Configured outputs for build target to cache `target/debug` and `target/release` directories
  - Documented Nx task orchestration in CONTRIBUTING.md with usage examples and troubleshooting
  - Verified task execution with caching and dependency resolution working correctly
- [x] Add dataset fixture management helpers to keep fixtures synchronized and documented in
      `docs/fixtures/README.md`.
- [x] Integrate `cargo audit` and Node SCA checks into CI and document remediation workflows. -
      cargo-audit integrated into CI (`security-audit` job in `.github/workflows/ci.yml`) -
      Pre-commit hook updated to run cargo audit (step 5 in `.rusty-hook.toml`) - `make audit`
      target added to Makefile - Comprehensive remediation guide created in `docs/SECURITY_AUDIT.md`
- [x] Add CI guard requiring `CHANGELOG.md` modification for non-doc code changes
      ([ADR 0010](adrs/0010-maintain-changelog.md)). - Created `.github/scripts/check-changelog.sh`
      with file pattern detection and exemption rules - Added `changelog-guard` job to
      `.github/workflows/ci.yml` with proper triggering - Updated `CONTRIBUTING.md` with
      "Maintaining CHANGELOG.md" section - Implemented emergency override via `skip-changelog-check`
      label - Clear error messages with exemption rules and CONTRIBUTING.md link - Exempts
      `.github/scripts/**`, `specs/**`, and explicit root `.md` files - **Future v2 enhancement**:
      Bot PR exemption (Dependabot, Renovate) documented in spec.md edge cases
- [x] Schedule nightly dependency outdated report (Rust & Node) and publish artifact. - Created
      `.github/workflows/dependency-check.yml` with nightly schedule (2 AM UTC) - Separate jobs for
      Rust (`cargo outdated`) and Node (`pnpm outdated`) - Artifacts published with 30-day
      retention: `rust-outdated-report`, `node-outdated-report` - Manual trigger via
      `workflow_dispatch` also supported - Documented in `CONTRIBUTING.md` under "Dependency
      Management" section
  - [x] Establish benchmarking or profiling harnesses for pathfinding performance (optional but
        recommended).

## Documentation & Communication

- [x] Create `CHANGELOG.md` with an `Unreleased` section and update it for each change (per
      `CONTRIBUTING.md`). - CHANGELOG.md exists at repository root - Unreleased section actively
      maintained - Follows format: date - author - [category] - description - Integrated into
      pre-commit workflow via CONTRIBUTING.md guidance
- [x] Align `README.md` with the new workspace layout, CLI commands, Lambda crates, and development
      workflow.
  - Documented all 6 crates with clear descriptions (lib, cli, 4 Lambda functions)
  - Added comprehensive workspace structure section with categories
  - Updated all CLI examples to use real system names (Nod, Brana, H:2L2S)
  - Documented Nx task orchestration with practical examples
  - Added spatial index usage section
  - Included Lambda deployment overview
  - Added library API example code
- [x] Expand `docs/USAGE.md` with Lambda invocation examples and dataset caching behavior.
  - Added comprehensive "AWS Lambda Functions" section with three endpoints documented
  - Request/response schemas with real examples for all Lambda functions
  - AWS SDK examples in Python and JavaScript for each endpoint
  - curl invocation examples with API Gateway integration
  - Cold-start behavior and initialization sequence documentation
  - Performance metrics and memory usage guidelines
  - Configuration, secrets, and IAM permissions documentation
  - Deployment considerations with build commands and Lambda settings
- [x] Document release and signing procedures in `docs/RELEASE.md`, including cosign/GPG commands
      and attestation steps ([ADR 0007](adrs/0007-devsecops-practices.md)).
  - Created comprehensive 870-line release procedures document
  - Covers GPG-signed tags, cosign binary signatures, SBOM generation (CycloneDX)
  - Documents cross-compilation (x86_64/aarch64), spatial index inclusion
  - Includes CI integration patterns for GitHub Actions
  - Contains rollback/revocation procedures and troubleshooting guide
- [x] Implement CI release job with artifact signing (cosign/GPG) and attestation generation per
      [ADR 0007](adrs/0007-devsecops-practices.md).
  - Created `.github/workflows/release.yml` triggered on `v*` tags
  - Multi-architecture builds: x86_64 and aarch64 Linux targets
  - SHA256 checksums generated for all tarballs
  - Keyless cosign signing using GitHub Actions OIDC identity
  - SBOM generation in CycloneDX format via cargo-sbom
  - Automated GitHub Release creation with all artifacts
  - All actions pinned to full commit SHAs for supply chain security
- [x] Add architecture diagrams or sequence diagrams illustrating data flow between downloader,
      loader, graph, CLI, and Lambda components.
  - Created comprehensive `docs/ARCHITECTURE.md` with 7 Mermaid diagrams
  - Component Overview, Module Dependencies, Dataset Download Flow
  - Starmap Load Flow, Route Planning Flow, CLI Sequence, Lambda Cold-Start Sequence
  - Cross-referenced from README.md and docs/README.md
- [x] Provide onboarding steps in `docs/INITIAL_SETUP.md` once the workspace scaffolding stabilizes
      (update as tasks complete).
- [x] Extend `docs/USAGE.md` with KD-tree index usage and build instructions. - Added `index-build`
      subcommand documentation with examples - Documented when to rebuild and temperature-aware
      filtering
- [ ] Add ADR for any deviations from original KD-tree design if implementation adjustments occur.
- [x] Add `docs/RELEASE.md` section describing inclusion of spatial index artifact.
  - Spatial Index Generation subsection added with `index-build` command usage
  - Package Assembly subsection includes spatial index in tarball creation
- [x] Add Rust-specific badges to README (crates.io, docs.rs, maintenance status, license, build
      status).
- [ ] Publish `evefrontier-lib` to crates.io (prerequisite for badges).
- [ ] Publish `evefrontier-cli` to crates.io (prerequisite for badges).
- [ ] Create GitHub Release v0.1.0 with signed artifacts (prerequisite for release badge).

## Security & Operations

- [x] Define secrets management and configuration guidance for Lambda deployments (environment
      variables, AWS Secrets Manager, etc.).
- [x] Implement logging and observability hooks (structured logs, optional tracing) across CLI and
      Lambdas.
- [ ] Add metrics or usage telemetry (if desired) with opt-in controls and documentation.
- [ ] Plan for dataset update automation (scheduled job or manual release) and document operational
      runbooks.
- [x] Add CI to verify spatial index artifact freshness against dataset version.
  - Created `index-verify` CLI command with JSON and human-readable output
  - Added `spatial-index-freshness` CI job to `.github/workflows/ci.yml`
  - Implemented v2 spatial index format with embedded source metadata (checksum, release tag,
    timestamp)
  - Full backward compatibility with v1 format loading
- [x] Document operational procedure for regenerating spatial index after dataset updates.
  - Added "Regenerating Spatial Index" section to `docs/USAGE.md`
  - Added "Troubleshooting CI Failures" section with common causes and debugging steps
  - Added "Spatial Index Format v2" section with format specification
  - Added "Lambda Freshness Behavior" section explaining build-time verification
- [ ] Build Terraform deployment solution to deploy the Lambda functions and document per
      [ADR 0007](adrs/0007-devsecops-practices.md) secrets management requirements.
- [ ] Bake AWS deployment into GitHub CI runner using GitHub Secrets as a secrets source.
- [x] Add Rust based cyclomatic complexity scanning to ensure that code complexity remains within
      acceptable bounds (i.e. < 15)
  - Created `clippy.toml` with complexity thresholds: cognitive_complexity=15, too_many_lines=100,
    excessive_nesting=8, too_many_arguments=8
  - Added `complexity-check` CI job to `.github/workflows/ci.yml` with clippy lints enabled
  - Added `complexity` Nx target to all 6 Rust crates in `project.json` files
  - Updated `nx.json` targetDefaults and cacheableOperations to include complexity checks
  - Documented complexity rules in `docs/CODING_GUIDELINES.md` with thresholds and remediation tips
  - Added complexity check command examples to `CONTRIBUTING.md`

## Docker Microservices & Kubernetes Deployment

- [x] Create ADR documenting containerization and Kubernetes deployment strategy, covering security
      practices, image signing, and operational requirements.
  - Created `docs/adrs/0014-containerization-strategy.md`
- [x] Create Dockerfiles for each microservice (route, scout-gates, scout-range) using multi-stage
      builds with Distroless base images for minimal runtime containers.
  - Created multi-stage Dockerfiles for all three microservices
  - Base image: `gcr.io/distroless/cc-debian12:nonroot` (~20MB)
  - Uses cargo-zigbuild for musl static linking
- [x] Ensure microservice scope aligns with Lambda function boundaries: each container should
      implement a single, well-defined API endpoint matching its Lambda equivalent.
  - `evefrontier-service-route`, `evefrontier-service-scout-gates`,
    `evefrontier-service-scout-range`
  - All use `evefrontier-service-shared` for common HTTP infrastructure (axum-based)
- [x] Configure Traefik as the API Gateway for the microservices stack:
  - [x] Define Traefik routing rules and middleware (rate limiting, authentication, etc.)
    - Traefik IngressRoute in `charts/evefrontier/templates/ingress.yaml`
    - Rate limiting and CORS middleware in `charts/evefrontier/templates/middleware.yaml`
  - [x] Set up service discovery and load balancing configuration
    - Kubernetes Services with ClusterIP, path-based routing via IngressRoute
  - [x] Document Traefik ingress configuration and TLS/certificate management
    - Documented in `charts/evefrontier/README.md`
- [x] Create Helm chart for Kubernetes deployment:
  - [x] Chart structure with values.yaml for configuration (replicas, resource limits, etc.)
    - `charts/evefrontier/Chart.yaml` and `charts/evefrontier/values.yaml`
  - [x] Kubernetes manifests for deployments, services, and config maps
    - `deployment-*.yaml`, `service-*.yaml`, `configmap.yaml`, `pvc.yaml`
  - [x] Traefik ingress resources and routing configuration
    - `ingress.yaml` with both IngressRoute (Traefik) and standard Ingress support
  - [x] Health check and readiness probe definitions
    - Liveness (`/health/live`) and readiness (`/health/ready`) probes configured
  - [x] Documentation for chart installation and configuration options
    - Comprehensive `charts/evefrontier/README.md` with examples
- [x] Add CI/CD pipeline for building and publishing Docker images per
      [ADR 0007](adrs/0007-devsecops-practices.md):
  - [x] Multi-architecture builds (amd64, arm64) for container images
    - cargo-zigbuild for cross-compilation in `.github/workflows/docker-release.yml`
  - [x] Image scanning for vulnerabilities (e.g., Trivy, Grype) integrated into CI workflow
    - Trivy scan with CRITICAL,HIGH severity blocking
  - [x] Image signing with cosign for supply chain security
    - Keyless cosign signing using GitHub OIDC identity
  - [x] Push to container registry with semantic versioning tags
    - ghcr.io with v0.1.0, 0.1, 0, latest tags
  - [x] Generate and attach SBOM (Software Bill of Materials) to images
    - syft generates SPDX and CycloneDX SBOMs
- [x] Document deployment procedures in `docs/DEPLOYMENT.md`:
  - [x] Local development with Docker Compose
    - `docker-compose.yml` with Traefik reverse proxy
  - [x] Kubernetes deployment using Helm
    - Helm installation instructions and examples
  - [x] Configuration and secrets management for containerized environments
    - values.yaml configuration, ConfigMap, ServiceAccount
  - [ ] Observability setup (metrics, logs, traces) for microservices
    - Tracing infrastructure exists but metrics/observability documentation pending
    - Prefer OpenTelemetry integration for future work, refactor existing tracing hooks to use OTEL
      when feasible.
- [ ] Generate quadlet files for systemd based container management with Podman
  - [ ] Create quadlet files for each microservice with appropriate resource limits and restart
        policies
  - [ ] Document quadlet installation and management procedures in `docs/DEPLOYMENT.md`

## Architecture Documentation (Recommended ADRs)

The following ADR topics are recommended to formalize currently implicit architectural decisions:

- [ ] **ADR 0016: Web-Based Starmap Explorer Architecture** (Currently deferred feature)
  - [ ] Document frontend framework choice (React, Svelte, Vue, etc.)
  - [ ] Define API contract between CLI server and web UI
  - [ ] Specify deployment strategy (bundled with CLI vs. separate service)
  - [ ] Define interactive route planning UX and algorithm exposure
  - Currently blocked: Prioritize other work; design document deferred
  - Related task: Implement web explorer under "Web-based Starmap Explorer" section below

- [x] **ADR 0017: NX Repository Orchestration Strategy** (Proposed; awaiting review)
  - [x] Document rationale for Nx selection (build caching, task orchestration, developer
        experience)
  - [x] Specify target configuration patterns (input/output hashing, parallel execution)
  - [x] Document CI integration (task execution, artifact caching)
  - [x] Define custom task patterns for Rust crates and scripts
  - [x] Created ADR in docs/adrs/0017-nx-orchestration-strategy.md with full decision, consequences,
        and alternatives
  - [x] Updated CONTRIBUTING.md with reference to ADR 0017 for task configuration guidance
  - [x] Updated AGENTS.md with note about ADR 0017 for task configuration patterns

- [ ] **ADR 0018: Heat Mechanics Research Summary** (Prerequisite for ADR 0015 Phase 2)
  - [ ] Research and validate heat mechanic formulas from EVE Frontier game mechanics
  - [ ] Document findings: thermal stress model, ship-specific heat tolerance, rate of heat
        accumulation
  - [ ] Create validation test cases against observed in-game behavior
  - [ ] Define implementation specification for heat calculations in RouteStep/RouteSummary
  - Currently blocked: Pending community research and game mechanic validation
  - Related task: This ADR must complete before implementing "Ship Data & Fuel Calculations Phase 4"

- [ ] **ADR 0019: Lambda Architecture and Cold-Start Optimization** (Currently implicit in Lambda
      crates)
  - [ ] Document cold-start constraints (binary size limits, initialization timing budgets)
  - [ ] Specify spatial index bundling and lazy-loading strategy
  - [ ] Define state initialization sequence (database loading, index deserialization, metrics
        setup)
  - [ ] Justify architectural choices (e.g., why shared infrastructure in evefrontier-lambda-shared)
  - [ ] Specify performance targets and monitoring strategy
  - Currently working: Implementation is sound; ADR would clarify trade-offs and constraints

## Web-based Starmap Explorer

- [ ] Design and implement a web-based starmap explorer using a modern frontend framework (e.g.,
      React) that can be started with `evefrontier-cli serve` which starts both an API server and
      serves the web app.
- [ ] Display star systems, jump gates, and allow users to interactively plan routes using the same
      algorithms as the CLI and Lambdas all based upon local data.

## Known Issues / Tweaks

- [x] The GOAL item in `enhanced` mode doesn't include a status line (min temp, planets, moons).
      (Fixed in output.rs; also added "Black Hole" indicator for systems 30000001-30000003)
- [ ] `enhanced` should be the default not basic as it provides the clearest representation.
- [ ] Add LibrePay integration with a link in the README for donations/support, plus a once every 7
      days reminder in the CLI footer.
- [x] Add support for generating fmap URLs based upon
      [ROUTE_FEATURE.md](https://github.com/frontier-reapers/starmap/blob/main/docs/ROUTE_FEATURE.md)
      specification.
  - [x] Write a comprehensive ADR describing both the encoder and decoder algorithms for fmap URLs.
      (Implemented in `crates/evefrontier-lib/src/fmap.rs` with 3-bit waypoint type encoding;
      CLI commands `fmap-encode` and `fmap-decode` added; route output includes fmap URL)
- [ ] Add support for avoiding systems by solarsystem ID, name or radius from solar system.
- [ ] Add support for specifying some parameters (i.e. avoidance systems, algorithim, etc.) either
      in `~/.config` or ENV_VARs

## Packaging

- [ ] Add `nixpkgs` packaging for easy installation on NixOS and other Nix-based systems.
- [ ] Add Homebrew formula for macOS users to easily install via `brew install evefrontier-cli`.
- [ ] Add Debian/Ubuntu packaging for installation via `apt-get install evefrontier-cli`.
- [ ] Add Fedora/CentOS packaging for installation via `dnf install evefrontier-cli`.
- [ ] Add Arch Linux packaging for installation via `pacman -S evefrontier-cli`.
- [ ] Add Windows installer (MSI) for easy installation on Windows systems.
- [ ] Add Snap package for easy installation on Linux systems via `snap install evefrontier-cli`.
- [ ] Add Flatpak package for easy installation on Linux systems via
      `flatpak install evefrontier-cli`.
- [ ] Add Winget package for easy installation on Windows systems via
      `winget install evefrontier-cli`.
- [ ] Add Chocolatey package for easy installation on Windows systems via
      `choco install evefrontier-cli`.
- [ ] Add Docker image for easy deployment in containerized environments via
      `docker pull evefrontier-cli`.

## MCP Server Integration

- [ ] **CLI Subcommand & Stdio Transport**: Implement the `mcp` command and set up JSON-RPC 2.0
      communication over `stdin`/`stdout`.
- [ ] **EVE Frontier Tool Mapping**: Expose core library functionality (world state queries, account
      balance, blockchain transactions) as MCP tools.
- [ ] **Resource & Schema Definitions**: Define AI-readable resources for game data models and smart
      assembly configurations.
- [ ] **Docker & Security Hardening**: Ensure the server runs with dropped capabilities
      (`CAP_DROP=all`), non-root users, and static `musl` builds.
- [ ] **IO & Logging Isolation**: Configure the logging framework to redirect all system logs to
      `stderr` to prevent protocol corruption on `stdout`.
- [ ] **Integration & Distribution**: Provide configuration templates for MCP clients (Claude
      Desktop, Cursor) and automated Docker health checks.

---

This checklist should be updated as work progresses. When tasks are completed, mark the relevant
checkboxes and add links to PRs or documentation updates.
