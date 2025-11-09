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
edits must append the changelog entry when they apply changes. Reviewers should verify the changelog
entry for clarity.

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

- Pin a stable toolchain for the repository (example):

  rustup toolchain install stable rustup override set stable

2. Install Node.js and pnpm (for developer tooling)

- Use a tool like `volta` or `nvm` to manage Node versions.
- Install `pnpm` (recommended):

  npm install -g pnpm

3. Optional developer tools

- Install `cargo-audit` for dependency scanning:

  cargo install cargo-audit

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
- ensure that your commit message is less than 72 chars in the first line, add more details in the summary from line 3 onwards.
- use conventional commit standards to create clear commits

Thanks for helping improve the project. If you add new developer-facing tools, please update
`CONTRIBUTING.md` and `docs/adrs/0006-software-components.md`.
