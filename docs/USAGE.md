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

- Download the dataset to the default location resolved by the CLI and report the path:

  ```pwsh
  evefrontier-cli download
  ```

- Download the dataset into a specific directory (helpful for tests or fixtures):

  ```pwsh
  evefrontier-cli download --data-dir docs/fixtures
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

Ensures the dataset is present on disk and reports the resolved path. The downloader implementation
is still pending, so the command will return an error until the feature is completed.

```pwsh
cargo run -p evefrontier-cli -- download --data-dir docs/fixtures
```

### `route`

Computes a simple breadth-first route between two system names using the loaded dataset. Provide a
path to the dataset explicitly until the downloader is implemented or set `EVEFRONTIER_DATA_DIR` to
point at an existing `.db` file.

```pwsh
cargo run -p evefrontier-cli -- route --from "Y:170N" --to "BetaTest" --data-dir docs/fixtures/minimal_static_data.db
```

## Configuration & data path resolution

The CLI resolves the data path in the following order:

1. CLI `--data-dir` flag (if provided)
2. `EVEFRONTIER_DATA_DIR` environment variable
3. XDG `directories::ProjectDirs` default location (`<platform data dir>/static_data.db`)

If the dataset is absent in all locations, the library will attempt to download it (feature pending).

## Library API

Key library entrypoints (in `crates/evefrontier-lib`):

- `ensure_c3e6_dataset(target_dir: Option<&Path>)` — resolves or downloads the dataset (download not
  yet implemented). The optional argument allows tests to point at fixture data.
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
