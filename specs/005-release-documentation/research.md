# Research: Release & Signing Documentation

**Feature**: 005-release-documentation  
**Phase**: 0 - Research  
**Date**: 2025-12-28

---

## Research Questions

### 1. GPG Tag Signing Best Practices

**Question**: What are the best practices for GPG signing Git tags for releases?

**Findings**:

- Use `git tag -s vX.Y.Z -m "Release vX.Y.Z"` for signed tags
- Key should be published to keyserver (keys.openpgp.org recommended)
- Key fingerprint should be documented in repository (SECURITY.md or README)
- Verification: `git tag -v vX.Y.Z`

**Decision**: Document standard GPG tag signing with key publication guidance  
**Rationale**: Matches existing commit signing workflow, widely understood  
**Alternatives Considered**:

- SSH signing (newer, less tooling support)
- No signing (rejected per Constitution)

---

### 2. cosign for Binary Signing

**Question**: How should cosign be used for signing release binaries?

**Findings**:

- **Keyless signing** (OIDC): Uses Sigstore public good instance, requires OIDC identity
  - Pros: No key management, transparency log
  - Cons: Requires GitHub Actions or similar OIDC provider for CI
- **Key-based signing**: Traditional approach with generated key pair
  - Pros: Works locally and in CI, no external dependencies
  - Cons: Key management burden

```bash
# Key-based (recommended for local)
cosign generate-key-pair
cosign sign-blob --key cosign.key binary.tar.gz > binary.tar.gz.sig
cosign verify-blob --key cosign.pub --signature binary.tar.gz.sig binary.tar.gz

# Keyless (recommended for CI)
cosign sign-blob --yes binary.tar.gz > binary.tar.gz.sig  # Uses OIDC
```

**Decision**: Document both approaches; key-based for local releases, keyless for CI  
**Rationale**: Flexibility for different release scenarios  
**Alternatives Considered**:

- GPG-only (rejected: cosign adds transparency log benefits)
- Keyless-only (rejected: won't work for local releases)

---

### 3. SBOM Generation for Rust

**Question**: What is the best tool for generating SBOMs from Rust projects?

**Findings**:

| Tool            | Format         | Pros                            | Cons                  |
| --------------- | -------------- | ------------------------------- | --------------------- |
| cargo-sbom      | CycloneDX      | Native Rust, active development | Newer, less adoption  |
| cargo-cyclonedx | CycloneDX      | Well-maintained                 | Similar to cargo-sbom |
| syft            | CycloneDX/SPDX | Multi-language, mature          | External binary       |
| trivy           | CycloneDX/SPDX | Security scanning too           | Larger tool           |

**Installation & Usage**:

```bash
# cargo-sbom (recommended)
cargo install cargo-sbom
cargo sbom --output-format cyclonedx > sbom.json

# Alternatively with syft
syft . -o cyclonedx-json > sbom.json
```

**Decision**: Recommend cargo-sbom as primary, document syft as alternative  
**Rationale**: cargo-sbom is native Rust and integrates with cargo ecosystem  
**Alternatives Considered**: syft (document as CI alternative)

---

### 4. Release Artifact Structure

**Question**: What should be included in release artifacts?

**Findings** (based on Constitution requirements and industry standards):

**Minimum Release Contents**:

1. Compiled binaries (linux-x86_64, linux-aarch64)
2. SHA256SUMS file with checksums
3. GPG detached signature for SHA256SUMS
4. SBOM in CycloneDX JSON format
5. Optional: Spatial index artifact

**Artifact Naming Convention**:

```
evefrontier-cli-{version}-{os}-{arch}.tar.gz
evefrontier-cli-{version}-{os}-{arch}.tar.gz.sig   # cosign
SHA256SUMS
SHA256SUMS.asc                                      # GPG
evefrontier-{version}.sbom.json
```

**Decision**: Follow structure above with both GPG and cosign signatures  
**Rationale**: Comprehensive coverage for different consumer preferences

---

### 5. CI Integration Patterns

**Question**: How should release signing be integrated with GitHub Actions?

**Findings**:

**Keyless cosign in GitHub Actions**:

```yaml
- uses: sigstore/cosign-installer@v3
- run: cosign sign-blob --yes artifact.tar.gz > artifact.tar.gz.sig
  env:
    COSIGN_EXPERIMENTAL: 1 # For keyless
```

**GPG in GitHub Actions**:

```yaml
- name: Import GPG key
  run: |
    echo "${{ secrets.GPG_PRIVATE_KEY }}" | gpg --import
    echo "${{ secrets.GPG_PASSPHRASE }}" | gpg --batch --passphrase-fd 0 --pinentry-mode loopback --sign
```

**Decision**: Document CI patterns as "Integration Notes" section for future automation  
**Rationale**: This feature is documentation-only; actual CI is separate TODO

---

### 6. Verification Procedures

**Question**: How should consumers verify release artifacts?

**Findings**:

```bash
# Verify GPG tag
git fetch --tags
git tag -v v0.1.0

# Verify GPG artifact signature
gpg --verify SHA256SUMS.asc SHA256SUMS
sha256sum -c SHA256SUMS

# Verify cosign signature (keyless)
cosign verify-blob --certificate-identity=https://github.com/Scetrov/evefrontier-rs/.github/workflows/release.yml@refs/tags/v0.1.0 \
  --certificate-oidc-issuer=https://token.actions.githubusercontent.com \
  --signature artifact.sig artifact.tar.gz

# Verify cosign signature (key-based)
cosign verify-blob --key cosign.pub --signature artifact.sig artifact.tar.gz
```

**Decision**: Document all verification methods with copy-paste examples  
**Rationale**: Consumer trust requires clear verification path

---

## Research Summary

| Topic           | Decision               | Tool/Approach           |
| --------------- | ---------------------- | ----------------------- |
| Tag signing     | GPG signed tags        | `git tag -s`            |
| Binary signing  | cosign (key + keyless) | `cosign sign-blob`      |
| SBOM generation | cargo-sbom primary     | CycloneDX JSON          |
| Checksum format | SHA256SUMS             | `sha256sum`             |
| CI patterns     | Documented for future  | GitHub Actions + cosign |

---

## Outstanding Questions

1. **Spatial index versioning**: Should spatial index be versioned separately from CLI?
   - **Resolution**: Include with major releases, regenerate when schema changes
2. **Cross-compilation targets**: Which platforms should be officially supported?
   - **Resolution**: linux-x86_64 and linux-aarch64 initially, document others as
     community-contributed

3. **Key rotation policy**: How often should signing keys be rotated?
   - **Resolution**: Document annual review, no automatic rotation

---

## Next Steps

Phase 1 will produce:

- `docs/RELEASE.md` with complete procedures
- `quickstart.md` with abbreviated release checklist
