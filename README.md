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

3. Run tests:

   ```bash
   cargo test --workspace
   ```

4. Run the CLI (use the bundled fixture until the downloader is implemented):

   ```bash
   cargo run -p evefrontier-cli -- route --from "Y:170N" --to "BetaTest" --data-dir docs/fixtures/minimal_static_data.db
   ```

   The `--data-dir` flag accepts either a directory (the dataset filename will be appended) or a path
   to a `.db` file. If omitted, the CLI resolves the dataset location using the order described in
   [`docs/INITIAL_SETUP.md`](docs/INITIAL_SETUP.md).

## Library highlights

- `ensure_c3e6_dataset` — resolves the dataset path using CLI arguments, environment variables, or
  platform-specific defaults, and (eventually) downloads the dataset if needed.
- `load_starmap` — loads systems and jumps from the SQLite database with basic schema detection.
- `build_graph` and `find_route` — construct an adjacency graph and compute simple breadth-first
  routes between two systems.

See [`docs/TODO.md`](docs/TODO.md) for the comprehensive backlog covering the downloader, advanced
pathfinding options, Lambda integration, and additional tooling.

## Contributing

Please review [`CONTRIBUTING.md`](CONTRIBUTING.md) and the ADRs before submitting changes. All code
changes must add an entry to [`CHANGELOG.md`](CHANGELOG.md).
