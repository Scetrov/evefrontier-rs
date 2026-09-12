## 1. Dependency remediation

- [x] 1.1 Add an exact patched `smol-toml` pnpm override and regenerate the committed lockfile with pinned pnpm.
- [x] 1.2 Inspect the resolved dependency graph to confirm all `smol-toml` instances are patched.

## 2. Quality workflow reliability

- [x] 2.1 Update the scheduled Rust outdated-report workflow to reuse a cached `cargo-outdated` executable and install only when unavailable.

## 3. Validation and delivery

- [x] 3.1 Run frozen pnpm installation, Node high-severity audit, and applicable Nx quality gates.
- [x] 3.2 Run repository pre-commit checks and prepare the PR security-impact, threat-model, compliance, and backout evidence.
- [x] 3.3 Prepare the verified remediation for GPG-signed feature-branch delivery and pull-request creation.