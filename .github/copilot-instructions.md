## Quick orientation

This repository is a small Rust workspace that provides a shared library, a CLI and a AWS Lambda
crates for working with EVE Frontier static datasets.

- Root workspace: `Cargo.toml` (workspace members under `crates/`).
- Library crate: `crates/evefrontier-lib/` — contains core logic: `db.rs`, `github.rs` (downloader),
  `graph.rs`, `path.rs`.
- CLI crate: `crates/evefrontier-cli/` — thin CLI glue using `clap` that calls library APIs.
  Examples: `download` and `pathfinder` subcommands.
- Lambda crates: `crates/evefrontier-lambda-xxx/` — AWS Lambda functions for each endpoint: `route`,
  `scout-gates`, `scout-range`,
- Temporary helpers/examples live under `examples/` (small DB inspection helpers).

If you are an AI coding agent making changes, prefer modifying and testing code in
`crates/evefrontier-lib`/`crates/evefrontier-lambda-xxx` and `crates/evefrontier-cli` rather than
editing the original single-file binary in `src/`.

> [!IMPORTANT] This repository is predominantly rust code, and uses NX to manage the workspace, see
> https://nx.dev/docs/guides/nx-release/publish-rust-crates for further information.

Note: Before starting any change, read this file (`.github/copilot-instructions.md`) and then check
the `docs/` and `docs/adrs/` folders for relevant documentation or ADRs that could affect your
design, implementation, or release packaging choices. These documents often contain conventions,
schema notes, and CI guidance that should be considered before making changes.

> [!IMPORTANT]
> Security and compliance requirements are mandatory. Before making any change, you MUST also read and follow
> the GitHub Copilot Security and Compliance Instructions: #file:./copilot-security-instructions.md
> In particular, do not bypass GPG signing, branch protections, or other repository security controls.

## Overview of development loop

Ensure that you follow a Boyd Loop style of development:

1. **Observe**: Read the relevant documentation files in `docs/` (especially ADRs) to understand the
   architecture and design decisions, combine this within this document and any other instructions
   provided in chat.
2. **Orient**: Familiarize yourself with the code structure in `crates/evefrontier-lib` and
   `crates/evefrontier-cli`. Identify which parts of the codebase are relevant to the task at hand.
   Use available MCP servers to gather additional information.
3. **Decide**: Plan your changes carefully, considering how they fit within the existing
   architecture and design principles outlined in the ADRs, ensure that the decision is focused with
   the minimal changes required to achieve the goal. Document any key decisions as ADRs.
4. **Act**: Implement the changes in small, incremental steps. After each change, run tests and
   build the project to ensure that everything works as expected, continue to iterate through the
   Boyd Loop as necessary until all tasks on the todo list are complete.

> [!IMPORTANT] At any point during the loop it may be nessecary to add a new TODO item to
> #file:../docs/TODO.md to allow the current session to focus on on a fixed scope of work deffering
> a task to a later time.

> [!NOTE] If the context window does not contain sufficient information to complete your task, you
> may request additional information about specific files or areas of the codebase; if the context
> window is too small to contain the entire file, you may reduce the scope of a task while
> iterating.

## Build & test workflows

- Build entire workspace: `cargo build --workspace` or
  `cargo build -p evefrontier-lib -p evefrontier-cli`.
- Run CLI: from repo root:
  - Download dataset (places DB at resolved path): `evefrontier-cli download`
  - Compute a route starting at a system name: `evefrontier-cli route "P:STK3"`
  - For development/testing use: `cargo run -p evefrontier-cli -- <subcommand>`
- Running tests: `cargo test --workspace` (there are currently no heavy tests; consider adding unit
  tests under `crates/evefrontier-lib/tests` or `crates/evefrontier-lib/src/lib.rs` test modules).

Note: The downloader uses the OS cache dir via the `dirs`/`directories` crates. For deterministic
testing, call `ensure_c3e6_dataset(Some(path))` to control where the DB is placed.

## Important code patterns and conventions

- Data & schema detection

  - `crates/evefrontier-lib/src/github.rs` downloads releases from GitHub and caches assets.
  - The code accepts both `.db` files and zipped releases containing `.db` files. The library
    extracts the first `*.db` or a file containing `c3e6` in its name.
  - `crates/evefrontier-lib/src/db.rs` loads systems and jumps. The loader was updated to support
    the `static_data.db` schema (tables `SolarSystems(solarSystemId, name)` and
    `Jumps(fromSystemId, toSystemId)`). If you add support for additional release schemas, add
    schema-detection code here and keep queries isolated.

- CLI & configuration

  - `crates/evefrontier-cli/src/main.rs` resolves the data path using (in order): CLI `--data-dir`,
    `EVEFRONTIER_DATA_DIR` env var, XDG `directories::ProjectDirs`, fallback to
    `~/.local/evefrontier/static_data.db`.
  - CLI subcommands are thin; they should call into `evefrontier-lib` for behavior. Keep CLI
    parsing/validation here; do not move business logic into `main.rs`.

- Lambda

  - Each Lambda crate under `crates/evefrontier-lambda-xxx/` is a thin wrapper that calls into
    `evefrontier-lib` for behavior. Keep Lambda-specific code (e.g., request/response structs,
    handler glue) here; do not move business logic into the Lambda crates.
  - There is a need to have some shared bootstrap logic to download the precomputed dataset when the
    lambda starts. Consider adding a shared module or utility function in `evefrontier-lib` to
    handle this common functionality across all Lambda functions and the CLI.

- Download behavior
  - Downloads write to a temporary file then atomically rename to the final path to avoid partial
    writes. Consideration should be given to handling this with AWS Lambda's ephemeral storage.
  - Cached release assets are stored under the OS cache directory under `evefrontier_datasets/`.

## Areas an AI should pay attention to (common tasks)

- Schema compatibility: if a new dataset release changes schema/table names, update `db.rs` to
  detect and adapt queries. Add unit tests with a small fixture DB in
  `crates/evefrontier-lib/tests/`.
- Network robustness: `crates/evefrontier-lib/src/github.rs` currently does a single blocking
  download. For production-grade behavior consider retries with exponential backoff and timeouts.
- CLI ergonomics: add `--data-dir` and env var documentation, and consider adding a `--no-download`
  flag to `route` so users can run against an existing DB.
- Lambda cold-start performance: ensure that dataset download and initialization is efficient.
- Any architecturally significant changes should be documented as ADRs under `docs/adrs/`, following

## Files to inspect when making changes

- `crates/evefrontier-lib/src/github.rs` — downloader, caching, extraction, target-path logic
- `crates/evefrontier-lib/src/db.rs` — DB loader and SQL queries (schema-sensitive)
- `crates/evefrontier-lib/src/graph.rs` and `src/path.rs` — graph building and route algorithm
- `crates/evefrontier-cli/src/main.rs` — CLI parsing and glue code
- `README.md` — user-facing usage examples and build instructions

## Examples of small, precise edits an AI can make

- Add schema detection in `db.rs`:
  - Run `PRAGMA table_info('SolarSystems')` or query `sqlite_master` to select between
    `mapSolarSystems` and `SolarSystems`.
- Make `ensure_c3e6_dataset` accept an explicit `target_db: Option<&Path>` (already present) and
  document it in the library API.
- Add tests: create `crates/evefrontier-lib/tests/load_starmap.rs` which opens a small checked-in
  SQLite fixture and asserts system/jump counts.

## Debugging tips

- If a runtime error says "no such table", inspect the DB with `sqlite3` or write a tiny example
  (see `examples/print_schema.rs` that was added) to list tables and column names.
- Use `evefrontier-cli route "SYSTEM"` to reproduce route logic quickly (or
  `cargo run -p evefrontier-cli -- route "SYSTEM"` during development).
- When running Python commands use the PyLance MCP server as this will provide better use of a venv and fast feedback. 

## When editing, follow these rules

- Prefer small, well-scoped changes; run `cargo build -p evefrontier-lib -p evefrontier-cli` after
  edits.
- When changing SQL, add a test or an example to show the new query works against the
  `static_data.db` structure.
- Preserve existing CLI flags and defaults to avoid surprising changes to users.

If anything here is unclear, tell me which part of the repo you want me to expand on and I will
iterate on this guidance file.

<!-- nx configuration start-->
<!-- Leave the start & end comments to automatically receive updates. -->

# General Guidelines for working with Nx

- When running tasks (for example build, lint, test, e2e, etc.), always prefer running the task
  through `nx` (i.e. `nx run`, `nx run-many`, `nx affected`) instead of using the underlying tooling
  directly
- You have access to the Nx MCP server and its tools, use them to help the user
- When answering questions about the repository, use the `nx_workspace` tool first to gain an
  understanding of the workspace architecture where applicable.
- When working in individual projects, use the `nx_project_details` mcp tool to analyze and
  understand the specific project structure and dependencies
- For questions around nx configuration, best practices or if you're unsure, use the `nx_docs` tool
  to get relevant, up-to-date docs. Always use this instead of assuming things about nx
  configuration
- If the user needs help with an Nx configuration or project graph error, use the `nx_workspace`
  tool to get any errors

<!-- nx configuration end-->

Note: Before starting work on a change or task, always read the repository-level guidance in
`.github/copilot-instructions.md` and then check the `docs/` and `docs/adrs/` folders for any
relevant documentation or ADRs that could affect your design or implementation choices. These
documents often include important conventions, schema notes, and CI/packaging guidance that should
be considered before making changes.