## Why

OSV-Scanner reports RUSTSEC-2026-0285 because the standalone fuzz workspace lockfile resolves
`rustls` 0.23.42. The advisory affects releases from 0.23.13 through 0.23.44, so every committed
lockfile must resolve `rustls` 0.23.45 or newer.

## What Changes

- Update the fuzz workspace's locked `rustls` release from 0.23.42 to patched release 0.23.45.
- Confirm the root and fuzz lockfiles contain no release affected by RUSTSEC-2026-0285.
- Preserve the existing dependency graph, application code, and public APIs.

## Capabilities

### New Capabilities

- `rustls-dependency-security`: Ensures committed Cargo dependency resolutions exclude `rustls`
  releases affected by RUSTSEC-2026-0285.

### Modified Capabilities

- None.

## Impact

- `fuzz/Cargo.lock` dependency resolution and repository vulnerability-scan results.
- No intended source-code, public-API, runtime, or deployment-configuration changes.
