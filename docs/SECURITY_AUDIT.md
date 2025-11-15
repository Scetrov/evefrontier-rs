# Security Audit Guide

This document describes how to run security audits and remediate vulnerabilities in the evefrontier-rs project.

## Overview

We use `cargo-audit` to scan our dependencies for known security vulnerabilities from the [RustSec Advisory Database](https://rustsec.org/). The audit runs automatically in:

- **CI Pipeline**: Every push and pull request
- **Pre-commit Hook**: Before every commit (via rusty-hook)
- **Manual Runs**: Via `make audit` or `cargo audit`

## Running Security Audits

### Via Make (Recommended)

```bash
make audit
```

This runs `cargo audit --deny warnings`, which will:
- Fetch the latest advisory database from RustSec
- Scan all dependencies in `Cargo.lock`
- **Fail** if any vulnerabilities are found

### Via Cargo Directly

```bash
# Basic audit
cargo audit

# Fail on warnings (CI/pre-commit mode)
cargo audit --deny warnings

# Update advisory database first
cargo audit fetch
cargo audit
```

### In CI

The GitHub Actions workflow includes a dedicated `security-audit` job that:
1. Checks out the code
2. Sets up Rust toolchain
3. Installs `cargo-audit`
4. Runs `cargo audit --deny warnings`

If vulnerabilities are found, the CI build will **fail**, blocking merges.

### In Pre-commit Hook

The rusty-hook pre-commit runs 5 checks (step 5 is the audit):
1. Format check (`cargo fmt`)
2. Clippy lints (`cargo clippy`)
3. Build (`cargo build`)
4. Tests (`cargo test`)
5. **Security audit** (`cargo audit --deny warnings`)

If vulnerabilities are found, the commit will be **blocked** until resolved.

## Understanding Audit Results

When `cargo-audit` finds vulnerabilities, it outputs:

```
Crate:     example-crate
Version:   1.2.3
Warning:   vulnerability
Title:     Example Vulnerability Description
Date:      2024-01-15
ID:        RUSTSEC-2024-0001
URL:       https://rustsec.org/advisories/RUSTSEC-2024-0001
Dependency tree:
example-crate 1.2.3
└── your-crate 0.1.0
```

**Key fields:**
- **Crate**: The vulnerable dependency
- **Version**: The version you're using
- **ID**: RustSec advisory ID (use this to search for details)
- **URL**: Link to full advisory with remediation guidance
- **Dependency tree**: Shows how the vulnerable crate is included

## Remediation Workflow

### 1. Assess Severity

Visit the advisory URL and check:
- **CVSS score**: How severe is the vulnerability?
- **Attack vector**: Is it exploitable in our use case?
- **Patched version**: Is a fix available?

### 2. Update Dependencies

If a patched version exists, update the dependency:

```bash
# Update specific crate to latest compatible version
cargo update -p vulnerable-crate

# Or update all dependencies
cargo update

# Verify the vulnerability is resolved
make audit
```

### 3. If No Patch Available

If no fix exists yet, you have several options:

#### Option A: Wait for Upstream Fix
- Monitor the advisory and upstream repository
- Add a TODO or tracking issue
- Consider temporary workarounds

#### Option B: Ignore Warning (Use Sparingly)
Create `audit.toml` in the workspace root to suppress specific advisories:

```toml
[advisories]
ignore = [
    "RUSTSEC-2024-0001",  # Brief justification
]
```

**Only use this for:**
- False positives
- Vulnerabilities that don't apply to our usage
- Temporarily while waiting for upstream fix

**Always include:**
- The advisory ID
- A clear comment explaining why it's safe to ignore
- A tracking issue or TODO if temporary

#### Option C: Replace Dependency
If the vulnerable crate is unmaintained or the fix is delayed:
1. Search for alternative crates with similar functionality
2. Evaluate security posture and maintenance status
3. Update code to use the replacement
4. Test thoroughly

### 4. Verify Fix

After remediation:

```bash
# Run audit
make audit

# Run full test suite
make test

# Test pre-commit hook
git add .
git commit -m "fix: update vulnerable dependency"
```

## Advisory Database Updates

The RustSec advisory database is updated frequently. To get the latest advisories:

```bash
cargo audit fetch
```

This is automatically done in CI and pre-commit hooks.

## False Positives

Sometimes `cargo-audit` reports advisories that don't affect our usage. Examples:
- **Denial of service** vulnerabilities in a library we only use at build time
- **Unsoundness issues** in unsafe code we don't trigger
- **Platform-specific** vulnerabilities on platforms we don't support

In these cases:
1. Document why the advisory doesn't apply (in a commit message or comment)
2. Add the advisory to `audit.toml` ignore list
3. Include a TODO to revisit when a proper fix is available
4. Link to a tracking issue if long-term

## Integration with Other Tools

### cargo-deny

For more advanced dependency policy enforcement, consider [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny):
- License policy enforcement
- Dependency ban lists
- More granular advisory controls

### Dependabot

GitHub Dependabot can automatically:
- Detect vulnerable dependencies
- Open PRs with version updates
- Keep dependencies current

Enable it in `.github/dependabot.yml` for automated dependency updates.

## References

- [RustSec Advisory Database](https://rustsec.org/)
- [cargo-audit Documentation](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [OWASP Dependency Check](https://owasp.org/www-project-dependency-check/)
- [ADR 0007: DevSecOps Practices](adrs/0007-devsecops-practices.md)

## Questions or Issues

If you encounter issues with the security audit process:
1. Check this document for remediation steps
2. Review the RustSec advisory for detailed guidance
3. Consult the team security contact (see `SECURITY.md`)
4. Open an issue in the repository for discussion
