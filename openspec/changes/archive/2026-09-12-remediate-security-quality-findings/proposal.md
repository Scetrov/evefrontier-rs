## Why

Two open Dependabot PRs and the current `main` security-audit job fail because the committed pnpm graph resolves vulnerable `smol-toml` versions. GitHub Dependabot alert #56 and Scorecard code-scanning alert #52 identify the same high-severity denial-of-service advisory, GHSA-7w5x-hrqm-74c2 / CVE-2026-85730.

## What Changes

- Pin transitive `smol-toml` resolution to a currently released, patched version through the root pnpm override and regenerate `pnpm-lock.yaml`.
- Verify the committed dependency graph no longer contains vulnerable `smol-toml` versions and that the Node audit passes.
- Repair the scheduled Rust dependency-report workflow so a pre-existing `cargo-outdated` binary does not cause an unnecessary install attempt or artifact-report failure.
- Document the GitHub evidence, scope boundaries, validation, security impact, and rollback plan in the PR.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `npm-dependency-security`: Require patched `smol-toml` resolution and security-audit verification for the affected pnpm graph.

## Impact

- `package.json` and `pnpm-lock.yaml` dependency metadata.
- `.github/workflows/dependency-check.yml` scheduled reporting reliability.
- CI security-audit result, Dependabot alert #56, Scorecard alert #52, and failing PRs #234 and #236.
- No Rust or runtime application API changes. Open enhancement #50 is unrelated and explicitly out of scope.