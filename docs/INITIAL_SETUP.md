# Initial setup & reconstruction blueprint

This document combines the quick rebuild recipe and the reconstruction blueprint into a
single, concise guide intended for someone starting a new project or reconstructing this
workspace from scratch. It focuses on the practical steps to reproduce behaviorally equivalent
artifacts (library + CLI), the minimal project layout, tooling, and important implementation
notes.

## 1. Quick rebuild steps

Prerequisites

- Rust toolchain: see `.rust-toolchain` in the repository root (pin the toolchain for
  reproducible builds).
- Node.js: see `.nvmrc` in the repository root (used for developer tooling only).

Install the required toolchains and developer dependencies:

```pwsh
# Install the Rust toolchain pinned in .rust-toolchain
rustup toolchain install (Get-Content .rust-toolchain) ; rustup override set (Get-Content .rust-toolchain)

# Install Node.js using nvm (example)
nvm install (Get-Content .nvmrc)
nvm use (Get-Content .nvmrc)

# Install Node developer dependencies and run format/lint
pnpm install
pnpm run lint:md
pnpm run format

# Build and test the Rust workspace
cargo build --workspace
cargo test --workspace
```

Notes

- If tests require the dataset, call `ensure_dataset(Some(path), DatasetRelease::latest())` in tests
  or provide the dataset path via environment/config when running the CLI. `ensure_c3e6_dataset`
  remains available when the Era 6 Cycle 3 dataset is specifically required.
- See `docs/DEPENDENCIES.md` and `CONTRIBUTING.md` for additional setup and contribution
  guidelines.

## 2. Minimal repository layout

- Root workspace:
  - `Cargo.toml` (workspace with members under `crates/`)
  - `crates/`
    - `evefrontier-lib/` — library crate (core logic)
      - `src/db.rs` — DB loader & schema detection
      - `src/github.rs` — optional GitHub downloader helper
      - `src/graph.rs` — graph construction from systems/jumps
      - `src/path.rs` — pathfinding algorithm(s)
      - `src/lib.rs` — public API and re-exports
    - `evefrontier-cli/` — CLI crate
      - `src/main.rs` — argument parsing and minimal glue to call library

## 3. Public library API (recommended)

Implement a small, stable public surface so other tools and Lambdas can reuse behavior:

- `ensure_dataset(target_dir: Option<&Path>, release: DatasetRelease) -> Result<PathBuf>`
  - Downloads or locates the requested dataset DB and returns an absolute path.
  - Accepts either a path to an existing `.db` file or downloads a release asset from GitHub,
    caching under the OS cache dir (e.g. `directories::BaseDirs::cache_dir()/evefrontier_datasets/`).
  - Download to a temporary file and atomically rename to the final filename on success.
  - `ensure_c3e6_dataset` is a helper that pins the release to `DatasetRelease::tag("e6c3")`.

- `load_starmap(db_path: &Path) -> Result<Starmap>`
  - Loads systems and jumps into in-memory structures. `Starmap` should contain:
    - `systems`: `HashMap<SystemId, System { id, name }>`
    - `adjacency`: a mapping from `SystemId` to a vector of neighbour `SystemId`s

- `build_gate_graph(starmap: &Starmap) -> Graph` — build the gate-only graph used by the
  unweighted pathfinder.
- `build_spatial_graph(starmap: &Starmap) -> Graph` — build a spatial jump graph using system
  coordinates.
- `build_hybrid_graph(starmap: &Starmap) -> Graph` — combine gate and spatial edges for richer
  routing options.

- `find_route(graph: &Graph, start: SystemId, goal: SystemId) -> Option<Vec<SystemId>>` — returns
  an ordered list of system IDs that forms the route or `None` when no path exists.

## 4. CLI surface

- Subcommands (minimum):
- `download` — ensure dataset present (wraps `ensure_dataset`).
  - `route <SYSTEM>` — compute a route from a named system using the library API.

- Data path resolution order:
  1. `--data-dir` CLI flag
  2. `EVEFRONTIER_DATA_DIR` env var
  3. XDG `ProjectDirs` default path
  4. Fallback `~/.local/evefrontier/static_data.db`

## 5. Database schema expectations & detection

- Supported schemas:
  - `static_data.db` style: tables `SolarSystems(solarSystemId, name)` and
    `Jumps(fromSystemId, toSystemId)`.
  - Older style: `mapSolarSystems` and equivalent jumps tables.

- Detection approach:
  - Use `PRAGMA table_info('SolarSystems')` or query `sqlite_master` to find table names and
    column patterns. Based on results, select the appropriate SQL query set.

## 6. Downloader & caching

- Store downloaded assets under the OS cache directory (via the `directories` crate) in
  `evefrontier_datasets/`.
- Download behavior:
  - Download to a temporary file in the cache dir.
  - Validate the download (size / presence) and then atomically rename to the final filename.
  - If a release is a zip, extract the first `.db` matching `*.db` or containing `c3e6` in the name.

## 7. Tests and fixtures

- Include at least one small SQLite fixture per supported schema (4–10 systems) so loader and
  pathfinding logic can be exercised.
- Unit tests to add:
  - Loader tests validating detection and counts of systems/jumps.
  - Graph tests checking adjacency and a route-finding test (happy path).

## 8. Build, tooling & pinning

- Build commands:
  - `cargo build --workspace`
  - `cargo test --workspace`

- Tooling to document and pin:
  - `.rust-toolchain` with the pinned Rust compiler version.
  - `.nvmrc` or Volta config for Node used by docs tooling.

## 9. Release & signing

- Sign artifacts using `gpg` or `cosign`. Attach attestations for build/test and scan steps.
  Document exact signature commands and artifact layout in `docs/RELEASE.md` when you design the
  final release process.

## 10. Minimal implementation notes

- Pathfinding: start with BFS on an unweighted graph of systems (edges derived from `Jumps`).
  Switch to Dijkstra if weights are required later.
- System identifiers: use `INTEGER` IDs from the DB as primary keys and maintain a
  `HashMap<i64, String>` for `id -> name`.

## Change log

- 2025-11-08: Created `INITIAL_SETUP.md` by consolidating `REBUILD.md` and `RECONSTRUCT.md`.
