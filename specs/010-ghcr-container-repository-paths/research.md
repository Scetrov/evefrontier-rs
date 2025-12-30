# Research: GHCR Container Repository Path Standardization

**Date**: 2025-12-30  
**Status**: Complete

## Executive Summary

This research documents all files containing incorrect GHCR namespace references (`rslater-cs`) that need to be updated to the canonical namespace (`scetrov/evefrontier-rs`).

## Current State Analysis

### Search Results

**Pattern 1**: `ghcr.io/rslater-cs` (GHCR container registry)
- Found in 7 files, 11 occurrences

**Pattern 2**: `github.com/rslater-cs` (GitHub source repository)  
- Found in 5 files, 7 occurrences

### Files Requiring Updates

#### 1. CI/CD Workflows

| File | Line | Current | Target |
|------|------|---------|--------|
| `.github/workflows/docker-release.yml` | 40 | `IMAGE_PREFIX: ghcr.io/rslater-cs` | `IMAGE_PREFIX: ghcr.io/scetrov/evefrontier-rs` |
| `.github/workflows/docker-release.yml` | 39 | Comment referencing `rslater-cs` | Remove or update comment |

#### 2. Helm Chart

| File | Line | Current | Target |
|------|------|---------|--------|
| `charts/evefrontier/Chart.yaml` | 20 | `home: https://github.com/rslater-cs/evefrontier-rs` | `home: https://github.com/Scetrov/evefrontier-rs` |
| `charts/evefrontier/Chart.yaml` | 23 | `- https://github.com/rslater-cs/evefrontier-rs` | `- https://github.com/Scetrov/evefrontier-rs` |
| `charts/evefrontier/Chart.yaml` | 27 | `url: https://github.com/rslater-cs/evefrontier-rs` | `url: https://github.com/Scetrov/evefrontier-rs` |
| `charts/evefrontier/values.yaml` | 17 | `repository: ghcr.io/rslater-cs/evefrontier-service-route` | `repository: ghcr.io/scetrov/evefrontier-rs/evefrontier-service-route` |
| `charts/evefrontier/values.yaml` | 52 | `repository: ghcr.io/rslater-cs/evefrontier-service-scout-gates` | `repository: ghcr.io/scetrov/evefrontier-rs/evefrontier-service-scout-gates` |
| `charts/evefrontier/values.yaml` | 79 | `repository: ghcr.io/rslater-cs/evefrontier-service-scout-range` | `repository: ghcr.io/scetrov/evefrontier-rs/evefrontier-service-scout-range` |
| `charts/evefrontier/README.md` | 66 | `ghcr.io/rslater-cs/evefrontier-service-<name>` | `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-<name>` |

#### 3. Cargo.toml Files

| File | Line | Current | Target |
|------|------|---------|--------|
| `crates/evefrontier-service-route/Cargo.toml` | 8 | `repository = "https://github.com/rslater-cs/evefrontier-rs"` | `repository = "https://github.com/Scetrov/evefrontier-rs"` |
| `crates/evefrontier-service-scout-gates/Cargo.toml` | 8 | `repository = "https://github.com/rslater-cs/evefrontier-rs"` | `repository = "https://github.com/Scetrov/evefrontier-rs"` |
| `crates/evefrontier-service-scout-range/Cargo.toml` | 8 | `repository = "https://github.com/rslater-cs/evefrontier-rs"` | `repository = "https://github.com/Scetrov/evefrontier-rs"` |

#### 4. Documentation

| File | Line | Current | Target |
|------|------|---------|--------|
| `docs/DEPLOYMENT.md` | 42 | `git clone https://github.com/rslater-cs/evefrontier-rs.git` | `git clone https://github.com/Scetrov/evefrontier-rs.git` |
| `docs/RELEASE.md` | 771 | `ghcr.io/rslater-cs/evefrontier-service-route` | `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-route` |
| `docs/RELEASE.md` | 772 | `ghcr.io/rslater-cs/evefrontier-service-scout-gates` | `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-scout-gates` |
| `docs/RELEASE.md` | 773 | `ghcr.io/rslater-cs/evefrontier-service-scout-range` | `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-scout-range` |
| `docs/RELEASE.md` | 805 | `ghcr.io/rslater-cs/evefrontier-service-route:v0.1.0` | `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-route:v0.1.0` |
| `docs/RELEASE.md` | 813 | `ghcr.io/rslater-cs/evefrontier-service-${svc}:v0.1.0` | `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-${svc}:v0.1.0` |

### Files NOT Requiring Updates (Correct References)

The following files already use correct references to `Scetrov/evefrontier-rs`:
- `package.json` - ✅ Correct
- `Cargo.toml` (root) - ✅ Correct
- `README.md` - ✅ Correct
- `crates/evefrontier-lib/src/github.rs` - ✅ Correct
- `.github/workflows/release.yml` - ✅ Correct
- `docs/USAGE.md` - ✅ Correct
- `SECURITY.md` - ✅ Correct
- `terraform/modules/evefrontier-lambda/README.md` - ✅ Correct

## Design Decisions

### Decision 1: GHCR Path Structure

**Decision**: Use `ghcr.io/scetrov/evefrontier-rs/<service-name>` format

**Rationale**: 
- Follows GitHub's recommended pattern for repository-scoped packages
- Groups all project images under the repository namespace
- Matches the GitHub repository structure

**Alternatives Considered**:
- `ghcr.io/scetrov/<service-name>` - Rejected: doesn't scope to repository
- `ghcr.io/evefrontier/<service-name>` - Rejected: would require organization

### Decision 2: Case Sensitivity

**Decision**: Use lowercase `scetrov` for GHCR, preserve `Scetrov` case for GitHub URLs

**Rationale**:
- GHCR namespaces are case-insensitive but conventionally lowercase
- GitHub repository URLs are case-insensitive but display with original case
- Existing correct references use `Scetrov` for GitHub

## Validation Strategy

1. **Pre-merge**: Run `grep -r "rslater-cs"` to verify no references remain
2. **CI Validation**: Docker release workflow builds successfully
3. **Helm Lint**: `helm lint charts/evefrontier` passes
4. **Documentation Review**: Manual verification of updated docs

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Existing deployed systems reference old paths | Document migration path; old images remain available |
| CI workflow failures | Test on feature branch first |
| Helm chart breakage | Run `helm template` before merge |
