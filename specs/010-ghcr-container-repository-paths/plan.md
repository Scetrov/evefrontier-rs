# Implementation Plan: GHCR Container Repository Path Standardization

**Branch**: `010-ghcr-container-repository-paths` | **Date**: 2025-12-30 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/010-ghcr-container-repository-paths/spec.md`

## Summary

Standardize all GitHub Container Registry (GHCR) references across the repository to use the canonical pattern `ghcr.io/scetrov/evefrontier-rs/<service-name>` instead of the incorrect `ghcr.io/rslater-cs/<service-name>` references. This is a configuration and documentation refactoring task with no code logic changes.

## Technical Context

**Language/Version**: YAML, Markdown, TOML (configuration files only)  
**Primary Dependencies**: GitHub Actions, Helm, Docker/Podman  
**Storage**: N/A  
**Testing**: CI workflow validation, Helm lint  
**Target Platform**: GitHub Container Registry (ghcr.io)  
**Project Type**: Configuration/documentation refactoring  
**Performance Goals**: N/A (no runtime impact)  
**Constraints**: Must not break CI/CD pipelines  
**Scale/Scope**: 19 files with `rslater-cs` references to update

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-Driven Development | ✅ PASS | No production code changes; CI validation sufficient |
| II. Library-First Architecture | ✅ N/A | Configuration changes only |
| III. ADRs (Mandatory) | ✅ PASS | No architectural decisions; simple path correction |
| IV. Clean Code | ✅ PASS | Improving consistency and clarity |
| V. Security-First | ✅ PASS | Correct image paths prevent supply chain confusion |
| VI. Testing Tiers | ✅ PASS | CI workflow runs as validation |
| VII. Refactoring | ✅ PASS | Single-concern PR, no behavior changes |

**Constitution Check Result**: ✅ PASS - Proceed to implementation

## Project Structure

### Documentation (this feature)

```text
specs/010-ghcr-container-repository-paths/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Files requiring changes (Phase 0)
└── quickstart.md        # Implementation guide (Phase 1)
```

### Files to Modify

```text
.github/workflows/
└── docker-release.yml       # IMAGE_PREFIX update

charts/evefrontier/
├── Chart.yaml               # home/sources URLs
├── README.md                # Example repository paths
└── values.yaml              # Default image repositories

crates/
├── evefrontier-service-route/Cargo.toml
├── evefrontier-service-scout-gates/Cargo.toml
└── evefrontier-service-scout-range/Cargo.toml

docs/
├── DEPLOYMENT.md            # Clone URL
└── RELEASE.md               # Container image references
```

**Structure Decision**: This is a multi-file configuration update. No new files are created; existing files are modified to correct namespace references.

## Complexity Tracking

No constitution violations - this is a straightforward find-and-replace operation across configuration and documentation files.
