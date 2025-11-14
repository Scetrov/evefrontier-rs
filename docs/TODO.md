# Project TODOs

This TODO list captures the remaining work required to implement the EveFrontier CLI, library,
AWS Lambda functions, and supporting infrastructure described throughout the documentation and ADRs.
Tasks are grouped by domain; checkboxes track completion status.

## 🔥 High Priority - Test & Documentation Fixes

- [ ] **CRITICAL**: Ensure test fixture protection is maintained
  - The 3-system synthetic fixture (Y:170N, AlphaTest, BetaTest) must not be overwritten by CLI downloads
  - Tests currently use these synthetic systems and pass in CI; local overwrites with production data (e6c3) cause failures
  - All tests in `crates/evefrontier-lib/tests/routing.rs` and `crates/evefrontier-cli/tests/route_commands.rs` correctly reference the fixture systems
  - Documentation examples in `README.md` and `docs/USAGE.md` already use the correct fixture system names (Y:170N, BetaTest)
- [x] Add `.gitignore` entry for `*.db.release` marker files (ephemeral download metadata; implemented as a global pattern)
- [ ] Add fixture protection to prevent accidental overwrites:
  - [x] Document in `docs/fixtures/README.md` that fixture should not be replaced with downloads
  - Consider adding `docs/fixtures/.gitattributes` to lock the minimal fixture
  - OR: Decide to use real dataset as canonical fixture and update create_minimal_db.py accordingly
- [x] Document `.vscode/mcp.json` in README or CONTRIBUTING guide (GitHub Copilot MCP server config; clarify if required or optional for developers)
- [x] Add CI validation step that runs example commands from README/USAGE docs to ensure they continue working
## Workspace & Tooling

- [ ] Establish the Cargo workspace layout with `crates/evefrontier-lib`, `crates/evefrontier-cli`,
      and Lambda crates (for example `crates/evefrontier-lambda-route`, `crates/evefrontier-lambda-scout-gates`,
      `crates/evefrontier-lambda-scout-range`). Ensure shared code lives in the library crate.
- [ ] Configure Nx to orchestrate Rust, lint, and security tasks (align with [ADR 0006](adrs/0006-software-components.md) and
      `.github/copilot-instructions.md`). Define tasks for `build`, `test`, `lint`, `clippy`,
      `audit`, and dependency update reporting.
- [ ] Scaffold Node/Nx workspace: add `package.json`, `pnpm-lock.yaml`, `nx.json`, and project
      targets for Rust crates and scripts (ADR 0006 & 0007 alignment).
- [ ] Add `package.json`, `pnpm-lock.yaml`, and Nx project configuration. Document developer
      commands in `CONTRIBUTING.md` and `docs/USAGE.md`.
- [ ] Add CI workflow enforcing ADR filename pattern (`^\\d{4}-.+\\.md$`) and immutability (reject
      edits to historical ADRs except via explicit override label).
- [ ] Create reproducible toolchain pins for Node (`.nvmrc` or Volta config) and confirm
      `.rust-toolchain` matches the intended compiler release.
- [ ] Introduce automation scripts under `scripts/` (e.g., dataset fixture sync, release helpers)
      and register them as Nx tasks if applicable.

## Library (`evefrontier-lib`)

- [x] Implement `ensure_c3e6_dataset` to download the latest dataset release from GitHub, cache it
      in the OS cache directory under `evefrontier_datasets/`, perform atomic writes, and optionally
      extract `.zip` archives ([ADR 0003](adrs/0003-downloader-caching.md)).
- [x] Support injecting a pre-existing dataset path (for tests) and allow callers to override the
      cache location.
- [x] Implement `load_starmap` with runtime schema detection ([ADR 0004](adrs/0004-schema-detection.md)) for both `SolarSystems` / `Jumps`
      and legacy `mapSolarSystems` datasets.
- [x] Define the `Starmap` data model, including system metadata, adjacency lists, and a
      `HashMap` for name-to-ID lookups.
- [x] Build graph construction helpers (`graph.rs`) that transform the `Starmap` into search graphs
      for gate, spatial, or hybrid routing modes.
- [x] Implement pathfinding algorithms in `path.rs`: breadth-first search for unweighted graphs,
      Dijkstra for weighted routes, and A* for heuristic-guided searches. Support filters such as
      maximum jump distance, gate-only routes, spatial routes, avoided systems, and temperature
      constraints.
- [x] Provide serialization helpers for CLI/Lambda outputs (plain text, rich text, JSON, in-game
      note format) with appropriate structs and enums.
- [ ] Add robust error handling via a shared `Error` type (using `thiserror`) and bubble errors to
      callers with actionable messages.
- [ ] Write unit tests covering schema detection, dataset loading, graph construction, and routing
      behavior using fixtures in `docs/fixtures/`.
- [ ] Document the public API in `docs/USAGE.md` and Rustdoc comments.
- [ ] Implement KD-tree spatial index module (per ADR 0009): build, serialize (e.g., postcard +
      zstd), load, query nearest systems.
- [ ] Integrate KD-tree spatial index into spatial/hybrid routing path selection logic.
- [ ] Provide tests/benchmarks for KD-tree build and query performance.

## CLI (`evefrontier-cli`)

- [x] Implement the CLI skeleton with Clap, including global `--data-dir`, `--format`, `--no-logo`,
      and other shared options. Respect the data path resolution order defined in `docs/INITIAL_SETUP.md`.
- [x] Implement the `download` subcommand that wraps `ensure_c3e6_dataset`, reports the resolved path,
      and exits with appropriate codes.
- [x] Implement unified `route` subcommand (replacing earlier `search` / `path`) exposing all
      routing functionality via flags: `--algorithm`, `--format`, `--max-jump`, `--avoid`,
      `--avoid-gates`, `--max-temp`.
- [x] Provide friendly error messages for unknown systems and route failures.
- [ ] Add integration tests for CLI behavior (using `assert_cmd` or similar) with the fixture dataset.
- [ ] Update `README.md` and `docs/USAGE.md` with CLI examples that match the implemented behavior.
- [ ] Add `index-build` (or `build-index`) subcommand to precompute KD-tree spatial index artifact.
- [ ] Surface friendly errors when spatial index missing but requested.

## AWS Lambda crates

- [ ] Scaffold Lambda crates (e.g., `evefrontier-lambda-route`, `evefrontier-lambda-scout-gates`,
      `evefrontier-lambda-scout-range`) that depend on the library crate.
- [ ] Implement shared bootstrap logic to download or locate the dataset at cold start and share it
      across invocations (per `.github/copilot-instructions.md`).
- [ ] Define request/response models using `serde` and ensure JSON serialization matches API
      contracts.
- [ ] Wire Lambda handlers (using `lambda_runtime` or `aws_lambda_events`) to call library APIs and
      return results with structured errors.
- [ ] Provide infrastructure notes or SAM/CDK templates (if required) for deployment under `docs/`.
- [ ] Add Lambda-focused tests (unit tests and, if possible, integration tests using `lambda_runtime::run` mocks).
- [ ] Integrate KD-tree spatial index loading at cold start if artifact bundled.

## Testing & Quality

- [ ] Ensure `cargo fmt`, `cargo clippy --all-targets --all-features`, and `cargo test --workspace`
      run cleanly; hook them into Nx and CI ([ADR 0007](adrs/0007-devsecops-practices.md)).
- [ ] Add dataset fixture management helpers to keep fixtures synchronized and documented in
      `docs/fixtures/README.md`.
- [ ] Integrate `cargo audit` and Node SCA checks into CI and document remediation workflows.
- [ ] Add CI guard requiring `CHANGELOG.md` modification for non-doc code changes (ADR 0010).
- [ ] Schedule nightly dependency outdated report (Rust & Node) and publish artifact.
- [ ] Establish benchmarking or profiling harnesses for pathfinding performance (optional but
      recommended).

## Documentation & Communication

- [ ] Create `CHANGELOG.md` with an `Unreleased` section and update it for each change (per
      `CONTRIBUTING.md`).
- [ ] Align `README.md` with the new workspace layout, CLI commands, Lambda crates, and development
      workflow.
- [ ] Expand `docs/USAGE.md` with Lambda invocation examples and dataset caching behavior.
- [ ] Document release and signing procedures in `docs/RELEASE.md`, including cosign/GPG commands and
      attestation steps (ADR 0007).
- [ ] Add architecture diagrams or sequence diagrams illustrating data flow between downloader,
      loader, graph, CLI, and Lambda components.
- [ ] Provide onboarding steps in `docs/INITIAL_SETUP.md` once the workspace scaffolding stabilizes
      (update as tasks complete).
- [ ] Extend `docs/USAGE.md` with KD-tree index usage and build instructions.
- [ ] Add ADR for any deviations from original KD-tree design if implementation adjustments occur.
- [ ] Add `docs/RELEASE.md` section describing inclusion of spatial index artifact.

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

---

This checklist should be updated as work progresses. When tasks are completed, mark the relevant
checkboxes and add links to PRs or documentation updates.
