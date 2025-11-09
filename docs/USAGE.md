# EveFrontier CLI, Lambda & Library — Usage

This document describes how to build and use the `evefrontier-cli` workspace and its library crate
`evefrontier-lib`. Lambda crates will reuse the same APIs once they are implemented. Refer to
[`docs/TODO.md`](TODO.md) for the remaining backlog.

## Build

1. Build the entire workspace:

   ```pwsh
   cargo build --workspace
   ```

2. Build only the library and CLI crates:

   ```pwsh
   cargo build -p evefrontier-lib -p evefrontier-cli
   ```

## Run the CLI

The CLI currently exposes two subcommands (`download` and `route`) while the richer surface outlined
in ADR 0005 is implemented.

Preferred invocation for end users and scripts:

```pwsh
evefrontier-cli <subcommand> [args]
```

Developer invocation (build-and-run via cargo):

```pwsh
cargo run -p evefrontier-cli -- <subcommand> [args]
```

### Examples

- Download the latest dataset to the default location resolved by the CLI and report the path:

  ```pwsh
  evefrontier-cli download
  ```

- Download a specific dataset tag into a custom directory (helpful for tests or fixtures):

  ```pwsh
  evefrontier-cli download --data-dir docs/fixtures --dataset e6c3
  ```

- Calculate a route between two systems using an existing dataset path:

  ```pwsh
  evefrontier-cli route --from "Y:170N" --to "BetaTest" --data-dir docs/fixtures/minimal_static_data.db
  ```

- Calculate a route after pre-setting the dataset path via environment variable:

  ```pwsh
  $env:EVEFRONTIER_DATA_DIR = (Resolve-Path docs/fixtures/minimal_static_data.db).Path
  evefrontier-cli route --from "Y:170N" --to "BetaTest"
  ```

### `download`

Ensures the requested dataset is present on disk and reports the resolved path. The command downloads
the specified dataset release (or reuses the cached copy) and writes it to the resolved location. When
`--dataset` is omitted the downloader uses the latest release from
[`Scetrov/evefrontier_datasets`](https://github.com/Scetrov/evefrontier_datasets).

```pwsh
cargo run -p evefrontier-cli -- download --data-dir docs/fixtures
```

### `route`

Computes a simple breadth-first route between two system names using the loaded dataset. If the
dataset is not already present, the CLI will download it automatically before computing the route.

```pwsh
cargo run -p evefrontier-cli -- route --from "Y:170N" --to "BetaTest" --data-dir docs/fixtures/minimal_static_data.db
```

## Configuration & data path resolution

The CLI resolves the data path in the following order:

1. CLI `--data-dir` flag (if provided)
2. `EVEFRONTIER_DATA_DIR` environment variable
3. XDG `directories::ProjectDirs` default location. Examples:
   - Linux: `~/.local/share/evefrontier/static_data.db`
   - macOS: `~/Library/Application Support/com.evefrontier.evefrontier/static_data.db`
   - Windows: `%LOCALAPPDATA%\\EveFrontier\\static_data.db`

If the dataset is absent in all locations, the library will attempt to download it automatically.

## Library API

Key library entrypoints (in `crates/evefrontier-lib`):

- `ensure_dataset(target_dir: Option<&Path>, release: DatasetRelease)` — resolves or downloads the
  dataset release identified by `release`. The optional path argument allows tests to point at
  fixture data or custom paths. `ensure_c3e6_dataset` is still available as a shorthand for
  `DatasetRelease::tag("e6c3")`.
- `load_starmap(db_path: &Path)` — loads systems and jumps into memory with schema detection for the
  `SolarSystems`/`Jumps` schema.
- `build_graph` / `find_route` — build the adjacency graph and compute unweighted routes using BFS.

## Testing

Run unit tests across the workspace:

```pwsh
cargo test --workspace
```

The library test suite uses the bundled fixture located at `docs/fixtures/minimal_static_data.db`.
You can reuse the same file when running the CLI by passing `--data-dir docs/fixtures/minimal_static_data.db`.

### Local dataset overrides

For development and testing you can override the GitHub download by setting the
`EVEFRONTIER_DATASET_SOURCE` environment variable to a local path. The path may point to either a
`.db` file or a `.zip` archive containing the database. When set, `ensure_dataset` (and convenience
wrappers like `ensure_c3e6_dataset`) copy or extract the local file instead of contacting GitHub.

```pwsh
$env:EVEFRONTIER_DATASET_SOURCE = "docs/fixtures/minimal_static_data.db"
cargo run -p evefrontier-cli -- download --data-dir target/fixtures
```
