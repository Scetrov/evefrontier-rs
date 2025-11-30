# Contributing to EveFrontier

Thanks for your interest in contributing! This document explains the preferred workflow for filing
issues, submitting PRs, and running tests locally.

## Code of Conduct

This project follows the Contributor Covenant Code of Conduct. Please read `CODE_OF_CONDUCT.md` and
behave respectfully.

## How to file issues

- Search existing issues before opening a new one.
- For bug reports, use the Bug Report template and include: steps to reproduce, expected vs actual
  behavior, and environment details.
- For feature requests, use the Feature Request template and explain the motivation and high-level
  approach.

## Branching & PRs

- Branch from `main` using a short, descriptive name: `feature/<short>` or `fix/<short>`.
- Open a PR against `main` with the provided Pull Request template.
- Include unit tests for new behavior where practical.

## Changelog requirement

Before merging a PR that modifies code, docs, or other user-visible behavior, add a short entry to
`CHANGELOG.md` under the `Unreleased` section. The entry should contain a one-line summary, the date
(YYYY-MM-DD), the author (name or `auto-llm:<id>`), and a tag `[manual]` or `[auto-llm]`. LLM/agent
edits must append the changelog entry when they apply changes, check the system time to identify the
actual date do not use dates in the past. Reviewers should verify the changelog entry for clarity.

## Tooling requirements

- Node.js: use the version pinned in `.nvmrc` (currently 24 LTS as of November 2025). Install via `nvm use`.
- Package manager: pnpm 10+. The repository declares `"packageManager": "pnpm@10.0.0"` and
  enforces `engines.pnpm >= 10.0.0`.

Quick setup:

```sh
# Ensure Node 24 per .nvmrc
nvm use

# Install pnpm v10 globally (recommended)
npm install -g pnpm@10

# Install workspace tools and generate lockfile
pnpm install
```

Common developer commands (recommended via package.json scripts for consistency):

```sh
pnpm run build
pnpm run test
pnpm run clippy
pnpm run lint:md
```

Or, run Nx directly with the required exclusion flag to avoid recursive invocation of the root project:

```sh
pnpm nx run-many -t build --exclude evefrontier-rs
pnpm nx run-many -t test --exclude evefrontier-rs
pnpm nx run-many -t clippy --exclude evefrontier-rs
```

## Nx Task Orchestration

The repository uses Nx to orchestrate Rust build, test, lint, and clippy tasks across all 6 crates in the workspace:

- evefrontier-lib
- evefrontier-cli
- evefrontier-lambda-shared
- evefrontier-lambda-route
- evefrontier-lambda-scout-gates
- evefrontier-lambda-scout-range

> **Note:** When referring to "all 6 crates", this means all workspace members listed above. "All Lambda crates" refers to the 4 crates under `crates/evefrontier-lambda-*` (including the shared crate), while "Lambda functions" refers specifically to the 3 function crates (`route`, `scout-gates`, `scout-range`). Be explicit in PRs and documentation to avoid confusion.

Nx provides:
- **Task dependencies**: Tests automatically run after builds complete (`test` depends on `build`)
- **Intelligent caching**: Nx caches task outputs locally to skip redundant work
- **Affected detection**: Run tasks only for projects impacted by your changes

### Task Execution Model

Nx tasks are configured with `parallel: false` for all Rust targets. This means:
- Nx orchestrates task order and dependencies (e.g., ensuring builds complete before tests)
- Cargo manages its own internal parallelism for compilation (`-j` flag)
- No conflicts between Nx and Cargo parallelism

### Common Nx Commands

Run a single project's tests (with automatic build):
```sh
pnpm nx run evefrontier-lib:test
```

Run tests for all projects:
```sh
pnpm nx run-many --target=test --all
```

Run tests only for projects affected by your changes:
```sh
pnpm nx affected --target=test
```

Build specific projects with explicit dependency order:
```sh
pnpm nx run-many --target=build --projects=evefrontier-lib,evefrontier-cli
```

Run clippy on all crates:
```sh
pnpm nx run-many --target=clippy --all
```

### Cache Behavior

- Nx caches build and test outputs in `.nx/cache/` (local only, not committed)
- Cache keys include input files (Cargo.toml, Cargo.lock, src/**), toolchain version, and command
- To clear cache: `pnpm nx reset`
- To bypass cache for a single run: `pnpm nx run <project>:<target> --skip-nx-cache`

### Troubleshooting

If Nx daemon causes issues (rare):
```sh
NX_DAEMON=false pnpm nx run-many --target=test --all
```

To see detailed task execution:
```sh
pnpm nx run <project>:<target> --verbose
```

## Local development

1. Install Rust (see `.rust-toolchain`) and Node (see `.nvmrc` if using Node tools).
2. Build the workspace:

```pwsh
cargo build --workspace
```

3. Run tests:

```pwsh
cargo test --workspace
```

4. Use the included `minimal_static_data.db` fixture for deterministic tests. See `docs/USAGE.md`
   for an example of wiring `EVEFRONTIER_DATA_DIR`.

## Formatting & linting

- Run `cargo fmt` and `cargo clippy` before opening a PR.
- Run `pnpm run lint:md` to lint markdown files.

## Security

If you discover a security vulnerability, do NOT open a public issue. Instead, report it privately
to `security@EXAMPLE.COM` (replace with the maintainers' contact). See `SECURITY.md` for details.

## Contact

# Pre-start checklist for contributors and agents

Before starting work on a change, read the repository guidance and check for relevant documentation:

- Read `.github/copilot-instructions.md` for workspace-wide conventions and developer guidance.
- Review `docs/` and `docs/adrs/` for ADRs and design decisions that could affect your change.
- If your change touches release packaging, CI, or security, consult
  `docs/adrs/0007-devsecops-practices.md`.

Following this checklist helps avoid duplicated effort and ensures changes align with established
decisions.

# Contributing

Thank you for contributing! This document explains how to set up a local development environment,
run the standard checks, and contribute changes to the repository.

Environment

1. Install Rust (recommended: rustup)

- Install `rustup` from the official site: <https://rustup.rs>

- The repository uses Rust 1.91.1 (pinned in `.rust-toolchain`). When you enter the repository
  directory, rustup will automatically use this version. To verify:

  rustc --version # should show 1.91.1

2. Install Node.js and pnpm (for developer tooling)

- Use a tool like `volta` or `nvm` to manage Node versions.
- The repository pins Node 24 (LTS) in `.nvmrc`. If using nvm:

  nvm use # automatically uses version from .nvmrc

> Note: If your environment policy mandates current LTS prior to October 2025,
> use Node 22 LTS and update `.nvmrc` accordingly. The workspace scripts and Nx
> tooling are compatible with Node 22+, but the repository standard is Node 24
> LTS to align with modern features and CI configuration.

- Install `pnpm` (recommended using corepack for version consistency):

  corepack enable
  corepack prepare pnpm@10.0.0 --activate

  Alternatively, install globally with version pin:

  npm install -g pnpm@10

3. Optional developer tools

- Install `cargo-audit` for dependency scanning:

  cargo install cargo-audit

4. VS Code MCP Configuration (Optional)

The repository includes `.vscode/mcp.json` which configures the GitHub Copilot Model Context
Protocol (MCP) server. This enables enhanced GitHub integration features within VS Code when using
GitHub Copilot.

**This configuration is optional.** The MCP setup is not required to build, test, or contribute to
the project. It provides additional AI-assisted features for developers using GitHub Copilot in VS
Code, such as:

- Enhanced repository context awareness
- GitHub API integration for PR/issue management
- Improved code suggestions based on repository patterns

If you're not using GitHub Copilot or prefer not to use MCP features, you can safely ignore this
configuration. The repository will work normally without it.

Local checks

- Build the Rust workspace:

  cargo build --workspace

- Run tests:

  cargo test --workspace

- Run Rust lints:

  cargo clippy --all-targets --all-features -- -D warnings

- Format and lint markdown:

  pnpm install pnpm run lint:md

- Format code and docs:

  pnpm run format

Pre-commit and CI

- The project uses `husky` and `lint-staged` for local pre-commit hooks. These run quick formatters
  on staged files. Keep hooks fast — heavy checks run in CI.
- CI should run the full matrix: `cargo test`, `clippy`, `cargo audit` (or nightly),
  `pnpm run lint:md`, and any additional integration tests.

Dependency Management

- **Nightly Dependency Checks**: A GitHub Actions workflow runs nightly to check for outdated Rust
  and Node dependencies. The workflow publishes artifacts containing reports you can review:
  - `rust-outdated-report`: JSON and text reports from `cargo outdated`
  - `node-outdated-report`: JSON and text reports from `pnpm outdated`
  - Find these under the "Dependency Check" workflow runs in the Actions tab
  - Reports are kept for 30 days
- **Manual Dependency Updates**: To update dependencies locally:
  - Rust: `cargo update` (patch versions) or edit `Cargo.toml` for minor/major updates
  - Node: `pnpm update` or edit `package.json`
  - Always run tests after updating dependencies
  - Update `CHANGELOG.md` when making dependency changes

Signing and releases

- For release artifacts, prefer signing binaries or release archives using a dedicated signing
  process (GPG or `cosign`), and publish attestations for the important pipeline steps
  (build/test/scan).

Notes

- If you need deterministic runs for tests that use the dataset downloader, call
  `ensure_dataset(Some(path), DatasetRelease::latest())` (or choose a specific
  `DatasetRelease::tag(...)`) to provide a fixed dataset path.
- If you don't want to install Node locally, you can still build and test the Rust crates; however
  markdown linting and other Node-based tooling won't run until Node/pnpm is available.
- ensure that your commit message is less than 72 chars in the first line, add more details in the
  summary from line 3 onwards.
- use conventional commit standards to create clear commits

Thanks for helping improve the project. If you add new developer-facing tools, please update
`CONTRIBUTING.md` and `docs/adrs/0006-software-components.md`.
