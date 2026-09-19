## Context

The production workspace already resolves `rustls` 0.23.45 in `Cargo.lock`, but the independently
locked fuzz workspace still records 0.23.42. OSV-Scanner examines both committed lockfiles and
correctly reports RUSTSEC-2026-0285 for the fuzz resolution. The patched threshold is 0.23.45.

## Goals / Non-Goals

**Goals:**

- Remove every `rustls` release affected by RUSTSEC-2026-0285 from committed lockfiles.
- Keep the dependency-resolution diff minimal and reviewable.
- Retain locked, checksum-verified dependency resolution.

**Non-Goals:**

- Upgrade unrelated dependencies.
- Change TLS configuration, application behavior, or public APIs.
- Suppress or ignore the advisory.

## Decisions

### Apply a targeted patch-level lockfile update

Update only the fuzz lockfile's `rustls` package entry to 0.23.45. The patched package has the same
recorded dependency list, so no broader graph change is needed.

A broad fuzz-workspace refresh was rejected because it would expand regression risk and obscure the
security remediation. An advisory ignore was rejected because it would retain the vulnerable
release.

### Validate every committed Cargo lockfile

Check both `Cargo.lock` and `fuzz/Cargo.lock` against the advisory's affected range. Validate the
fuzz manifest with locked, offline Cargo metadata and run repository-standard checks where the
sandbox's cached dependencies permit them.

## Risks / Trade-offs

- [The checksum is copied incorrectly] → Use the checksum already resolved for `rustls` 0.23.45 in
  the root lockfile and validate locked Cargo metadata.
- [The fuzz target has an unforeseen compatibility issue] → Rely on the patch release's unchanged
  dependency shape and CI/reviewer validation; revert and investigate if fuzz compilation fails.
- [Reverting the lockfile change reintroduces the finding] → Call this out explicitly in the pull
  request backout plan.

## Migration Plan

1. Update `fuzz/Cargo.lock` to `rustls` 0.23.45 with its registry checksum.
2. Validate the lockfile and relevant repository checks.
3. Submit a signed pull request with security impact, evidence, and rollback implications.

Rollback is a revert of the lockfile update. Because rollback restores the vulnerable release, it
requires renewed security review rather than routine operational rollback.

## Open Questions

None.
