# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

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
