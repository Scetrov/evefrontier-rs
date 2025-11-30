# Project TODOs

This TODO list captures the remaining work required to implement the EveFrontier CLI, library, AWS
Lambda functions, and supporting infrastructure described throughout the documentation and ADRs.
Tasks are grouped by domain; checkboxes track completion status.

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
- [ ] Add CI workflow enforcing ADR filename pattern (`^\\d{4}-.+\\.md$`) and immutability (reject
      edits to historical ADRs except via explicit override label) per [ADR 0001](adrs/0001-use-nygard-adr.md).
- [x] Create reproducible toolchain pins for Node (`.nvmrc` or Volta config) and confirm
      `.rust-toolchain` matches the intended compiler release. - Created `.nvmrc` with Node 20
      (LTS) - Confirmed `.rust-toolchain` specifies Rust 1.91.1 - Updated all CI workflows to use
      Rust 1.91.1 (matching `.rust-toolchain`) - Documented version pinning in `CONTRIBUTING.md`
- [ ] Introduce automation scripts under `scripts/` (e.g., dataset fixture sync, release helpers)
      and register them as Nx tasks if applicable.

## Library (`evefrontier-lib`)

- [x] Implement `ensure_c3e6_dataset` to download the latest dataset release from GitHub, cache it
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

## CLI (`evefrontier-cli`)

- [x] Implement the CLI skeleton with Clap, including global `--data-dir`, `--format`, `--no-logo`,
      and other shared options. Respect the data path resolution order defined in
      `docs/INITIAL_SETUP.md`.
- [x] Implement the `download` subcommand that wraps `ensure_c3e6_dataset`, reports the resolved
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
- [ ] Provide infrastructure notes or SAM/CDK templates (if required) for deployment under `docs/`.
- [ ] Add Lambda-focused tests (unit tests and, if possible, integration tests using
      `lambda_runtime::run` mocks).
- [x] Integrate KD-tree spatial index loading at cold start if artifact bundled.
  - `SpatialIndex::load_from_bytes()` and `load_from_reader()` added to library
  - Lambda runtime loads spatial index from bundled bytes at cold start

## Testing & Quality

- [x] Ensure `cargo fmt`, `cargo clippy --all-targets --all-features`, and `cargo test --workspace`
      run cleanly in CI ([ADR 0007](adrs/0007-devsecops-practices.md)). Pre-commit hooks configured
      with rusty-hook to run all CI checks locally.
- [x] Hook Rust build, test, lint, and clippy tasks into Nx orchestration per [ADR 0006](adrs/0006-software-components.md)
      and [ADR 0007](adrs/0007-devsecops-practices.md).
  - Added `parallel: false` to all Rust task targetDefaults in nx.json to allow Cargo to manage its own parallelism
  - Added `dependsOn: ["build"]` to all test targets across 6 crates to ensure builds complete before tests
  - Added `dependsOn: ["build"]` to clippy target in nx.json's targetDefaults (not in individual project.json files) to ensure compilation before linting
  - Configured outputs for build target to cache `target/debug` and `target/release` directories
  - Documented Nx task orchestration in CONTRIBUTING.md with usage examples and troubleshooting
  - Verified task execution with caching and dependency resolution working correctly
- [x] Add dataset fixture management helpers to keep fixtures synchronized and documented in
      `docs/fixtures/README.md`.
- [x] Integrate `cargo audit` and Node SCA checks into CI and document remediation workflows. -
      cargo-audit integrated into CI (`security-audit` job in `.github/workflows/ci.yml`) -
      Pre-commit hook updated to run cargo audit (step 5 in `.rusty-hook.toml`) - `make audit`
      target added to Makefile - Comprehensive remediation guide created in `docs/SECURITY_AUDIT.md`
- [ ] Add CI guard requiring `CHANGELOG.md` modification for non-doc code changes ([ADR 0010](adrs/0010-maintain-changelog.md)).
      Currently documented in CONTRIBUTING.md but not enforced by CI workflow.
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
- [ ] Expand `docs/USAGE.md` with Lambda invocation examples and dataset caching behavior.
- [ ] Document release and signing procedures in `docs/RELEASE.md`, including cosign/GPG commands
      and attestation steps ([ADR 0007](adrs/0007-devsecops-practices.md)).
- [ ] Implement CI release job with artifact signing (cosign/GPG) and attestation generation per
      [ADR 0007](adrs/0007-devsecops-practices.md).
- [ ] Add architecture diagrams or sequence diagrams illustrating data flow between downloader,
      loader, graph, CLI, and Lambda components.
- [ ] Provide onboarding steps in `docs/INITIAL_SETUP.md` once the workspace scaffolding stabilizes
      (update as tasks complete).
- [x] Extend `docs/USAGE.md` with KD-tree index usage and build instructions. - Added `index-build`
      subcommand documentation with examples - Documented when to rebuild and temperature-aware
      filtering
- [ ] Add ADR for any deviations from original KD-tree design if implementation adjustments occur.
- [ ] Add `docs/RELEASE.md` section describing inclusion of spatial index artifact.
- [ ] Add Rust-specific badges to README (crates.io, docs.rs, maintenance status, license, build status).

## Security & Operations

- [ ] Define secrets management and configuration guidance for Lambda deployments (environment
      variables, AWS Secrets Manager, etc.).
- [ ] Implement logging and observability hooks (structured logs, optional tracing) across CLI and
      Lambdas.
- [ ] Add metrics or usage telemetry (if desired) with opt-in controls and documentation.
- [ ] Plan for dataset update automation (scheduled job or manual release) and document operational
      runbooks.
- [ ] Add CI to verify spatial index artifact freshness against dataset version.
- [ ] Document operational procedure for regenerating spatial index after dataset updates.
- [ ] Build Terraform deployment solution to deploy the Lambda functions and document per
      [ADR 0007](adrs/0007-devsecops-practices.md) secrets management requirements.
- [ ] Bake AWS deployment into GitHub CI runner using GitHub Secrets as a secrets source.
- [ ] Add Rust based cyclomatic compexity scanning to ensure that code complexity remains within
      acceptable bounds (i.e. < 15)

## Docker Microservices & Kubernetes Deployment

- [ ] Create ADR documenting containerization and Kubernetes deployment strategy, covering security
      practices, image signing, and operational requirements.
- [ ] Create Dockerfiles for each microservice (route, scout-gates, scout-range) using multi-stage
      builds with Distroless base images for minimal runtime containers.
- [ ] Ensure microservice scope aligns with Lambda function boundaries: each container should
      implement a single, well-defined API endpoint matching its Lambda equivalent.
- [ ] Configure Traefik as the API Gateway for the microservices stack:
  - [ ] Define Traefik routing rules and middleware (rate limiting, authentication, etc.)
  - [ ] Set up service discovery and load balancing configuration
  - [ ] Document Traefik ingress configuration and TLS/certificate management
- [ ] Create Helm chart for Kubernetes deployment:
  - [ ] Chart structure with values.yaml for configuration (replicas, resource limits, etc.)
  - [ ] Kubernetes manifests for deployments, services, and config maps
  - [ ] Traefik ingress resources and routing configuration
  - [ ] Health check and readiness probe definitions
  - [ ] Documentation for chart installation and configuration options
- [ ] Add CI/CD pipeline for building and publishing Docker images per [ADR 0007](adrs/0007-devsecops-practices.md):
  - [ ] Multi-architecture builds (amd64, arm64) for container images
  - [ ] Image scanning for vulnerabilities (e.g., Trivy, Grype) integrated into CI workflow
  - [ ] Image signing with cosign for supply chain security
  - [ ] Push to container registry with semantic versioning tags
  - [ ] Generate and attach SBOM (Software Bill of Materials) to images
- [ ] Document deployment procedures in `docs/DEPLOYMENT.md`:
  - [ ] Local development with Docker Compose
  - [ ] Kubernetes deployment using Helm
  - [ ] Configuration and secrets management for containerized environments
  - [ ] Observability setup (metrics, logs, traces) for microservices

---

This checklist should be updated as work progresses. When tasks are completed, mark the relevant
checkboxes and add links to PRs or documentation updates.
