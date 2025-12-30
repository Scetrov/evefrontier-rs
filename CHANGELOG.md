# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

## [0.1.0-alpha.1] - 2025-12-30

### Added

- **Core Library** (`evefrontier-lib`)
  - Dataset downloader with GitHub release caching, atomic writes, and zip extraction
  - Starmap loader with runtime schema detection for multiple dataset formats
  - Graph builders for gate-only, spatial, and hybrid routing modes
  - Pathfinding algorithms: BFS (unweighted), Dijkstra (weighted), A\* (heuristic-guided)
  - Route constraints: max jump distance, avoided systems, gate-free travel, temperature limits
  - KD-tree spatial index with postcard+zstd serialization and SHA-256 checksums
  - Temperature-aware nearest-neighbor queries with min_external_temp filtering
  - Fuzzy system name matching with Jaro-Winkler similarity suggestions
  - Output formatters: plain text, rich text, JSON, in-game note format

- **CLI** (`evefrontier-cli`)
  - `download` command for dataset acquisition with caching
  - `route` command with `--algorithm`, `--format`, `--max-jump`, `--avoid`, `--max-temp` flags
  - `index-build` command to precompute spatial index artifacts
  - `index-verify` command for spatial index freshness verification
  - Global options: `--data-dir`, `--format`, `--no-logo`, `--dataset`
  - Enhanced output format with system details (planets, moons, temperature)
  - Sci-fi styled CLI banner with color support

- **AWS Lambda Functions**
  - `evefrontier-lambda-route` - Route planning endpoint
  - `evefrontier-lambda-scout-gates` - Gate-connected neighbor lookup
  - `evefrontier-lambda-scout-range` - Spatial range queries with temperature filtering
  - `evefrontier-lambda-shared` - Common infrastructure (request validation, RFC 9457 errors)
  - Terraform module at `terraform/modules/evefrontier-lambda/`

- **Docker Microservices**
  - `evefrontier-service-route` - HTTP route planning service
  - `evefrontier-service-scout-gates` - HTTP gate lookup service
  - `evefrontier-service-scout-range` - HTTP spatial query service
  - `evefrontier-service-shared` - Axum 0.8 HTTP infrastructure, health checks, metrics
  - Multi-stage Dockerfiles with Distroless base images (~30MB containers)
  - Helm chart at `charts/evefrontier/` with deployments, services, ingress, probes
  - docker-compose.yml with Traefik reverse proxy for local development

- **Observability**
  - Prometheus metrics integration with `record_request_metrics()`
  - Structured JSON logging with RFC3339 timestamps
  - Health endpoints: `/health/live`, `/health/ready` with dependency checks
  - Grafana dashboard JSON and Prometheus alerting rules

- **CI/CD**
  - Release workflow with multi-arch builds (x86_64, aarch64)
  - Keyless cosign signing via GitHub OIDC
  - SBOM generation (CycloneDX format)
  - Docker release workflow with Trivy scanning
  - ADR governance workflow enforcing naming and immutability
  - Changelog guard requiring CHANGELOG.md updates
  - Nightly dependency outdated reports
  - Spatial index freshness CI verification

- **Documentation**
  - Comprehensive RELEASE.md with GPG signing and verification procedures
  - DEPLOYMENT.md for Lambda and Kubernetes deployment
  - ARCHITECTURE.md with Mermaid diagrams
  - OBSERVABILITY.md for metrics, logging, and alerting
  - USAGE.md with CLI examples and library API
  - 14 Architecture Decision Records (ADRs)

- **Testing**
  - 100+ unit and integration tests across all crates
  - Test fixture with real e6c3 dataset systems
  - CLI integration tests with assert_cmd
  - Lambda unit tests with shared test utilities

### Fixed

- Standardize GHCR container registry paths to `ghcr.io/scetrov/evefrontier-rs/*`
- Fix CLI integration tests for GitHub API rate limiting
- Fix temperature calculation bugs and formula accuracy
- Fix JSON output format pollution from tracing logs
- Fix race condition in dataset download tests

### Known Issues

- `kiddo 5.2.3` depends on yanked `cmov 0.3.1` (not a security vulnerability, just yanked)
  - Waiting for upstream fix; cargo audit passes with warning only

