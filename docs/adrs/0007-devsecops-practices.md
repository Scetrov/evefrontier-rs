# ADR 0007: DevSecOps practices — pre-commit, CI/CD, attestations & testing

## Status

Accepted (Recommended practices)

## Context

This repository is small but produces artifacts (binaries, datasets) and will be used by other
developers. Adopting DevSecOps practices early reduces security and reliability risks as the project
grows. The repository already uses a few developer tools (husky, lint-staged, NX) and CI should
codify behavior so contributors and automation have a shared, enforceable standard.

## Decision

Adopt the following DevSecOps practices for the repository and CI pipelines:

1. Pre-commit checks
   - Use `husky` + `lint-staged` to run lightweight formatting and linting on staged files (Prettier
     for markdown and code format checks). Keep hooks fast; heavy checks belong to CI.

2. Continuous Integration (CI)
   - A PR-triggered CI workflow should run: unit tests, basic integration tests, static analysis
     (clippy for Rust), formatting check, and markdown linting.
   - CI should pin tool versions (Rust toolchain and Node) in workflow definitions to ensure
     reproducible runs.

3. Secrets & dependency scanning
   - Run dependency vulnerability scanning (for example, `cargo audit` for Rust crates and an SCA
     tool for Node) as part of CI or nightly jobs.
   - Do not store secrets in the repo. Use repository secrets or CI-provided secret stores.

4. Attestations & signatures
   - Sign release artifacts (binaries, packages, or dataset releases) using a reproducible and
     auditable approach; consider using `cosign` for OCI or binary signatures and/or GPG signatures
     for tarballs.
   - Produce simple attestations for important pipeline steps (build, test, scan) and store them
     with the release metadata.

5. Unit testing and testing pyramid
   - Follow the testing pyramid (unit tests at the base, fewer integration and end-to-end tests
     above). Keep unit tests fast and deterministic. Use small SQLite fixture DBs to validate DB
     schema detection logic.

6. Release process & reproducibility
   - CI should produce reproducible build artifacts when possible. Document how to locally reproduce
     the build and the pinned toolchain versions.

## Rationale

- Pre-commit hooks catch trivial problems early and improve PR quality without blocking CI
  resources.
- CI enforces project-wide standards and is the canonical place for heavier checks that take longer
  to run.
- Attestations and signatures enable downstream consumers to validate origin and integrity of
  artifacts.
- Following the testing pyramid helps keep feedback fast and focused while still assuring behavior
  through higher-level tests.

## Consequences

- Contributors must install or use compatible tool versions (or rely on CI containers) for
  consistent results.
- Some additional CI time and complexity is introduced; however the trade-off is improved
  reliability and security.

## Implementation notes

- Example CI jobs to add:
  - `lint` — runs `pnpm run lint:md` and `cargo fmt -- --check`
  - `test` — runs `cargo test --workspace`
  - `clippy` — runs `cargo clippy --all-targets --all-features -- -D warnings`
  - `audit` — runs `cargo audit` (nightly or PR optional) and any Node SCA
  - `release` — build artifacts, sign with `cosign`/GPG and publish with attached attestations

- For signing keys, prefer using ephemeral keys in CI when possible or a dedicated signing service.
  Document the signing procedure in `docs/`.

  - Dependency checks / outdated reporting
    - Run a pnpm outdated check via the root Nx project target:
      ```pwsh
      pnpm exec nx run evefrontier-pathfinder:outdated
      ```
    - For a prettier report that fails with a readable JSON payload, use:
      ```pwsh
      pnpm run outdated:report
      ```
    - These commands are used by the repository pre-commit flow. Prefer running dependency checks via
      the Nx targets so they behave the same in CI and locally.

**Reference**: See [ADR 0017: NX Repository Orchestration Strategy](0017-nx-orchestration-strategy.md)
for detailed documentation of task orchestration patterns, cache strategy, and how CI/CD tasks
inherit from workspace defaults.
