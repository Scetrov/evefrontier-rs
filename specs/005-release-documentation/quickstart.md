# Release Quickstart Guide

**Feature**: 005-release-documentation  
**Phase**: 1 - Design  
**Date**: 2025-12-28

---

## Quick Release Checklist

Use this abbreviated checklist for releases. For detailed procedures, see `docs/RELEASE.md`.

### Pre-Release

- [ ] All tests pass: `cargo test --workspace`
- [ ] CHANGELOG.md updated with release date
- [ ] Version bumped in `Cargo.toml` files
- [ ] No uncommitted changes: `git status`

### Build & Sign

```bash
# 1. Create signed tag
git tag -s v0.1.0 -m "Release v0.1.0"

# 2. Build release binaries
cargo build --release -p evefrontier-cli

# 3. Generate spatial index (optional)
./target/release/evefrontier-cli index-build

# 4. Create checksums
sha256sum target/release/evefrontier-cli > SHA256SUMS

# 5. Sign checksums
gpg --armor --detach-sign SHA256SUMS

# 6. Generate SBOM
cargo sbom --output-format cyclonedx > evefrontier-v0.1.0.sbom.json

# 7. Sign with cosign (optional)
cosign sign-blob --key cosign.key target/release/evefrontier-cli > evefrontier-cli.sig
```

### Publish

- [ ] Push signed tag: `git push origin v0.1.0`
- [ ] Create GitHub release with artifacts
- [ ] Verify signatures work: `gpg --verify SHA256SUMS.asc`

### Post-Release

- [ ] Update docs/TODO.md if applicable
- [ ] Announce release (if applicable)
- [ ] Bump version to next dev version

---

## Verification Commands

```bash
# Verify tag signature
git tag -v v0.1.0

# Verify artifact signatures
gpg --verify SHA256SUMS.asc SHA256SUMS
sha256sum -c SHA256SUMS

# Verify cosign signature
cosign verify-blob --key cosign.pub --signature evefrontier-cli.sig evefrontier-cli
```

---

## Common Issues

| Issue | Solution |
|-------|----------|
| "gpg: signing failed: No secret key" | Run `gpg --list-secret-keys` and configure signing key |
| "cosign: no key" | Generate with `cosign generate-key-pair` |
| Missing cargo-sbom | Install with `cargo install cargo-sbom` |

---

**Full documentation**: See `docs/RELEASE.md` for complete procedures.
