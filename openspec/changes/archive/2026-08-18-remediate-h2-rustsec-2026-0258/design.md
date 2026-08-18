## Context

`cargo audit` scans every package recorded in the committed `Cargo.lock`. It currently reports
RUSTSEC-2026-0258 for `h2` 0.4.15, whose patched threshold is 0.4.16. The lockfile records `hyper`
1.10.1 as a consumer of that package, but the current Cargo dependency tree contains neither
package; this indicates stale lockfile resolution rather than an active workspace dependency path.

The repository treats uncached dependency auditing as a required security control. The change must
preserve locked resolution and use Nx-managed validation where targets are available.

## Goals / Non-Goals

**Goals:**

- Make the committed lockfile audit-clean for RUSTSEC-2026-0258 without suppressing the advisory.
- Retain a minimal, reviewable dependency-resolution diff.
- Confirm no application source or public API change is required.

**Non-Goals:**

- Upgrade unrelated dependencies merely because newer versions exist.
- Change HTTP/2 request handling, networking behavior, Lambda behavior, or application APIs.
- Add an `audit.toml` exception or weaken audit/CI/pre-commit gates.

## Decisions

### Re-resolve stale lockfile state rather than patch application code

The implementation will use Cargo's resolver to produce a valid lockfile that excludes vulnerable
`h2` releases or selects `h2` 0.4.16 or newer when a live constraint requires it. This directly
addresses the audit input and preserves Cargo's checksums and dependency integrity.

**Alternatives considered:**

- Add an advisory ignore: rejected because it leaves the vulnerable lockfile entry and violates the
  no-suppression goal.
- Modify HTTP/2 body-draining logic: rejected because no active resolver path currently includes
  `h2`, and the issue is dependency metadata rather than observed application behavior.
- Perform a broad workspace dependency refresh: rejected because it obscures the security
  remediation and expands regression risk.

### Validate the resolved graph as well as the audit result

The implementation will inspect the updated dependency graph and run `cargo audit`; it will then run
the repository-standard relevant Nx Rust validation targets. This distinguishes a lockfile cleanup
from a latent runtime dependency upgrade and provides evidence the change did not alter application
behavior.

**Alternatives considered:**

- Audit only: rejected because a passing audit alone does not establish that the workspace still
  builds and tests with locked resolution.
- Run raw Cargo commands exclusively: rejected for normal validation because repository policy uses
  Nx orchestration; Cargo remains appropriate for the audit and resolver inspection that Nx
  delegates to.

## Risks / Trade-offs

- [A normal resolver refresh changes unrelated lockfile entries] → Inspect the lockfile diff and
  constrain the update to the smallest resolver action possible; stop and reassess if unrelated
  production dependencies move.
- [The stale package becomes live on another supported target or feature set] → Inspect
  target/feature-aware dependency resolution and ensure any retained `h2` version is at least
  0.4.16.
- [RustSec data changes during validation] → Record the audit timestamp and advisory output; do not
  cache or suppress the audit.
- [A patch-level transitive update exposes compatibility issues] → Run locked Nx build, test, lint,
  and clippy targets before submitting the remediation.

## Migration Plan

1. Create a focused branch and update only the dependency-resolution state required to remove or
   patch vulnerable `h2`.
2. Review the lockfile diff, run the security audit and relevant Nx validation targets, and submit
   the evidence in a signed PR.
3. If validation fails, restore the prior lockfile and investigate the resolver/constraint conflict;
   do not merge an audit suppression as a substitute.

Rollback is a `Cargo.lock` revert. Reverting is expected to reintroduce RUSTSEC-2026-0258, so it
requires a new security assessment and should only occur if the remediation proves incompatible.

## Open Questions

- Does Cargo's minimal resolver action remove the stale `hyper`/`h2` entries entirely, or upgrade
  the retained `h2` entry to 0.4.16? The implementation must capture the actual diff and validate
  either compliant outcome.
