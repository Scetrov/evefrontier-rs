## Why

The committed `Cargo.lock` contains vulnerable `h2` 0.4.15, causing the repository's required
RustSec audit to fail with RUSTSEC-2026-0258. Although the current resolver does not use that stale
entry, it must be removed or upgraded so the lockfile remains audit-clean and cannot reintroduce the
advisory.

## What Changes

- Re-resolve the Rust lockfile so it no longer contains a version of `h2` affected by
  RUSTSEC-2026-0258 (GHSA-q83h-524g-xf6h).
- Verify the uncached Rust dependency audit passes without suppressing the advisory.
- Verify standard Nx-managed Rust validation continues to pass without application-code or
  public-API changes.

## Capabilities

### New Capabilities

- `h2-dependency-security`: Ensures the committed Cargo dependency resolution excludes vulnerable
  `h2` releases covered by RUSTSEC-2026-0258 and remains audit-clean.

### Modified Capabilities

- None.

## Impact

- `Cargo.lock` dependency resolution and security-audit evidence.
- CI and pre-commit audit gates that execute `cargo audit`.
- No intended Rust source-code, public API, runtime behavior, or deployment-configuration changes.
