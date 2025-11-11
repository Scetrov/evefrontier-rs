# Project TODOs

This TODO list captures the remaining work required to implement the EveFrontier CLI, library,
AWS Lambda functions, and supporting infrastructure described throughout the documentation and ADRs.
Tasks are grouped by domain; checkboxes track completion status.

## Workspace & Tooling

- [ ] Establish the Cargo workspace layout with `crates/evefrontier-lib`, `crates/evefrontier-cli`,
      and Lambda crates (for example `crates/evefrontier-lambda-route`, `crates/evefrontier-lambda-scout-gates`,
      `crates/evefrontier-lambda-scout-range`). Ensure shared code lives in the library crate.
- [ ] Configure Nx to orchestrate Rust, lint, and security tasks (align with [ADR 0006](adrs/0006-software-components.md) and
      `.github/copilot-instructions.md`). Define tasks for `build`, `test`, `lint`, `clippy`,
      `audit`, and dependency update reporting.
- [ ] Add `package.json`, `pnpm-lock.yaml`, and Nx project configuration. Document developer
      commands in `CONTRIBUTING.md` and `docs/USAGE.md`.
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
- [ ] Build graph construction helpers (`graph.rs`) that transform the `Starmap` into search graphs
      for gate, spatial, or hybrid routing modes.
- [ ] Implement pathfinding algorithms in `path.rs`: breadth-first search for unweighted graphs,
      Dijkstra for weighted routes, and A* for heuristic-guided searches. Support filters such as
      maximum jump distance, gate-only routes, spatial routes, avoided systems, and temperature
      constraints.
- [ ] Provide serialization helpers for CLI/Lambda outputs (plain text, rich text, JSON, in-game
      note format) with appropriate structs and enums.
- [ ] Add robust error handling via a shared `Error` type (using `thiserror`) and bubble errors to
      callers with actionable messages.
- [ ] Write unit tests covering schema detection, dataset loading, graph construction, and routing
      behavior using fixtures in `docs/fixtures/`.
- [ ] Document the public API in `docs/USAGE.md` and Rustdoc comments.

## CLI (`evefrontier-cli`)

- [x] Implement the CLI skeleton with Clap, including global `--data-dir`, `--format`, `--no-logo`,
      and other shared options. Respect the data path resolution order defined in `docs/INITIAL_SETUP.md`.
- [x] Implement the `download` subcommand that wraps `ensure_c3e6_dataset`, reports the resolved path,
      and exits with appropriate codes.
- [x] Implement routing subcommands (`route`, `search`, `path`) that call into the library to build
      the graph and produce formatted output, including optional arguments (`--algorithm`,
      `--max-jump`, `--avoid`, `--avoid-gates`, `--max-temp`).
- [ ] Provide friendly error messages for unknown systems and route failures.
- [ ] Add integration tests for CLI behavior (using `assert_cmd` or similar) with the fixture dataset.
- [ ] Update `README.md` and `docs/USAGE.md` with CLI examples that match the implemented behavior.

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

## Testing & Quality

- [ ] Ensure `cargo fmt`, `cargo clippy --all-targets --all-features`, and `cargo test --workspace`
      run cleanly; hook them into Nx and CI ([ADR 0007](adrs/0007-devsecops-practices.md)).
- [ ] Add dataset fixture management helpers to keep fixtures synchronized and documented in
      `docs/fixtures/README.md`.
- [ ] Integrate `cargo audit` and Node SCA checks into CI and document remediation workflows.
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

## Security & Operations

- [ ] Define secrets management and configuration guidance for Lambda deployments (environment
      variables, AWS Secrets Manager, etc.).
- [ ] Implement logging and observability hooks (structured logs, optional tracing) across CLI and
      Lambdas.
- [ ] Add metrics or usage telemetry (if desired) with opt-in controls and documentation.
- [ ] Plan for dataset update automation (scheduled job or manual release) and document operational
      runbooks.

---

This checklist should be updated as work progresses. When tasks are completed, mark the relevant
checkboxes and add links to PRs or documentation updates.
