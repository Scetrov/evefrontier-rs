# Implementation Plan: Docker Microservices & Kubernetes Deployment

**Branch**: `007-docker-kubernetes` | **Date**: 2025-12-30 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-docker-kubernetes/spec.md`

## Summary

Create containerized microservice equivalents of the EVE Frontier Lambda functions, with Docker
Compose for local development, Helm charts for Kubernetes deployment, and CI/CD for automated
image building, scanning, signing, and publishing.

## Technical Context

**Language/Version**: Rust 1.91.1 (microservices), YAML (Docker/Kubernetes)
**Primary Dependencies**: axum (HTTP server), tokio (async runtime), Traefik (ingress)
**Storage**: SQLite dataset embedded at build time (same as Lambda)
**Testing**: Integration tests via `docker compose`, Helm chart validation
**Target Platform**: Linux containers (amd64, arm64)
**Project Type**: Infrastructure/DevOps feature
**Performance Goals**: <30s cold start, <50MB image size
**Constraints**: Must match Lambda API contracts exactly
**Scale/Scope**: 3 microservices, 1 Helm chart, 1 CI workflow

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. TDD | ✅ Applicable | Integration tests via docker compose and API contract tests |
| II. Library-First | ✅ Pass | Microservices reuse evefrontier-lib, no code duplication |
| III. ADR Required | ✅ Yes | Need ADR for containerization strategy decisions |
| IV. Clean Code | ✅ Applicable | Dockerfiles and Helm templates must be maintainable |
| V. Security-First | ✅ Critical | Image scanning, signing, minimal base images |
| VI. Testing Tiers | ✅ Applicable | Contract tests, integration tests |
| VII. Refactoring | ✅ N/A | New infrastructure, not refactoring existing code |

**Gate Status**: ✅ PASS - Requires ADR for containerization decisions.

## Project Structure

### Documentation (this feature)

```text
specs/007-docker-kubernetes/
├── plan.md              # This file
├── spec.md              # Feature specification with user stories
├── research.md          # Research on container frameworks, ingress options
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
# New microservice crates (mirror Lambda structure)
crates/
├── evefrontier-service-route/      # HTTP service for route endpoint
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs                 # axum HTTP server
│   └── Dockerfile                  # Multi-stage build
├── evefrontier-service-scout-gates/
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
│   └── Dockerfile
├── evefrontier-service-scout-range/
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
│   └── Dockerfile
└── evefrontier-service-shared/     # Shared HTTP infrastructure
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── health.rs               # Health check endpoints
        ├── request.rs              # Request parsing (reuse from lambda-shared)
        └── response.rs             # Response formatting

# Docker Compose for local development
docker-compose.yml

# Helm chart for Kubernetes
charts/
└── evefrontier/
    ├── Chart.yaml
    ├── values.yaml
    ├── templates/
    │   ├── _helpers.tpl
    │   ├── deployment-route.yaml
    │   ├── deployment-scout-gates.yaml
    │   ├── deployment-scout-range.yaml
    │   ├── service-route.yaml
    │   ├── service-scout-gates.yaml
    │   ├── service-scout-range.yaml
    │   ├── ingressroute.yaml       # Traefik IngressRoute (conditional)
    │   ├── middleware.yaml         # Rate limiting, CORS (optional)
    │   └── configmap.yaml          # Runtime configuration (optional)
    └── README.md

# CI workflow for container builds
.github/workflows/
└── docker-release.yml              # Build, scan, sign, publish

# ADR for containerization decisions
docs/adrs/
└── 0014-containerization-strategy.md
```

**Structure Decision**: Microservice crates mirror the Lambda structure for consistency. Each
service is a thin axum wrapper around evefrontier-lib. Shared HTTP infrastructure lives in
`evefrontier-service-shared` to avoid duplication with lambda-shared (request/response types
can be reused or extracted to a common crate if needed).

## Deliverables

### Phase 1: Foundation
1. **ADR 0014** - Containerization strategy documenting decisions on:
   - Base image choice (Distroless)
   - HTTP framework (axum)
   - Multi-arch build strategy
   - Image signing approach

### Phase 2: Microservice Crates
2. **evefrontier-service-shared** - Shared HTTP infrastructure
3. **evefrontier-service-route** - Route microservice with Dockerfile
4. **evefrontier-service-scout-gates** - Scout-gates microservice with Dockerfile
5. **evefrontier-service-scout-range** - Scout-range microservice with Dockerfile

### Phase 3: Local Development
6. **docker-compose.yml** - Local development configuration

### Phase 4: Kubernetes Deployment
7. **charts/evefrontier/** - Helm chart with Traefik ingress

### Phase 5: CI/CD
8. **.github/workflows/docker-release.yml** - Container build and publish workflow

### Phase 6: Documentation
9. **docs/DEPLOYMENT.md** - Update with Docker/Kubernetes deployment instructions
10. **docs/TODO.md** - Mark items complete

## Complexity Tracking

No violations expected - this is infrastructure work that follows established patterns from the
Lambda crates.

## Technical Decisions

### HTTP Framework: axum
- Native async Rust, built on tokio and hyper
- Type-safe extractors match well with existing request/response types
- Minimal binary size compared to actix-web
- Same ecosystem as Lambda runtime (tower middleware)

### Base Image: Distroless
- gcr.io/distroless/cc-debian12 for Rust binaries
- No shell, minimal attack surface
- ~20MB base layer

### Multi-arch Build Strategy
- Use cargo-zigbuild for cross-compilation in CI
- Build linux/amd64 and linux/arm64 in parallel
- Combine with docker buildx manifest

### Dataset Embedding
- Same approach as Lambda: include_bytes! for dataset at compile time
- Consider volume mount option for development (configurable via env var)
