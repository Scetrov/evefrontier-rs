## Context

GitHub evidence shows one root cause: `smol-toml` <= 1.7.0 is resolved transitively through both Nx and markdownlint-cli2. This causes two active Dependabot PRs (#234, #236), the push on #235, and the current main branch security-audit job to fail. Dependabot's direct remediation failed because it could not select a safe transitive resolution from the lockfile. A separate scheduled dependency-report run failed after `cargo install cargo-outdated --locked || true` encountered an existing binary and the subsequent artifact upload hit a transient network reset.

## Goals / Non-Goals

**Goals:**
- Force every pnpm resolution of `smol-toml` to a patched release (at least 1.7.1).
- Preserve deterministic pnpm installation and make the normal Node security audit pass.
- Avoid an unnecessary cargo-outdated reinstall when the executable is already cached.
- Deliver evidence needed to close GitHub alert #56 and the duplicate Scorecard alert #52.

**Non-Goals:**
- Change application code or implement enhancement #50.
- Suppress, dismiss, or ignore a security alert.
- Upgrade unrelated direct dependencies or modify the separate Dependabot PRs.

## Decisions

1. Add an exact `pnpm.overrides.smol-toml` version and regenerate the lockfile. Version 1.8.0 is the latest release reported by Dependabot and is newer than the advisory's 1.7.1 minimum. An override is preferable to waiting for upstream dependency releases because both vulnerable paths are transitive and the lockfile must be safe now.
2. Use the repository's pinned pnpm version to regenerate and inspect the lockfile. Validate with `pnpm audit --audit-level high` plus the Nx workspace checks so CI behavior is reproduced locally.
3. Change the workflow to use the cached `cargo-outdated` binary when present; install only when absent. Artifact upload remains required and GitHub transport failures are not masked.

## Risks / Trade-offs

- [An override could conflict with an upstream package's API expectations] → `smol-toml` is a transitive tooling dependency; run frozen install, audit, and Nx checks before submission.
- [A future advisory could invalidate 1.8.0] → keep Dependabot and the uncached CI audit enabled; no alert suppression is introduced.
- [GitHub artifact transport can still fail transiently] → remove the local install collision; retry a workflow only if GitHub reports a transport failure.

## Migration Plan

1. Apply the override and regenerate the lockfile.
2. Run the security audit and workspace quality gates.
3. Submit a feature-branch PR with alert and check evidence.
4. After merge, rerun/observe CI and confirm Dependabot #56 and Scorecard #52 close.

Rollback: revert this PR. This reintroduces the vulnerable transitive dependency, so rollback requires documented security approval and immediate follow-up remediation.

## Open Questions

- None; the advisory specifies a patched minimum and Dependabot confirms 1.8.0 is the latest available release.