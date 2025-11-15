# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

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
