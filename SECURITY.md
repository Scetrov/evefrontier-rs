# Security

If you discover a security vulnerability, please report it privately to the maintainers via the
[GitHub Security Reporting](https://github.com/Scetrov/evefrontier-rs/security/advisories) feature.
Do NOT open a public issue.

We will acknowledge security reports within 48 hours and coordinate responsible disclosure.

See `CONTRIBUTING.md` for further guidance on reporting and contact details.

## Known Vulnerabilities and Risk Acceptances

The following security advisories have been reviewed and temporarily accepted due to upstream
dependency constraints. These are tracked in `.cargo/audit.toml` and will be resolved when fixes
become available.

| Advisory | Crate | Severity | Status | Notes |
|----------|-------|----------|--------|-------|
| [RUSTSEC-2026-0003](https://github.com/RustCrypto/utils/security/advisories/GHSA-2gqc-6j2q-83qp) | `cmov` 0.3.1 | High (8.9) | Accepted | Non-constant-time code generation on ARM32 targets. Transitive dependency via `kiddo` 5.2.3. Waiting for `kiddo` to update to `cmov >= 0.4.4`. **Impact**: This project does not target ARM32; risk is mitigated for x86_64/aarch64 deployments. |

### Review Schedule

Accepted vulnerabilities are reviewed monthly. When upstream fixes become available:

1. Update the dependency in `Cargo.toml`
2. Remove the advisory from `.cargo/audit.toml`
3. Update this table to remove the entry
4. Document the resolution in `CHANGELOG.md`
