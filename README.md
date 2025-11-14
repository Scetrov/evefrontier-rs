# EveFrontier Rust Workspace

This repository contains a Rust workspace for working with the EveFrontier static dataset. It
provides:

- `evefrontier-lib` — a reusable library that knows how to locate the dataset, load the starmap into
  memory, build graphs, and run pathfinding algorithms.
- `evefrontier-cli` — a command line interface that wraps the library APIs and exposes developer
  utilities such as route computation.
- (Planned) AWS Lambda crates for serverless endpoints that reuse the same library code.

The workspace follows the structure documented in [`docs/INITIAL_SETUP.md`](docs/INITIAL_SETUP.md)
and the accepted ADRs under [`docs/adrs/`](docs/adrs/).

## Getting started

1. Ensure the Rust toolchain pinned in [`.rust-toolchain`](.rust-toolchain) is installed:

   ```bash
   rustup toolchain install $(cat .rust-toolchain)
   rustup override set $(cat .rust-toolchain)
   ```

2. Build the workspace:

   ```bash
   cargo build --workspace
   ```

   Optional: build a release binary for faster startup:

   ```bash
   cargo build -p evefrontier-cli --release
   # Binary: target/release/evefrontier-cli
   # Or install globally:
   cargo install --path crates/evefrontier-cli
   ```

3. Run tests:

   ```bash
   cargo test --workspace
   ```

4. Run the CLI (it will download the dataset automatically on first use):

   ```bash
   evefrontier-cli download
   evefrontier-cli route --from "Y:170N" --to "BetaTest"
   ```

  The `--data-dir` flag accepts either a directory (the dataset filename will be appended) or a path
  to a `.db` file. If omitted, the CLI resolves the dataset location using the order described in
  [`docs/INITIAL_SETUP.md`](docs/INITIAL_SETUP.md). Pass `--dataset e6c2` (for example) to download a
  specific dataset tag; otherwise the CLI downloads the latest release from
  [`Scetrov/evefrontier_datasets`](https://github.com/Scetrov/evefrontier_datasets).

- Use `--format json` with `route` to emit machine-readable output for integrations:

  ```pwsh
  evefrontier-cli --format json route --from "Y:170N" --to "BetaTest"
  ```

- Use `--format basic` (or `--format note`) when you need a minimal path listing for pasting into
  other tools:

  ```pwsh
  evefrontier-cli --format basic route --from "Y:170N" --to "BetaTest"
  ```

- Apply `--format rich` for Markdown-style bullet output or `--format note` to emit an in-game note
  listing the systems in order.

All routing subcommands accept `--algorithm`, `--max-jump`, `--avoid`, `--avoid-gates`, and
`--max-temp` flags. The library supports breadth-first search (`bfs`), Dijkstra (`dijkstra`), and
heuristic-guided A* (`a-star`) algorithms alongside jump distance limits, system avoidance,
gate-free traversal, and temperature caps.

## Library highlights

- `ensure_dataset` — resolves the dataset path using CLI arguments, environment variables, or
  platform-specific defaults, downloads the requested dataset release (latest by default), and caches
  it under the OS cache directory. `ensure_c3e6_dataset` remains available as a convenience wrapper
  for the Era 6 Cycle 3 dataset.
- `load_starmap` — loads systems and jumps from the SQLite database with basic schema detection.
- `plan_route` — converts system names into IDs, validates routing options, and plans a route using
  breadth-first search, Dijkstra, or A* while applying distance, avoidance, gate, and temperature
  constraints. Lower-level helpers such as `build_graph`/`find_route_bfs` remain available when
  needed.

See [`docs/TODO.md`](docs/TODO.md) for the comprehensive backlog covering the downloader, advanced
pathfinding options, Lambda integration, and additional tooling.

## Contributing

Please review [`CONTRIBUTING.md`](CONTRIBUTING.md) and the ADRs before submitting changes. All code
changes must add an entry to [`CHANGELOG.md`](CHANGELOG.md).
