# EveFrontier CLI, Lambda & Library — Usage

This document describes how to build and use the `evefrontier-cli` or
`evefrontier-lambda-xxx` workspace and its library crate `evefrontier-lib`.

Build

1. Build the entire workspace:

   cargo build --workspace

Build

1. Build the entire workspace

```pwsh
cargo build --workspace
```

2. Build only the library and CLI crates

```pwsh
cargo build -p evefrontier-lib -p evefrontier-cli
```

Run the CLI

Preferred invocation for end users and scripts:

```pwsh
evefrontier-cli <subcommand> [args]
```

Developer invocation (build-and-run via cargo):

```pwsh
cargo run -p evefrontier-cli -- <subcommand> [args]
```

Examples

- Download dataset (places DB at resolved path):

```pwsh
evefrontier-cli download
```

- Compute a search path starting at a system name using only gates:

```pwsh
evefrontier-cli search "P:STK3" --gate-only
```

- Compute a search path starting at a system name using spatial jumps:

```pwsh
evefrontier-cli search "P:STK3" --spatial
```

- Compute a search path starting at a system name using either spatial or gate jumps:

```pwsh
evefrontier-cli search "P:STK3"
```

- Calculate a route between two systems

```pwsh
evefrontier-cli path --from "P:STK3" --to "Strym" --algorithm dijkstra --max-jump 80ly
```

Optional route parameters (examples and semantics):

- `--algorithm` — algorithm to use: `dijkstra` or `astar` (default: `dijkstra`).
- `--max-jump <distance>` — maximum jump distance. Accepts a number with optional `ly` suffix
  (examples: `80`, `80ly`, `80.0ly`). Default: `80ly`.
- `--avoid <list>` — comma-separated list of system names to avoid. Wrap the value in quotes
  if it contains spaces. Example: `--avoid "Strym,P:STK3"`.
- `--avoid-gates` — boolean flag to avoid gate jumps; use without a value.
- `--max-temp <kelvin>` — maximum temperature in Kelvin (integer). Example: `--max-temp 1200`.

## Common Parameters

- `--format <format>` — output format. One of:
  - `ingame` — EVE Frontier in-game notepad format.
  - `rich` — human-friendly output with tables and emoji (default).
  - `plain` — plain text without emoji.
  - `json` — machine-readable JSON output.
- `--no-logo` — skip displaying the application logo.

## Configuration & data path resolution

The CLI resolves the data path in the following order:

1. CLI `--data-dir` flag (if provided)
2. `EVEFRONTIER_DATA_DIR` environment variable
3. XDG `directories::ProjectDirs` default location
4. Fallback to `~/.local/evefrontier/static_data.db`

Downloader & caching

The downloader stores cached release assets under the OS cache directory in a
`evefrontier_datasets/` subdirectory. It writes to a temporary file and then atomically
renames it to the final path to avoid partial writes.

Database schema compatibility

The library detects supported DB schemas and adapts queries at load time. Currently supported
variants include the `static_data.db` schema (tables `SolarSystems(solarSystemId, name)` and
`Jumps(fromSystemId, toSystemId)`) and the older `mapSolarSystems` schema. If you add support
for additional schemas, update `crates/evefrontier-lib/src/db.rs` and add unit tests under
`crates/evefrontier-lib/tests/`.

Library API

Key library entrypoints (in `crates/evefrontier-lib`):

- `ensure_c3e6_dataset(target_dir: Option<&Path>)` — download and ensure dataset is present.
  The optional `target_dir` argument is useful for deterministic tests.
- Graph & path functions live in `graph.rs` and `path.rs` and expose the route-finding APIs used
  by the CLI and Lambdas.

Testing

Run unit tests across the workspace:

```pwsh
cargo test --workspace
```

If tests require the dataset, call `ensure_c3e6_dataset(Some(path))` to place the DB in a
deterministic location during test runs.

Test fixture

- A small SQLite fixture is included in the repository as `minimal_static_data.db`. Use this for
  deterministic unit tests and CI.

Example (PowerShell):

```pwsh
# point the CLI/library at the fixture for tests
$env:EVEFRONTIER_DATA_DIR = (Resolve-Path .\minimal_static_data.db).Path
cargo test --workspace
```

Notes

- For contributors, prefer the `cargo run` developer invocation when iterating on code; for
  scripting and production usage prefer the `evefrontier-cli` binary invocation.
