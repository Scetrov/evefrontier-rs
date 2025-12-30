# Data Model: GHCR Container Repository Configuration

**Date**: 2025-12-30

## Overview

This document defines the configuration model for container registry references across the EVE Frontier project. Since this feature involves no runtime data structures, this document captures the configuration schema and naming conventions.

## Configuration Schema

### GHCR Image Reference Pattern

```
ghcr.io/<owner>/<repository>/<image-name>:<tag>
```

**Components**:
| Component | Value | Description |
|-----------|-------|-------------|
| Registry | `ghcr.io` | GitHub Container Registry |
| Owner | `scetrov` | GitHub username (lowercase) |
| Repository | `evefrontier-rs` | Repository name |
| Image Name | `evefrontier-service-*` | Service identifier |
| Tag | `v0.1.0`, `latest` | Version or alias |

### GitHub Repository Reference Pattern

```
https://github.com/<Owner>/<repository>
```

**Components**:
| Component | Value | Description |
|-----------|-------|-------------|
| Host | `github.com` | GitHub domain |
| Owner | `Scetrov` | GitHub username (original case) |
| Repository | `evefrontier-rs` | Repository name |

## Service Image Mapping

| Service | Crate | Image Path |
|---------|-------|------------|
| Route Planning | `evefrontier-service-route` | `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-route` |
| Gate Scout | `evefrontier-service-scout-gates` | `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-scout-gates` |
| Range Scout | `evefrontier-service-scout-range` | `ghcr.io/scetrov/evefrontier-rs/evefrontier-service-scout-range` |

## Configuration Files

### CI/CD Environment Variables

**Location**: `.github/workflows/docker-release.yml`

```yaml
env:
  REGISTRY: ghcr.io
  IMAGE_PREFIX: ghcr.io/scetrov/evefrontier-rs
```

### Helm Chart Values

**Location**: `charts/evefrontier/values.yaml`

```yaml
route:
  image:
    repository: ghcr.io/scetrov/evefrontier-rs/evefrontier-service-route
    tag: ""  # Defaults to chart appVersion
    pullPolicy: IfNotPresent
```

### Cargo Package Metadata

**Location**: `crates/evefrontier-service-*/Cargo.toml`

```toml
[package]
repository = "https://github.com/Scetrov/evefrontier-rs"
```

## Validation Rules

1. **GHCR paths MUST be lowercase**: `ghcr.io/scetrov/...` not `ghcr.io/Scetrov/...`
2. **GitHub URLs preserve case**: `github.com/Scetrov/...` 
3. **Image names match crate names**: `evefrontier-service-route` crate → `evefrontier-service-route` image
4. **Repository suffix required**: `evefrontier-rs/` must appear in GHCR path to scope images to repository
