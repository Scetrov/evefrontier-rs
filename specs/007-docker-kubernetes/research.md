# Research: Docker Microservices & Kubernetes Deployment

**Feature**: 007-docker-kubernetes
**Date**: 2025-12-30

## Context

This research documents technical decisions for containerizing the EVE Frontier services.

## HTTP Framework Selection

### Options Evaluated

| Framework | Binary Size | Async | Tokio Native | Ecosystem Fit |
|-----------|-------------|-------|--------------|---------------|
| axum | ~2MB | Yes | Yes | Excellent (tower) |
| actix-web | ~3MB | Yes | No (actix-rt) | Good |
| warp | ~2MB | Yes | Yes | Good |
| rocket | ~4MB | Yes | Yes (0.5+) | Moderate |

### Decision: axum

- **Rationale**: Native tokio integration matches Lambda runtime ecosystem, type-safe extractors
  work well with existing request/response types, minimal binary size.
- **Key Dependencies**: `axum`, `tokio`, `tower-http` (CORS, tracing)

## Base Image Selection

### Options Evaluated

| Image | Size | Shell | Attack Surface | Rust Compat |
|-------|------|-------|----------------|-------------|
| gcr.io/distroless/cc | ~20MB | No | Minimal | Excellent |
| gcr.io/distroless/static | ~2MB | No | Minimal | Requires musl |
| alpine | ~5MB | Yes | Low | Requires musl |
| debian:slim | ~80MB | Yes | Medium | Excellent |
| scratch | ~0MB | No | Minimal | Requires static |

### Decision: gcr.io/distroless/cc-debian12

- **Rationale**: Includes glibc and libgcc for standard Rust binaries, no shell for security,
  ~20MB base layer acceptable for our use case.
- **Alternative for smaller images**: Use `gcr.io/distroless/static` with musl target if size
  becomes critical.

## Multi-Architecture Build Strategy

### Options Evaluated

1. **QEMU emulation in GitHub Actions** - Slow (30+ min for arm64)
2. **cross-rs** - Cross-compilation with Docker, ~5 min per arch
3. **cargo-zigbuild** - Cross-compilation with Zig, ~3 min per arch
4. **Native arm64 runners** - Fast but expensive

### Decision: cargo-zigbuild + docker buildx

- **Rationale**: Fast cross-compilation (~3 min per target), no QEMU overhead, simple setup.
- **Implementation**:
  ```yaml
  - uses: cargo-bins/cargo-binstall@main
  - run: cargo binstall cargo-zigbuild
  - run: cargo zigbuild --release --target x86_64-unknown-linux-gnu
  - run: cargo zigbuild --release --target aarch64-unknown-linux-gnu
  ```

## Service Architecture

### Request Flow

```
Client → Traefik Ingress → Service → Pod (axum) → evefrontier-lib
```

### Endpoint Mapping

| Endpoint | Service | Lambda Equivalent |
|----------|---------|-------------------|
| `/route` | evefrontier-service-route | evefrontier-lambda-route |
| `/scout/gates` | evefrontier-service-scout-gates | evefrontier-lambda-scout-gates |
| `/scout/range` | evefrontier-service-scout-range | evefrontier-lambda-scout-range |
| `/health` | All services | N/A (Lambda has built-in) |

### Health Check Strategy

Each service exposes:
- `GET /health/live` - Liveness probe (always returns 200 if process is running)
- `GET /health/ready` - Readiness probe (returns 200 if starmap is loaded)

## Helm Chart Structure

### Resources per Service

1. **Deployment** - Pod spec with:
   - Resource limits (configurable)
   - Liveness/readiness probes
   - Single replica (scalable via values.yaml)
   
2. **Service** - ClusterIP service exposing port 8080

3. **IngressRoute** (Traefik) - Path-based routing:
   - `/route` → route-service
   - `/scout/gates` → scout-gates-service
   - `/scout/range` → scout-range-service

### Configuration via values.yaml

```yaml
# Default values
route:
  replicas: 1
  resources:
    limits:
      memory: "256Mi"
      cpu: "500m"
    requests:
      memory: "128Mi"
      cpu: "100m"

scoutGates:
  replicas: 1
  resources:
    # same structure

scoutRange:
  replicas: 1
  resources:
    # same structure

ingress:
  enabled: true
  host: evefrontier.local
  tls:
    enabled: false
    secretName: ""
```

## Container Registry

### Options

1. **GitHub Container Registry (ghcr.io)** - Free for public repos, integrated with Actions
2. **Docker Hub** - Popular, rate limits on free tier
3. **AWS ECR** - Good for AWS deployments, not free

### Decision: GitHub Container Registry

- **Rationale**: Free, integrated with GitHub Actions, supports multi-arch manifests and
  cosign signatures.
- **Image names**:
  - `ghcr.io/scetrov/evefrontier-route:v0.1.0`
  - `ghcr.io/scetrov/evefrontier-scout-gates:v0.1.0`
  - `ghcr.io/scetrov/evefrontier-scout-range:v0.1.0`

## Security Scanning

### Tools

| Tool | Speed | Accuracy | Integration |
|------|-------|----------|-------------|
| Trivy | Fast | Good | GitHub Action |
| Grype | Fast | Good | GitHub Action |
| Snyk | Moderate | Excellent | GitHub App |

### Decision: Trivy

- **Rationale**: Fast, free, excellent GitHub Action support, scans both OS packages and
  application dependencies.
- **Configuration**:
  ```yaml
  - uses: aquasecurity/trivy-action@master
    with:
      image-ref: '${{ env.IMAGE }}'
      severity: 'CRITICAL,HIGH'
      exit-code: '1'  # Fail on high/critical
  ```

## Code Reuse Strategy

### Shared Types

The Lambda crates already define request/response types in `evefrontier-lambda-shared`. Options:

1. **Duplicate in service-shared** - Simple but violates DRY
2. **Extract to evefrontier-api-types** - Clean but adds a crate
3. **Make lambda-shared generic** - Reuse existing crate

### Decision: Extract core types to library

- Move core request/response types (sans Lambda-specific wrappers) to `evefrontier-lib` or a
  new `evefrontier-api-types` crate.
- Both Lambda and service crates depend on shared types.
- Lambda-specific wrappers (LambdaEvent, etc.) stay in lambda-shared.
- Service-specific wrappers (axum extractors, etc.) stay in service-shared.

**Scope for this feature**: Start with duplication (option 1), extract later if maintenance
burden increases.

## Docker Compose Configuration

```yaml
version: '3.8'

services:
  route:
    build:
      context: .
      dockerfile: crates/evefrontier-service-route/Dockerfile
    ports:
      - "8081:8080"
    # Note: Distroless images have no shell/wget/curl. Health checks rely on
    # Docker's TCP health check or Traefik's built-in health probing.
    # For explicit HTTP health checks, use a sidecar or rely on Traefik labels.
    labels:
      - "traefik.http.services.route.loadbalancer.healthcheck.path=/health/live"
      - "traefik.http.services.route.loadbalancer.healthcheck.interval=10s"

  scout-gates:
    build:
      context: .
      dockerfile: crates/evefrontier-service-scout-gates/Dockerfile
    ports:
      - "8082:8080"
    healthcheck:
      # same structure

  scout-range:
    build:
      context: .
      dockerfile: crates/evefrontier-service-scout-range/Dockerfile
    ports:
      - "8083:8080"
    healthcheck:
      # same structure

  traefik:
    image: traefik:v3.0
    command:
      - "--providers.docker=true"
      - "--entrypoints.web.address=:8080"
    ports:
      - "8080:8080"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
    depends_on:
      - route
      - scout-gates
      - scout-range
```

## References

- [axum documentation](https://docs.rs/axum)
- [Distroless containers](https://github.com/GoogleContainerTools/distroless)
- [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild)
- [Trivy GitHub Action](https://github.com/aquasecurity/trivy-action)
- [Helm chart best practices](https://helm.sh/docs/chart_best_practices/)
- [Traefik IngressRoute](https://doc.traefik.io/traefik/routing/providers/kubernetes-crd/)
