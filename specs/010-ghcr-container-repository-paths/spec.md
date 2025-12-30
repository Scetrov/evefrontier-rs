# Feature Specification: GHCR Container Repository Path Standardization

**Spec ID**: 010  
**Status**: Draft  
**Author**: GitHub Copilot  
**Date**: 2025-12-30

## Problem Statement

The repository currently has inconsistent GitHub Container Registry (GHCR) references:
- Some references use `ghcr.io/rslater-cs/...` (incorrect namespace)
- Some references use `github.com/rslater-cs/...` (incorrect namespace)
- The intended pattern is `ghcr.io/scetrov/evefrontier-rs/*` for container images
- The intended pattern is `github.com/Scetrov/evefrontier-rs` for source repository

This inconsistency causes:
1. Confusion about the canonical image locations
2. Potential deployment failures if incorrect paths are used
3. Documentation that doesn't match production infrastructure

## Requirements

### Functional Requirements

1. **FR-01**: All GHCR image references MUST use the pattern `ghcr.io/scetrov/evefrontier-rs/<service-name>`
2. **FR-02**: All GitHub repository references MUST use `github.com/Scetrov/evefrontier-rs`
3. **FR-03**: All image references MUST be consistent across:
   - CI/CD workflows (`.github/workflows/docker-release.yml`)
   - Helm charts (`charts/evefrontier/`)
   - Documentation (`docs/RELEASE.md`, `docs/DEPLOYMENT.md`)
   - Cargo.toml repository fields

### Non-Functional Requirements

1. **NFR-01**: Changes MUST NOT break existing CI/CD pipelines
2. **NFR-02**: Documentation MUST be updated atomically with code changes
3. **NFR-03**: Helm chart defaults MUST reflect the new paths

## Scope

### In Scope

1. Update `IMAGE_PREFIX` in `.github/workflows/docker-release.yml`
2. Update Helm chart `values.yaml` default repositories
3. Update Helm chart `Chart.yaml` URLs
4. Update Helm chart `README.md` examples
5. Update `docs/RELEASE.md` container image references
6. Update `docs/DEPLOYMENT.md` repository clone URLs
7. Update service crate `Cargo.toml` repository URLs

### Out of Scope

1. Creating new container images (this is a reference update only)
2. Migration of existing images (handled separately)
3. Changes to `Scetrov/evefrontier-rs` source repository references (already correct)

## Target Image Naming Convention

| Service | Old Path | New Path |
|---------|----------|----------|
| route | `ghcr.io/rslater-cs/evefrontier-service-route` | `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-route` |
| scout-gates | `ghcr.io/rslater-cs/evefrontier-service-scout-gates` | `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-scout-gates` |
| scout-range | `ghcr.io/rslater-cs/evefrontier-service-scout-range` | `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-scout-range` |

## Acceptance Criteria

1. All `rslater-cs` references removed from repository
2. All GHCR paths follow `ghcr.io/scetrov/evefrontier-rs/*` pattern
3. All GitHub source references use `github.com/Scetrov/evefrontier-rs`
4. CI workflow validates image paths (build succeeds)
5. Helm chart installs with correct default image paths
6. Documentation examples use correct paths

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Broken image pulls | High | Test Helm chart installation in staging |
| CI/CD failures | Medium | Run CI on feature branch before merge |
| Documentation drift | Low | Atomic PR with all changes |
