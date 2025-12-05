# Changelog

All notable changes to this project will be documented in this file.

## Unreleased
- 2025-12-05 - auto-llm:copilot - [ci] - Added ADR governance workflow enforcing filename pattern and immutability per ADR 0001 and Constitution Principle III
- 2025-12-05 - auto-llm:copilot - [docs] - Added comprehensive Lambda invocation documentation to USAGE.md with request/response schemas, SDK examples, and cold-start behavior
- 2025-12-05 - auto-llm:copilot - [docs] - Added ADR governance section to CONTRIBUTING.md documenting immutability policy, naming conventions, and override procedures
- 2025-12-05 - auto-llm:copilot - [github] - Created PR template with ADR checklist and security/testing requirements

- 2025-11-30 - GitHub Copilot - [docs] - Aligned README.md with current workspace layout: documented 6 Lambda crates, Nx orchestration, spatial indexing, and updated all examples to use real system names

- 2025-11-30 - GitHub Copilot - [build] - Hooked Rust build/test/lint/clippy tasks into Nx orchestration with proper dependency chains and caching
- 2025-11-30 - GitHub Copilot - [build] - Added `dependsOn: ["build"]` to all test targets across 6 crates to ensure builds complete before tests
- 2025-11-30 - GitHub Copilot - [build] - Configured `parallel: false` for Rust tasks in nx.json to allow Cargo to manage its own compilation parallelism
- 2025-11-30 - GitHub Copilot - [docs] - Added comprehensive Nx task orchestration section to CONTRIBUTING.md documenting Nx task dependencies, caching behavior, usage examples, and troubleshooting
- 2025-11-30 - GitHub Copilot - [deps] - Bumped `kiddo` from 4.2.1 to 5.2 (KD-tree). Verified spatial index build/load and tests; no API adjustments required.
- 2025-11-30 - GitHub Copilot - [deps] - Bumped `criterion` from 0.5.1 to 0.8.0 (benchmarks). Updated bench to use `std::hint::black_box` removing deprecation warnings.
- 2025-11-30 - GitHub Copilot - [lint] - Fixed clippy doc comment warning in `evefrontier-lambda-scout-gates`.

- 2025-11-16 - GitHub Copilot - [feature] - Implemented KD-tree spatial index module (spatial.rs) per ADR 0009: build/save/load with postcard+zstd, SHA-256 checksum validation
- 2025-11-16 - GitHub Copilot - [feature] - Added temperature-aware nearest-neighbor and radius queries with min_external_temp filtering
- 2025-11-16 - GitHub Copilot - [feature] - Added CLI `index-build` subcommand to precompute spatial index artifacts (`{db}.spatial.bin`)
- 2025-11-16 - GitHub Copilot - [refactor] - Changed ensure_dataset to return DatasetPaths struct (database path + optional spatial index path)
- 2025-11-16 - GitHub Copilot - [refactor] - Added indexed graph builders (build_spatial_graph_indexed, build_hybrid_graph_indexed) that use spatial index for efficient queries
- 2025-11-16 - GitHub Copilot - [feature] - Auto-build spatial index with warning when index not found but spatial/hybrid routing requested
- 2025-11-16 - GitHub Copilot - [tests] - Added 8 spatial index tests covering build, serialization, checksum validation, queries, and temperature filtering
- 2025-11-16 - GitHub Copilot - [deps] - Added kiddo v4.2 (KD-tree), postcard v1.0 (serialization), zstd v0.13 (compression) dependencies
- 2025-11-16 - GitHub Copilot - [refactor] - Added conservative HTTP retries with exponential backoff in downloader (`github.rs`) for release metadata and asset downloads; improves robustness without API changes. Aligned with OWASP guidance (timeouts, transient failure handling).
- 2025-11-15 - GitHub Copilot - [fix] - Corrected b parameter from 1.125 to 1.25 (transcription error) - now matches expected values exactly
- 2025-11-15 - GitHub Copilot - [docs] - Documented exact EVE Frontier temperature formula with test cases in ADR 0012 and temperature module
- 2025-11-15 - GitHub Copilot - [docs] - Added comprehensive formula documentation: T(d) = 0.1 + 99.8/(1+(d/(3.215e-11*√L))^1.25)
- 2025-11-15 - GitHub Copilot - [docs] - Documented validated test cases: Nod (15.74K at 541.4 ls), Brana (0.32K at 9255.2 ls)
- 2025-11-15 - GitHub Copilot - [fix] - **BREAKING**: Removed nonsensical `--min-temp` flag - only `--max-temp` matters (prevents spatial jumps to hot systems)
- 2025-11-15 - GitHub Copilot - [fix] - Temperature constraint now correctly applies **only to spatial jumps** (EdgeKind::Spatial), not gate jumps
- 2025-11-15 - GitHub Copilot - [docs] - Updated ADR 0012, README.md, and USAGE.md to reflect correct temperature constraint semantics
- 2025-11-15 - GitHub Copilot - [tests] - Removed min_temperature tests and updated temperature logic validation
- 2025-11-15 - GitHub Copilot - [fix] - Fixed temperature calculation bug: star_luminosity and star_temperature columns now loaded correctly from e6c3 datasets
- 2025-11-15 - GitHub Copilot - [fix] - Updated temperature calculation to use Euclidean distance from centerX/Y/Z coordinates instead of orbitRadius
- 2025-11-15 - GitHub Copilot - [config] - Updated TemperatureModelParams defaults to match EVE Frontier formula: T_min=0.1K, T_max=99.9K, b=1.25, k=3.215e-11
- 2025-11-15 - GitHub Copilot - [fixture] - Regenerated test fixture with full e6c3 celestial data including coordinates, luminosity, and orbital parameters
- 2025-11-15 - GitHub Copilot - [tests] - All temperature tests now passing with real e6c3 data: Nod ~18.09K, Brana ~0.51K
- 2025-11-15 - GitHub Copilot - [tests] - Updated test_custom_model_near_star expectations to match EVE Frontier temperature model (max 99.9K)
- 2025-11-15 - GitHub Copilot - [bench] - Added Criterion-based pathfinding benchmarks, Makefile target, and documentation updates for running `cargo bench`
- 2025-11-15 - GitHub Copilot - [tests] - Added fixture metadata helper script, Makefile targets, docs, and integration test to keep `minimal_static_data.db` synchronized

- 2025-11-15 - auto-llm - [feature] - Compute per-system minimum external temperature at load from furthest celestial using a calibrated model and Stefan–Boltzmann; enforce via routing constraint; expose `--min-temp` in CLI
- 2025-11-15 - auto-llm - [cli] - Added `--min-temp` flag to `route` command and improved no-route suggestions to mention lowering `--min-temp`
- 2025-11-15 - auto-llm - [tests] - Added library and CLI tests for min-temperature behavior; removed a brittle failure case relying on fixture-specific data
- 2025-11-15 - auto-llm - [docs] - Created ADR 0012 documenting minimum external temperature calculation and load-time computation strategy
- 2025-11-15 - auto-llm - [docs] - Updated ADR 0009 with temperature-aware KD-tree neighbor query requirements and API shape
- 2025-11-15 - auto-llm - [docs] - Updated README and `docs/USAGE.md` with `--min-temp` examples and guidance

- 2025-11-15 - auto-llm - [docs] - Enhanced lib.rs with comprehensive module documentation and usage examples
- 2025-11-15 - auto-llm - [docs] - Added Library API section to docs/USAGE.md with code examples for common patterns
- 2025-11-15 - auto-llm - [docs] - Documented all three routing algorithms, constraint usage, error handling, and output formatting
- 2025-11-15 - auto-llm - [docs] - Marked three completed TODOs as done in TODO.md (error handling, unit tests, CHANGELOG)
- 2025-11-15 - auto-llm - [tooling] - Created .nvmrc file pinning Node 20 (LTS) for reproducible development
- 2025-11-15 - auto-llm - [tooling] - Updated CI workflows to use Rust 1.91.1 (matching .rust-toolchain file)
- 2025-11-15 - auto-llm - [docs] - Documented version pinning (.nvmrc, .rust-toolchain) in CONTRIBUTING.md
- 2025-11-15 - auto-llm - [ci] - Added nightly dependency check workflow that runs at 2 AM UTC daily
- 2025-11-15 - auto-llm - [ci] - Workflow checks Rust (cargo-outdated) and Node (pnpm outdated) dependencies
- 2025-11-15 - auto-llm - [ci] - Publishes rust-outdated-report and node-outdated-report artifacts with 30-day retention
- 2025-11-15 - auto-llm - [ci] - Workflow supports manual triggers via workflow_dispatch
- 2025-11-15 - auto-llm - [docs] - Documented dependency check workflow in CONTRIBUTING.md
- 2025-11-15 - auto-llm - [security] - Integrated cargo-audit into CI pipeline with dedicated `security-audit` job that fails on vulnerabilities
- 2025-11-15 - auto-llm - [security] - Added cargo audit to pre-commit hook (step 5) to block commits with vulnerable dependencies
- 2025-11-15 - auto-llm - [security] - Added `make audit` target for manual security scans with `--deny warnings`
- 2025-11-15 - auto-llm - [docs] - Created comprehensive security audit guide in `docs/SECURITY_AUDIT.md` documenting remediation workflows
- 2025-11-14 - auto-llm - [auto-llm] - Fixed JSON output format being polluted by tracing logs on stdout. Tracing is now suppressed when `--format json` is used, keeping stdout clean for machine-readable output.
- 2025-11-14 - auto-llm - [auto-llm] - Fixed race condition in dataset download tests by changing `download_from_source_with_cache` to require explicit `resolved_tag` parameter instead of reading from environment variables. Removed unused test helpers (`env_lock`, `with_latest_tag_override`, `LatestTagGuard`). All 28 workspace tests now pass consistently.
- 2025-11-14 - auto-llm - [auto-llm] - Consolidated the CLI around a single `route` command, moved `--format` to apply only to route output, added a footer with elapsed time and units, defaulted pathfinding to the A* planner, and ensured downloads ignore formatting flags while still honoring the global dataset/data-dir options.
- 2025-11-14 - auto-llm - [auto-llm] - Refreshed docs (README, `docs/USAGE.md`, `docs/EXAMPLES.md`, `docs/TODO.md`, ADR 0005) to describe the new CLI surface, routing formats, footer behavior, and Makefile-assisted workflow, and updated instructions to call `evefrontier-cli` directly after the release build.
- 2025-11-14 - auto-llm - [auto-llm] - Added a `Makefile` with `make test-smoke` tied to `cargo test --workspace` plus download/route smoke runs, and documented the skeletal test harness as part of the release guidance.
- 2025-11-14 - auto-llm - [auto-llm] - Improved CLI error handling so unknown systems surface fuzzy suggestions and no-route scenarios hint at constraint tweaks, and added regression tests to lock the behavior.
- 2025-11-13 - auto-llm - [auto-llm] - Added fuzzy matching for system names with Jaro-Winkler similarity to suggest corrections for typos (e.g., "F:3Z" suggests "F:3068", "F:3R6A"). Unknown system errors now include up to 3 similar system names.
- 2025-11-13 - auto-llm - [auto-llm] - Updated documentation examples to use direct `evefrontier-cli` binary invocation instead of `cargo run -p evefrontier-cli --`, improved README with release build and install instructions.
- 2025-11-12 - auto-llm - [auto-llm] - CI now generates minimal fixture fresh for each test run using `scripts/create_minimal_db.py`, eliminating dependency on git-tracked binary and preventing accidental overwrites.
- 2025-11-12 - auto-llm - [auto-llm] - Fixed CI fixture tests to use `--dataset fixture` flag with generated fixture, ensuring tests run entirely offline without hitting external dataset sources.
- 2025-11-12 - auto-llm - [auto-llm] - Fixed README `--format json` example to place global flag before subcommand (global flags must precede subcommands).
- 2025-11-12 - auto-llm - [auto-llm] - Added CI validation job that tests documentation examples from README to ensure they remain functional and accurate.
- 2025-11-12 - auto-llm - [auto-llm] - Documented `.vscode/mcp.json` GitHub Copilot MCP
  configuration in CONTRIBUTING.md, clarifying it is optional for developers and explaining
  its purpose for enhanced AI-assisted features.
- 2025-11-11 - auto-llm - [auto-llm] - Added shared route serialization helpers with plain, rich,
  and in-game note renderers, exposed them through the library, expanded the CLI with matching
  output formats, documented the new options, and tightened graph/path utilities with clarified
  constants and NaN-safe position handling.
- 2025-11-11 - auto-llm - [auto-llm] - Implemented weighted route planning with `A*` and Dijkstra
  support, added pathfinding constraints for jump distance, avoided systems, gate-free travel, and
  temperature limits, refreshed CLI/docs to reflect the new options, and extended the routing tests
  to cover the additional algorithms.
- 2025-11-11 - auto-llm - [auto-llm] - Added graph builders for gate, spatial, and hybrid routing
  modes, exposed edge metadata for upcoming pathfinders, enriched system records with optional
  coordinates, and documented the new helpers with tests.
- 2025-11-11 - auto-llm - [auto-llm] - Enriched the starmap data model with optional region and
  constellation metadata, tightened schema detection, and extended documentation and tests to cover
  the expanded surface.
- 2025-11-11 - auto-llm - [auto-llm] - Hardened the starmap loader with explicit schema validation,
  filtered invalid jump edges, and added integration tests covering the legacy dataset layout.
- 2025-11-11 - auto-llm - [auto-llm] - Expanded the CLI with `search`/`path` routing subcommands,
  introduced a library route planner with option validation, added integration and unit tests, and
  documented the new flags and usage examples.
- 2025-11-09 - auto-llm - [auto-llm] - Expanded CLI skeleton with global options (`--format`,
  `--no-logo`, `--dataset`), added structured JSON output for `route` and `download` commands,
  refactored CLI plumbing to `AppContext` and `RouteRequest` handling, bounded Windows dataset path
  normalization with helper functions and an iteration limit, added platform-aware tests for dataset
  path normalization, centered the CLI banner layout, and documented early-return coding guidelines.
- 2025-11-09 - auto-llm - [auto-llm] - Switched the dataset downloader to the
  `Scetrov/evefrontier_datasets` repository, added release tag selection (for example
  `e6c2`/`e6c3`), exposed the capability through the library and CLI, and updated documentation and
  tests.
- 2025-11-09 - auto-llm - [auto-llm] - Implemented the GitHub dataset downloader with caching and
  zip extraction, added local override support, exercised the feature with tests, and refreshed
  documentation and TODO tracking.
- 2025-11-11 - auto-llm - [auto-llm] - Detect cached latest datasets whose upstream release tag has
  changed and force a refresh so users always receive the requested release, even after updates.
- 2025-11-08 - auto-llm - [auto-llm] - Documented dataset cache locations, clarified ADR links,
  improved graph sharing semantics, and tightened CLI logging configuration.
- 2025-11-07 - auto-llm - [auto-llm] - Scaffolded the Rust workspace, added the evefrontier library
  and CLI skeleton, and introduced basic dataset loading and routing capabilities.
