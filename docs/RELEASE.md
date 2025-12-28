# Release Procedures

This document describes the complete release workflow for the EveFrontier Rust workspace, including
artifact signing, SBOM generation, and verification procedures. All releases **MUST** follow these
procedures to comply with the [Constitution v1.1.0](../README.md) and
[ADR 0007](adrs/0007-devsecops-practices.md).

## Table of Contents

- [Release Procedures](#release-procedures)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Prerequisites](#prerequisites)
    - [GPG Key Setup](#gpg-key-setup)
      - [Check Existing Keys](#check-existing-keys)
      - [Generate a New Key (if needed)](#generate-a-new-key-if-needed)
      - [Configure Git to Use Your Key](#configure-git-to-use-your-key)
      - [Publish Your Key](#publish-your-key)
    - [cosign Installation](#cosign-installation)
      - [Installation](#installation)
      - [Generate Key Pair (for local releases)](#generate-key-pair-for-local-releases)
    - [cargo-sbom Installation](#cargo-sbom-installation)
  - [Release Checklist](#release-checklist)
    - [Pre-Release Validation](#pre-release-validation)
    - [Build \& Sign](#build--sign)
    - [Publish](#publish)
  - [Version Bumping](#version-bumping)
    - [Semantic Versioning](#semantic-versioning)
    - [Cargo.toml Updates](#cargotoml-updates)
  - [Tag Signing](#tag-signing)
    - [Creating Signed Tags](#creating-signed-tags)
    - [Tag Verification](#tag-verification)
  - [Building Release Artifacts](#building-release-artifacts)
    - [Binary Compilation](#binary-compilation)
    - [Cross-Compilation (aarch64)](#cross-compilation-aarch64)
      - [Option 1: Using cross-rs (recommended)](#option-1-using-cross-rs-recommended)
      - [Option 2: Native toolchain](#option-2-native-toolchain)
    - [Spatial Index Generation](#spatial-index-generation)
    - [Package Assembly](#package-assembly)
  - [Artifact Signing](#artifact-signing)
    - [GPG Signatures](#gpg-signatures)
    - [cosign Signatures (Key-Based)](#cosign-signatures-key-based)
    - [cosign Signatures (Keyless)](#cosign-signatures-keyless)
  - [SBOM Generation](#sbom-generation)
    - [Alternative: syft](#alternative-syft)
  - [GitHub Release Creation](#github-release-creation)
    - [Create the Release](#create-the-release)
    - [Release Notes Template](#release-notes-template)
  - [Verification](#verification)
    - [Tag Verification](#tag-verification-1)
    - [Artifact Verification](#artifact-verification)
    - [SBOM Inspection](#sbom-inspection)
  - [Rollback \& Revocation](#rollback--revocation)
    - [Yanking a Release](#yanking-a-release)
    - [Revoking Signatures](#revoking-signatures)
    - [Emergency Response](#emergency-response)
  - [CI Integration Notes](#ci-integration-notes)
    - [GitHub Actions Workflow Structure](#github-actions-workflow-structure)
    - [GPG Key Management in CI](#gpg-key-management-in-ci)
    - [Required Secrets](#required-secrets)
  - [Troubleshooting](#troubleshooting)
    - [GPG Issues](#gpg-issues)
    - [cosign Issues](#cosign-issues)
    - [Build Issues](#build-issues)
    - [Checksum Issues](#checksum-issues)
  - [References](#references)
    - [Internal Documentation](#internal-documentation)
    - [External Resources](#external-resources)
    - [Tools](#tools)

---

## Overview

EveFrontier follows a security-first release process with multiple layers of verification:

1. **GPG-signed tags**: Every release tag is signed with the maintainer's GPG key
2. **Artifact checksums**: SHA256 checksums for all release files
3. **GPG signatures**: Detached signatures for checksum files
4. **cosign signatures**: Modern sigstore-based signatures for binary artifacts
5. **SBOM**: Software Bill of Materials in CycloneDX format

This multi-layered approach ensures:

- **Authenticity**: Releases originate from trusted maintainers
- **Integrity**: Artifacts have not been tampered with
- **Transparency**: Full dependency visibility via SBOM
- **Auditability**: Signatures are logged in public transparency logs (via cosign)

---

## Prerequisites

Before creating a release, ensure the following tools are installed and configured.

### GPG Key Setup

GPG is used for signing Git tags and creating detached signatures for release artifacts.

#### Check Existing Keys

```bash
# List existing secret keys
gpg --list-secret-keys --keyid-format LONG

# Example output:
# sec   rsa4096/ABCD1234EFGH5678 2024-01-01 [SC]
#       Key fingerprint = XXXX XXXX XXXX XXXX XXXX  XXXX ABCD 1234 EFGH 5678
# uid                 [ultimate] Your Name <your.email@example.com>
```

#### Generate a New Key (if needed)

```bash
# Generate a new GPG key (RSA 4096-bit recommended)
gpg --full-generate-key

# Select:
# - RSA and RSA (default)
# - 4096 bits
# - Key does not expire (or set expiration)
# - Your name and email (must match Git config)
```

#### Configure Git to Use Your Key

```bash
# Get your key ID
gpg --list-secret-keys --keyid-format LONG

# Configure Git
git config --global user.signingkey ABCD1234EFGH5678
git config --global commit.gpgsign true
git config --global tag.gpgsign true
```

#### Publish Your Key

For others to verify your signatures, publish your key to a keyserver:

```bash
# Export and publish to keys.openpgp.org (recommended)
gpg --export your.email@example.com | curl -T - https://keys.openpgp.org

# Or use SKS keyservers
gpg --keyserver hkps://keys.openpgp.org --send-keys ABCD1234EFGH5678
```

> **Important**: Document your key fingerprint in the repository's `SECURITY.md` file so consumers
> can verify signatures against a known good key.

### cosign Installation

[cosign](https://github.com/sigstore/cosign) provides modern container and artifact signing with
transparency log integration.

#### Installation

```bash
# macOS (Homebrew)
brew install cosign

# Linux (via Go)
go install github.com/sigstore/cosign/v2/cmd/cosign@latest

# Linux (binary download - check https://github.com/sigstore/cosign/releases for latest)
# Replace with the latest stable version from the releases page
COSIGN_VERSION="v3.0.1"  # Example version - verify latest at https://github.com/sigstore/cosign/releases
curl -LO "https://github.com/sigstore/cosign/releases/download/${COSIGN_VERSION}/cosign-linux-amd64"
curl -LO "https://github.com/sigstore/cosign/releases/download/${COSIGN_VERSION}/cosign-linux-amd64.sig"
curl -LO "https://github.com/sigstore/cosign/releases/download/${COSIGN_VERSION}/cosign_checksums.txt"

# Verify checksum before installing
sha256sum --ignore-missing -c cosign_checksums.txt
chmod +x cosign-linux-amd64
sudo mv cosign-linux-amd64 /usr/local/bin/cosign

# Verify installation
cosign version
```

#### Generate Key Pair (for local releases)

```bash
# Generate a key pair (you'll be prompted for a password)
cosign generate-key-pair

# This creates:
# - cosign.key (private key - keep secure!)
# - cosign.pub (public key - distribute with releases)
```

> **Security**: Store `cosign.key` securely. Never commit it to the repository. For CI releases, use
> keyless signing with OIDC instead.

### cargo-sbom Installation

[cargo-sbom](https://crates.io/crates/cargo-sbom) generates Software Bill of Materials in CycloneDX
format.

```bash
# Install cargo-sbom
cargo install cargo-sbom

# Verify installation
cargo sbom --help
```

> **Alternative**: For CI environments, you can use [syft](https://github.com/anchore/syft) instead.
> Install from a pinned release with checksum verification:
>
> ```bash
> # Replace with the latest stable version from https://github.com/anchore/syft/releases
> SYFT_VERSION="1.18.1"  # Example version - verify latest before use
> curl -LO "https://github.com/anchore/syft/releases/download/v${SYFT_VERSION}/syft_${SYFT_VERSION}_linux_amd64.tar.gz"
> curl -LO "https://github.com/anchore/syft/releases/download/v${SYFT_VERSION}/syft_${SYFT_VERSION}_checksums.txt"
> sha256sum -c syft_${SYFT_VERSION}_checksums.txt --ignore-missing
> tar -xzf syft_${SYFT_VERSION}_linux_amd64.tar.gz syft
> sudo mv syft /usr/local/bin/
> ```

---

## Release Checklist

Use this checklist before creating any release:

### Pre-Release Validation

- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Code is formatted: `cargo fmt --all -- --check`
- [ ] Security audit clean: `cargo audit`
- [ ] CHANGELOG.md updated with release date (move items from Unreleased)
- [ ] Version bumped in all Cargo.toml files
- [ ] No uncommitted changes: `git status`
- [ ] On `main` branch with latest changes: `git pull origin main`

### Build & Sign

- [ ] Release binaries compiled for all targets
- [ ] Spatial index generated (if schema changed)
- [ ] Checksums generated (SHA256SUMS)
- [ ] GPG signatures created (SHA256SUMS.asc)
- [ ] cosign signatures created (for binaries)
- [ ] SBOM generated

### Publish

- [ ] Signed tag created and pushed
- [ ] GitHub release created with all artifacts
- [ ] Signatures verified locally before announcement

---

## Version Bumping

### Semantic Versioning

EveFrontier follows [Semantic Versioning 2.0.0](https://semver.org/):

| Change Type | Version Bump | Examples                                    |
| ----------- | ------------ | ------------------------------------------- |
| **MAJOR**   | `X.0.0`      | Breaking API changes, schema changes        |
| **MINOR**   | `0.X.0`      | New features, backward-compatible additions |
| **PATCH**   | `0.0.X`      | Bug fixes, documentation, performance       |

### Cargo.toml Updates

Update version in all workspace crates:

```bash
# Files to update:
# - Cargo.toml (workspace)
# - crates/evefrontier-lib/Cargo.toml
# - crates/evefrontier-cli/Cargo.toml
# - crates/evefrontier-lambda-shared/Cargo.toml
# - crates/evefrontier-lambda-route/Cargo.toml
# - crates/evefrontier-lambda-scout-gates/Cargo.toml
# - crates/evefrontier-lambda-scout-range/Cargo.toml
```

Use workspace inheritance where possible:

```toml
# In workspace Cargo.toml
[workspace.package]
version = "0.2.0"

# In crate Cargo.toml
[package]
version.workspace = true
```

After updating versions, verify the build:

```bash
cargo build --workspace
cargo test --workspace
```

---

## Tag Signing

### Creating Signed Tags

All release tags **MUST** be GPG-signed per the Constitution:

```bash
# Create a signed tag
git tag -s v0.2.0 -m "Release v0.2.0"

# The -s flag signs with your configured GPG key
# The -m flag provides the tag message
```

For releases with detailed notes:

```bash
# Create tag with multi-line message
git tag -s v0.2.0 -m "Release v0.2.0

Highlights:
- New feature X
- Performance improvement Y
- Bug fix Z

See CHANGELOG.md for full details."
```

### Tag Verification

Verify your tag before pushing:

```bash
# Verify the tag signature
git tag -v v0.2.0

# Expected output:
# object abc123...
# type commit
# tag v0.2.0
# tagger Your Name <email> ...
#
# Release v0.2.0
# gpg: Signature made ...
# gpg: Good signature from "Your Name <email>"
```

Push the signed tag:

```bash
# Push the tag to origin
git push origin v0.2.0

# Or push all tags
git push origin --tags
```

---

## Building Release Artifacts

### Binary Compilation

Build optimized release binaries:

```bash
# Build release binary for current platform
cargo build --release -p evefrontier-cli

# The binary is at: target/release/evefrontier-cli
```

For reproducible builds, ensure:

- Rust version matches `.rust-toolchain` (currently 1.91.1)
- Build from a clean checkout
- Use `--locked` to respect Cargo.lock

```bash
# Clean build from scratch
cargo clean
cargo build --release --locked -p evefrontier-cli
```

### Cross-Compilation (aarch64)

For ARM64/aarch64 targets (e.g., AWS Graviton, Apple Silicon):

#### Option 1: Using cross-rs (recommended)

```bash
# Install cross
cargo install cross

# Build for aarch64
cross build --release --target aarch64-unknown-linux-gnu -p evefrontier-cli
```

#### Option 2: Native toolchain

```bash
# Install target (Ubuntu/Debian)
sudo apt install gcc-aarch64-linux-gnu

# Add Rust target
rustup target add aarch64-unknown-linux-gnu

# Configure linker in .cargo/config.toml
# [target.aarch64-unknown-linux-gnu]
# linker = "aarch64-linux-gnu-gcc"

# Build
cargo build --release --target aarch64-unknown-linux-gnu -p evefrontier-cli
```

### Spatial Index Generation

If the database schema has changed, regenerate the spatial index:

```bash
# Build the CLI first
cargo build --release -p evefrontier-cli

# Generate spatial index from the dataset
./target/release/evefrontier-cli index-build

# The index is saved alongside the database:
# ~/.local/share/evefrontier/static_data.db.spatial.bin
```

> **Note**: Include the spatial index in releases for major version bumps or when the underlying
> dataset schema changes.

### Package Assembly

Create release tarballs and checksums:

```bash
# Set version
VERSION="0.2.0"

# Create release directory
mkdir -p "release/evefrontier-v${VERSION}"

# Copy binaries
cp target/release/evefrontier-cli "release/evefrontier-v${VERSION}/evefrontier-cli-linux-x86_64"
cp target/aarch64-unknown-linux-gnu/release/evefrontier-cli "release/evefrontier-v${VERSION}/evefrontier-cli-linux-aarch64"

# Copy spatial index (if applicable)
cp ~/.local/share/evefrontier/static_data.db.spatial.bin "release/evefrontier-v${VERSION}/"

# Create tarballs
cd release
tar -czvf "evefrontier-cli-${VERSION}-linux-x86_64.tar.gz" "evefrontier-v${VERSION}/evefrontier-cli-linux-x86_64"
tar -czvf "evefrontier-cli-${VERSION}-linux-aarch64.tar.gz" "evefrontier-v${VERSION}/evefrontier-cli-linux-aarch64"

# Generate checksums
sha256sum *.tar.gz > SHA256SUMS
sha256sum "evefrontier-v${VERSION}/static_data.db.spatial.bin" >> SHA256SUMS
```

---

## Artifact Signing

### GPG Signatures

Sign the checksums file with GPG:

```bash
# Create detached ASCII-armored signature
gpg --armor --detach-sign SHA256SUMS

# This creates SHA256SUMS.asc
# Verify it was created correctly
gpg --verify SHA256SUMS.asc SHA256SUMS
```

### cosign Signatures (Key-Based)

For local releases, use your cosign key pair:

```bash
# Sign each tarball (cosign v3+ uses --bundle format)
cosign sign-blob --key cosign.key \
  --bundle "evefrontier-cli-${VERSION}-linux-x86_64.tar.gz.bundle" \
  "evefrontier-cli-${VERSION}-linux-x86_64.tar.gz"

cosign sign-blob --key cosign.key \
  --bundle "evefrontier-cli-${VERSION}-linux-aarch64.tar.gz.bundle" \
  "evefrontier-cli-${VERSION}-linux-aarch64.tar.gz"

# Verify signatures
cosign verify-blob --key cosign.pub \
  --bundle "evefrontier-cli-${VERSION}-linux-x86_64.tar.gz.bundle" \
  "evefrontier-cli-${VERSION}-linux-x86_64.tar.gz"
```

> **Note**: This documentation standardizes on cosign v3's `--bundle` format throughout. The bundle
> format combines the signature, certificate, and timestamps into a single `.bundle` file. Earlier
> cosign versions used separate `.sig` files with `--signature` flag.

### cosign Signatures (Keyless)

For CI releases, use keyless signing with OIDC:

```bash
# Keyless signing (requires OIDC identity provider)
# This is typically done in GitHub Actions
cosign sign-blob --yes \
  --bundle "evefrontier-cli-${VERSION}-linux-x86_64.tar.gz.bundle" \
  "evefrontier-cli-${VERSION}-linux-x86_64.tar.gz"
```

> **Note**: cosign v3+ no longer requires `COSIGN_EXPERIMENTAL=1` for keyless signing.

Keyless signatures are recorded in the Sigstore transparency log and can be verified using the
certificate identity:

```bash
REPO="https://github.com/Scetrov/evefrontier-rs"
WORKFLOW=".github/workflows/release.yml"

cosign verify-blob \
  --certificate-identity="${REPO}/${WORKFLOW}@refs/tags/v${VERSION}" \
  --certificate-oidc-issuer="https://token.actions.githubusercontent.com" \
  --bundle "evefrontier-cli-${VERSION}-linux-x86_64.tar.gz.bundle" \
  "evefrontier-cli-${VERSION}-linux-x86_64.tar.gz"
```

---

## SBOM Generation

Generate a Software Bill of Materials for the release:

```bash
# Generate SBOM in CycloneDX JSON format
cargo sbom --output-format cyclonedx > "evefrontier-v${VERSION}.sbom.json"

# Sign the SBOM
gpg --armor --detach-sign "evefrontier-v${VERSION}.sbom.json"
# Creates: evefrontier-v${VERSION}.sbom.json.asc
```

The SBOM includes:

- All direct and transitive dependencies
- License information
- Package versions and checksums
- Dependency relationships

### Alternative: syft

For CI environments or broader language support:

```bash
syft . -o cyclonedx-json > "evefrontier-v${VERSION}.sbom.json"
```

---

## GitHub Release Creation

### Create the Release

1. Go to **Releases** → **Draft a new release**
2. Select the signed tag (e.g., `v0.2.0`)
3. Set release title: `v0.2.0`
4. Copy release notes from CHANGELOG.md
5. Upload all artifacts:
   - `evefrontier-cli-${VERSION}-linux-x86_64.tar.gz`
   - `evefrontier-cli-${VERSION}-linux-x86_64.tar.gz.bundle`
   - `evefrontier-cli-${VERSION}-linux-aarch64.tar.gz`
   - `evefrontier-cli-${VERSION}-linux-aarch64.tar.gz.bundle`
   - `SHA256SUMS`
   - `SHA256SUMS.asc`
   - `evefrontier-v${VERSION}.sbom.json`
   - `evefrontier-v${VERSION}.sbom.json.asc`
   - `static_data.db.spatial.bin` (if applicable)
   - `cosign.pub` (public key for verification)

### Release Notes Template

````markdown
## What's Changed

### Breaking Changes

- None

### Features

- Feature description (#PR)

### Bug Fixes

- Fix description (#PR)

### Documentation

- Doc update (#PR)

## Verification

### GPG Signature

```bash
gpg --verify SHA256SUMS.asc SHA256SUMS
sha256sum -c SHA256SUMS
```

### cosign Signature

```bash
cosign verify-blob --key cosign.pub \
  --bundle evefrontier-cli-0.2.0-linux-x86_64.tar.gz.bundle \
  evefrontier-cli-0.2.0-linux-x86_64.tar.gz
```

**Full Changelog**: https://github.com/Scetrov/evefrontier-rs/compare/v0.1.0...v0.2.0
````

---

## Verification

### Tag Verification

Consumers can verify the Git tag signature:

```bash
# Fetch tags
git fetch --tags

# Verify tag signature
git tag -v v0.2.0

# Import maintainer's public key (if not already imported)
gpg --keyserver hkps://keys.openpgp.org --recv-keys ABCD1234EFGH5678
```

### Artifact Verification

Verify downloaded artifacts:

```bash
# Download artifacts
curl -LO https://github.com/Scetrov/evefrontier-rs/releases/download/v0.2.0/SHA256SUMS
curl -LO https://github.com/Scetrov/evefrontier-rs/releases/download/v0.2.0/SHA256SUMS.asc
curl -LO https://github.com/Scetrov/evefrontier-rs/releases/download/v0.2.0/evefrontier-cli-0.2.0-linux-x86_64.tar.gz

# Verify GPG signature
gpg --verify SHA256SUMS.asc SHA256SUMS

# Verify checksums
sha256sum -c SHA256SUMS --ignore-missing

# Verify cosign signature (key-based, cosign v3+ bundle format)
curl -LO https://github.com/Scetrov/evefrontier-rs/releases/download/v0.2.0/cosign.pub
curl -LO https://github.com/Scetrov/evefrontier-rs/releases/download/v0.2.0/evefrontier-cli-0.2.0-linux-x86_64.tar.gz.bundle

cosign verify-blob --key cosign.pub \
  --bundle evefrontier-cli-0.2.0-linux-x86_64.tar.gz.bundle \
  evefrontier-cli-0.2.0-linux-x86_64.tar.gz
```

### SBOM Inspection

Inspect the SBOM for dependency information:

```bash
# Download SBOM
curl -LO https://github.com/Scetrov/evefrontier-rs/releases/download/v0.2.0/evefrontier-v0.2.0.sbom.json

# View with jq
jq '.components[] | {name, version, licenses}' evefrontier-v0.2.0.sbom.json

# Or use CycloneDX CLI
# npm install -g @cyclonedx/cyclonedx-cli
cyclonedx validate --input-file evefrontier-v0.2.0.sbom.json
```

---

## Rollback & Revocation

### Yanking a Release

If a critical issue is discovered after release:

1. **Do NOT delete the release** - this breaks existing links and signatures
2. Mark the release as "Pre-release" in GitHub
3. Add a prominent warning to the release notes:
   ```markdown
   > ⚠️ **WARNING**: This release contains a critical bug. Please use v0.2.1 instead.
   ```
4. Create a patch release with the fix

### Revoking Signatures

If a signing key is compromised:

1. **GPG Key Revocation**:

   ```bash
   # Generate revocation certificate (do this BEFORE key compromise if possible)
   gpg --gen-revoke ABCD1234EFGH5678 > revoke.asc

   # Import revocation certificate
   gpg --import revoke.asc

   # Publish revoked key
   gpg --keyserver hkps://keys.openpgp.org --send-keys ABCD1234EFGH5678
   ```

2. **Update SECURITY.md** with:
   - Notice of key compromise
   - Date of compromise
   - Which releases may be affected
   - New key fingerprint

3. **Re-sign affected releases** with new key (create new patch versions)

### Emergency Response

For security vulnerabilities:

1. Follow the [Security Policy](../SECURITY.md) disclosure process
2. Coordinate with downstream users before public announcement
3. Prepare a patch release before disclosing
4. Update CHANGELOG.md with security advisory reference

---

## CI Integration Notes

Future CI automation should implement the following patterns:

### GitHub Actions Workflow Structure

> **Security Note**: Third-party actions are pinned to full commit SHAs to prevent supply chain
> attacks. Update these intentionally as part of your dependency management process. See
> [GitHub's security hardening guide](https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions#using-third-party-actions).

```yaml
name: Release

on:
  push:
    tags:
      - "v*"

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest

    runs-on: ${{ matrix.os }}
    steps:
      # Pin actions to full commit SHAs for supply chain security
      # IMPORTANT: Verify SHA-to-version mappings before copying these examples.
      # Use: gh api repos/{owner}/{repo}/git/refs/tags/{tag} to confirm SHAs match claimed versions.
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2
      - uses: dtolnay/rust-toolchain@a54c7afa936fefeb4456b2dd8068152669aa8203 # stable
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: actions/upload-artifact@b4b15b8c7c6ac21ea08fcf65892d2ee8f75cf882 # v4.4.3
        with:
          name: binary-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/evefrontier-cli

  sign:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      id-token: write # For keyless signing
    steps:
      - uses: sigstore/cosign-installer@dc72c7d5c4d10cd6bcb8cf6e3fd625a9e5e537da # v3.7.0
      - uses: actions/download-artifact@fa0a91b85d4f404e444e00e005971372dc801d16 # v4.1.8

      # Keyless signing (cosign v3+ uses --bundle format)
      - run: |
          cosign sign-blob --yes \
            --bundle binary.bundle \
            binary-x86_64-unknown-linux-gnu/evefrontier-cli

  release:
    needs: sign
    runs-on: ubuntu-latest
    steps:
      - uses: softprops/action-gh-release@c95fe1489396fe8a9eb87c0abf8aa5b2ef267fda # v2.2.1
        with:
          files: |
            *.tar.gz
            *.bundle
            SHA256SUMS
            SHA256SUMS.asc
            *.sbom.json
```

### GPG Key Management in CI

```yaml
- name: Import GPG key
  env:
    GPG_PRIVATE_KEY: ${{ secrets.GPG_PRIVATE_KEY }}
    GPG_PASSPHRASE: ${{ secrets.GPG_PASSPHRASE }}
  run: |
    echo "$GPG_PRIVATE_KEY" | gpg --batch --import
    echo "$GPG_PASSPHRASE" | gpg --batch --passphrase-fd 0 \
      --pinentry-mode loopback --armor --detach-sign SHA256SUMS
```

### Required Secrets

| Secret            | Description               |
| ----------------- | ------------------------- |
| `GPG_PRIVATE_KEY` | ASCII-armored private key |
| `GPG_PASSPHRASE`  | Key passphrase            |

> **Note**: For keyless cosign signing, no secrets are required - OIDC identity is provided by
> GitHub Actions automatically.

---

## Troubleshooting

### GPG Issues

| Issue                                                 | Solution                                           |
| ----------------------------------------------------- | -------------------------------------------------- |
| "gpg: signing failed: No secret key"                  | Run `gpg --list-secret-keys` and verify key exists |
| "gpg: signing failed: Inappropriate ioctl for device" | Set `export GPG_TTY=$(tty)`                        |
| "gpg: public key not found"                           | Import with `gpg --recv-keys <KEYID>`              |
| Tag verification fails                                | Import maintainer's public key from keyserver      |

### cosign Issues

| Issue                  | Solution                                                                 |
| ---------------------- | ------------------------------------------------------------------------ |
| "cosign: no key"       | Generate with `cosign generate-key-pair`                                 |
| Keyless signing fails  | For v2: set `COSIGN_EXPERIMENTAL=1`; for v3+: verify OIDC setup and logs |
| OIDC token error       | Verify `id-token: write` permission in workflow                          |
| Transparency log error | Check network connectivity to rekor.sigstore.dev                         |

### Build Issues

| Issue                   | Solution                                      |
| ----------------------- | --------------------------------------------- |
| Cross-compilation fails | Install target toolchain and linker           |
| Version mismatch        | Ensure all Cargo.toml files have same version |
| Spatial index outdated  | Regenerate with `evefrontier-cli index-build` |

### Checksum Issues

| Issue                                                   | Solution                                  |
| ------------------------------------------------------- | ----------------------------------------- |
| `sha256sum: WARNING: 1 computed checksum did NOT match` | Re-download artifact                      |
| File not in SHA256SUMS                                  | Check release assets, file may be missing |

---

## References

### Internal Documentation

- [Constitution v1.1.0](../README.md) - Release policy requirements
- [ADR 0007: DevSecOps Practices](adrs/0007-devsecops-practices.md) - Signing and attestation
  requirements
- [SECURITY.md](../SECURITY.md) - Security policy and key fingerprints
- [CHANGELOG.md](../CHANGELOG.md) - Release history

### External Resources

- [GPG Handbook](https://www.gnupg.org/gph/en/manual.html)
- [cosign Documentation](https://docs.sigstore.dev/cosign/overview/)
- [Sigstore](https://www.sigstore.dev/) - Keyless signing infrastructure
- [CycloneDX Specification](https://cyclonedx.org/specification/overview/)
- [cargo-sbom](https://crates.io/crates/cargo-sbom)
- [Semantic Versioning](https://semver.org/)

### Tools

| Tool       | Purpose                          | Installation                         |
| ---------- | -------------------------------- | ------------------------------------ |
| GPG        | Tag and artifact signing         | System package manager               |
| cosign     | Binary signing with transparency | `brew install cosign` or [GitHub][1] |
| cargo-sbom | SBOM generation                  | `cargo install cargo-sbom`           |
| syft       | Alternative SBOM tool            | [GitHub][2]                          |
| sha256sum  | Checksum generation              | GNU coreutils (pre-installed)        |

[1]: https://github.com/sigstore/cosign/releases
[2]: https://github.com/anchore/syft/releases
