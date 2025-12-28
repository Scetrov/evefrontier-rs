# Feature Specification: Release & Signing Documentation

**Feature ID**: 005-release-documentation  
**Created**: 2025-12-28  
**Status**: Draft

---

## Summary

Create comprehensive release and artifact signing documentation in `docs/RELEASE.md` that covers GPG
signing procedures, cosign attestations, SBOM generation, and the complete release workflow. This
documentation is a prerequisite for implementing CI release automation (ADR 0007).

## Background

The repository has established:

- **Constitution requirements** (v1.1.0): Releases MUST be GPG-signed, artifacts MUST include SBOMs,
  and cosign/GPG signatures are mandatory for release artifacts
- **ADR 0007**: Documents DevSecOps practices including attestations and signatures, but the actual
  procedures are not documented
- **Security instructions**: GPG commit signing is enforced, but release signing is undocumented

Current gaps:

| Area | Status | Gap |
|------|--------|-----|
| Commit signing | ✅ Documented | `.github/copilot-security-instructions.md` |
| Tag signing | ⚠️ Referenced | Constitution mentions `git tag -s` but no guide |
| Artifact signing | ❌ Missing | No cosign/GPG procedures documented |
| SBOM generation | ❌ Missing | Constitution requires but not documented |
| CI release job | ❌ Missing | Depends on this documentation |

## Goals

1. **Complete release workflow documentation** in `docs/RELEASE.md`
2. **GPG signing procedures** for tags and artifacts
3. **cosign integration** for OCI/binary signatures
4. **SBOM generation** using cargo-sbom or similar
5. **Spatial index artifact inclusion** in releases
6. **Attestation guidance** for CI pipelines
7. **Enable follow-on CI release automation** implementation

## Non-Goals

- Implementing the CI release workflow (separate TODO item)
- Publishing to crates.io (separate TODO item)
- Custom signing key infrastructure (use existing developer keys)
- Hardware security module (HSM) integration

## Requirements

### Functional Requirements

| ID   | Requirement                                              | Priority |
| ---- | -------------------------------------------------------- | -------- |
| FR-1 | Document complete release checklist with pre-conditions  | MUST     |
| FR-2 | Document GPG key requirements and verification           | MUST     |
| FR-3 | Document `git tag -s` procedure with examples            | MUST     |
| FR-4 | Document artifact signing with GPG (tarballs)            | MUST     |
| FR-5 | Document cosign setup and binary signature procedure     | MUST     |
| FR-6 | Document SBOM generation and inclusion                   | MUST     |
| FR-7 | Document spatial index artifact bundling                 | MUST     |
| FR-8 | Document CI integration points for future automation     | SHOULD   |
| FR-9 | Provide verification commands for consumers              | SHOULD   |
| FR-10 | Document rollback/revocation procedures                 | COULD    |

### Non-Functional Requirements

| ID    | Requirement                                       | Priority |
| ----- | ------------------------------------------------- | -------- |
| NFR-1 | Documentation follows existing docs/ style        | MUST     |
| NFR-2 | Commands tested and verified working              | MUST     |
| NFR-3 | Aligns with Constitution v1.1.0 release policies  | MUST     |
| NFR-4 | Cross-references ADR 0007 appropriately           | MUST     |

## Technical Design

### Document Structure

```
docs/RELEASE.md
├── Overview
├── Prerequisites
│   ├── GPG Key Setup
│   ├── cosign Installation
│   └── cargo-sbom Installation
├── Release Checklist
├── Version Bumping
├── Tag Signing
├── Building Release Artifacts
│   ├── Binary Compilation
│   ├── Spatial Index Generation
│   └── Package Assembly
├── Artifact Signing
│   ├── GPG Signatures
│   └── cosign Signatures
├── SBOM Generation
├── GitHub Release Creation
├── Verification
│   ├── Tag Verification
│   ├── Artifact Verification
│   └── SBOM Inspection
├── CI Integration Notes
└── Troubleshooting
```

### Key Technical Decisions

| Decision | Rationale |
|----------|-----------|
| GPG over pure cosign | Matches existing commit signing workflow |
| cargo-sbom for SBOM | Native Rust, CycloneDX format |
| Detached signatures | Standard practice, smaller main artifacts |
| Checksums file | SHA256SUMS for integrity verification |

### Release Artifact Contents

```
evefrontier-vX.Y.Z/
├── evefrontier-cli-linux-x86_64      # Binary
├── evefrontier-cli-linux-aarch64     # Binary (cross-compiled)
├── static_data.db.spatial.bin        # Spatial index artifact
├── SHA256SUMS                        # Checksums
├── SHA256SUMS.asc                    # GPG signature (armored)
├── evefrontier-vX.Y.Z.sbom.json      # SBOM (CycloneDX)
└── evefrontier-vX.Y.Z.sbom.json.asc  # SBOM signature (armored)
```

## Dependencies

| Dependency | Status | Notes |
|------------|--------|-------|
| ADR 0007 | ✅ Complete | DevSecOps practices documented |
| Constitution v1.1.0 | ✅ Complete | Release policy defined |
| GPG infrastructure | ✅ Complete | Commit signing already required |
| Binary builds | ✅ Complete | CLI compiles and works |
| Spatial index | ✅ Complete | `index-build` subcommand exists |

## Success Criteria

1. `docs/RELEASE.md` exists and passes markdown linting
2. Document covers all FR-1 through FR-7 requirements
3. Example commands can be executed by a developer with GPG configured
4. Cross-references to Constitution and ADR 0007 are accurate
5. TODO.md item is marked complete

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| cosign requires OIDC/keyless | Low | Medium | Document traditional key-based signing alternative |
| cargo-sbom not widely adopted | Low | Low | Document manual SBOM alternatives |
| CI secrets complexity | Medium | Medium | Document local-first workflow, CI as enhancement |

## References

- Constitution v1.1.0: Versioning & Release Policy
- ADR 0007: DevSecOps practices
- `.github/copilot-security-instructions.md`: GPG signing requirements
- [cosign documentation](https://docs.sigstore.dev/cosign/overview/)
- [cargo-sbom](https://crates.io/crates/cargo-sbom)
