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

The CLI currently exposes four subcommands (`download`, `route`, `search`, and `path`) while the
richer surface outlined in ADR 0005 is implemented.

Preferred invocation for end users and scripts:

```pwsh
evefrontier-cli <subcommand> [args]
```

Developer invocation (build-and-run via cargo):

```pwsh
cargo run -p evefrontier-cli -- <subcommand> [args]
```

Global options accepted by every subcommand:

- `--data-dir <PATH>` — override the dataset directory or file.
- `--dataset <TAG>` — request a specific dataset release.
- `--format <text|json>` — control the command output format (`text` by default).
- `--no-logo` — suppress the ASCII banner (automatically implied for JSON output).

### Examples

- Download the latest dataset to the default location resolved by the CLI and report the path:

  ```pwsh
  evefrontier-cli download
  ```

- Download a specific dataset tag into a custom directory (helpful for tests or fixtures):

  ```pwsh
  evefrontier-cli download --data-dir docs/fixtures --dataset e6c3
  ```

- Retrieve dataset metadata as JSON for scripting workflows:

  ```pwsh
  evefrontier-cli download --format json
  ```

- Calculate a route between two systems using an existing dataset path:

  ```pwsh
  evefrontier-cli route --from "Y:170N" --to "BetaTest" --data-dir docs/fixtures/minimal_static_data.db
  ```

- Request structured route output suitable for downstream tooling:

  ```pwsh
  evefrontier-cli route --from "Y:170N" --to "BetaTest" --format json
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

### `search`

Runs the same breadth-first algorithm but labels the output as a search result, which is helpful when
debugging routing options or consuming the JSON response in tooling.

```pwsh
cargo run -p evefrontier-cli -- search --from "Y:170N" --to "BetaTest" --format json --data-dir docs/fixtures/minimal_static_data.db
```

### `path`

Outputs the raw path between two systems using an arrow-delimited format that is easier to pipe into
scripts.

```pwsh
cargo run -p evefrontier-cli -- path --from "Y:170N" --to "BetaTest" --data-dir docs/fixtures/minimal_static_data.db
```

### Routing options

The routing subcommands accept additional flags for future functionality:

- `--algorithm <bfs|dijkstra|a-star>` — select the pathfinding algorithm. At present only `bfs` is
  implemented; selecting another algorithm produces a clear error message.
- `--max-jump <LIGHT-YEARS>` — limit the jump distance when the algorithm supports it (not yet
  available).
- `--avoid <SYSTEM>` — avoid specific systems by name. Repeat the flag to provide more than one
  entry.
- `--avoid-gates` — prefer non-gate traversal modes (not yet supported).
- `--max-temp <KELVIN>` — constrain the maximum temperature of systems along the route.

Unsupported options are rejected by the library so users receive immediate feedback instead of silent
failures.

## Configuration & data path resolution

The CLI resolves the data path in the following order:

1. CLI `--data-dir` flag (if provided)
2. `EVEFRONTIER_DATA_DIR` environment variable
3. XDG `directories::ProjectDirs` default location. Examples:
   - Linux: `~/.local/share/evefrontier/static_data.db`
   - macOS: `~/Library/Application Support/com.evefrontier.evefrontier/static_data.db`
   - Windows: `%APPDATA%\\evefrontier\\data\\static_data.db`

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
