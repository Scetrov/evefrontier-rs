# ADR 0006: Software components used to build the solution

## Status

Accepted

## Context

This project is a Rust workspace with a small set of auxiliary developer tools and scripts.
Developers and CI must know which tools are expected to build, test and validate the project, so we
can ensure reproducible development and make onboarding easier.

## Decision

Document the primary software components and their roles. The documented set will be used by
contributors and CI to verify environment compatibility and to decide which versions to pin or
vendor when necessary.

## Components

- Rust toolchain (rustc, cargo)
  - Role: Build, test and publish Rust crates. The workspace uses Cargo to build
    `crates/evefrontier-lib` and `crates/evefrontier-cli`.
  - Notes: Use rustup to manage toolchains. CI should pin a toolchain (for example a stable channel
    version) for reproducible builds.

- Cargo workspace
  - Role: Manages multiple crates in a single repository, enabling `cargo build --workspace` and
    per-crate builds such as `cargo build -p evefrontier-lib`.

- Node.js & pnpm
  - Role: Manage developer tooling (markdownlint, prettier, husky, lint-staged, nx). `pnpm` is used
    for developer scripts defined in `package.json`.
  - Notes: Node is only required for developer tooling, not for runtime. Use a consistent Node
    version (via `nvm`, `volta` or CI image) when running linters or scripts.

- markdownlint-cli / prettier
  - Role: Lint and format markdown documentation. The repository includes a `.markdownlint.json`
    config and a `lint:md` script to run automatic fixes.

- NX (nx)
  - Role: Repository/task orchestration for developer workflows and scripts. The workspace uses `nx`
    for higher-level scripting and task composition.
  - Reference: [ADR 0017: NX Repository Orchestration Strategy](0017-nx-orchestration-strategy.md)
    documents the formal task orchestration patterns, cache strategy, and CI/CD integration.

- Husky & lint-staged
  - Role: Optional commit-time checks (formatting) via Git hooks. Keep pre-commit hooks light
    (formatters) and move longer-running checks to CI.

- Other developer utilities
  - cross-env: cross-platform environment variable helper used by scripts.

## Rationale

- Explicitly documenting these components reduces onboarding friction and clarifies which tools are
  required only for development vs required for runtime.
- Pinning or documenting versions (in CI images or via tool-version files) is recommended to avoid
  environment-related build failures.

## Consequences

- CI should declare a Rust toolchain version and a Node version to ensure consistent tooling across
  contributors.
- Developers without Node installed can still build the Rust code, but linting and some automation
  will not be available until Node/pnpm are installed.

## Follow-ups

- Add minimal CONTRIBUTING.md describing how to install toolchains (rustup, Node/pnpm) and how to
  run the linters and build commands locally.
- Consider pinning Node and Rust versions (via `.nvmrc` / `volta` / pinned CI container images) for
  stricter reproducibility.
