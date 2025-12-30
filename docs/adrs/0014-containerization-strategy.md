# ADR-0014: Containerization Strategy for Microservices

## Status

Accepted

## Context

The EVE Frontier services are currently deployed as AWS Lambda functions. To support self-hosting,
local development, and Kubernetes deployment scenarios, we need containerized microservice
equivalents. Key decisions include:

1. **HTTP Framework**: Lambda uses `lambda_runtime`; microservices need a standalone HTTP server.
2. **Base Image**: Container size and security posture are critical for production deployment.
3. **Multi-architecture Support**: Need to support both amd64 and arm64 for cloud and edge deployment.
4. **Build Tooling**: Cross-compilation strategy for multi-arch builds in CI.
5. **Container Registry**: Where to publish container images.
6. **Image Signing**: Supply chain security for published images.

## Decision

### HTTP Framework: axum

We will use `axum` as the HTTP framework for all microservices.

- Native async Rust built on tokio and hyper
- Type-safe extractors work well with existing request/response types
- Minimal binary size (~2MB contribution) compared to actix-web (~3MB)
- Same tower middleware ecosystem as Lambda runtime for consistent patterns
- Excellent documentation and active maintenance

### Base Image: Distroless

We will use `gcr.io/distroless/cc-debian12` as the runtime base image.

- Minimal attack surface (no shell, no package manager)
- Contains only glibc and libgcc needed for Rust binaries
- ~20MB base layer keeps total image size under 50MB target
- Supported by Google with regular security updates

### Multi-architecture Strategy: cargo-zigbuild

We will use `cargo-zigbuild` for cross-compilation in CI.

- Fast cross-compilation (~3 min per target vs 30+ min with QEMU)
- Produces native binaries without emulation overhead
- Targets: `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`
- Combined with docker buildx for multi-arch manifests

### Container Registry: GitHub Container Registry (ghcr.io)

We will publish images to GitHub Container Registry.

- Free for public repositories
- Integrated with GitHub Actions (OIDC authentication)
- Native support for multi-arch manifests
- Supports cosign signatures and SBOM attestations

### Image Signing: Cosign with OIDC

We will sign images using Sigstore cosign with keyless OIDC identity.

- No key management required (uses GitHub Actions OIDC token)
- Verifiable provenance tied to repository and workflow
- Industry standard for container signing
- Compatible with Kubernetes admission controllers (e.g., Kyverno, Gatekeeper)

## Rationale

### Why axum over actix-web?

Both are excellent frameworks. We chose axum because:

1. Native tokio integration matches Lambda runtime patterns
2. Tower middleware allows code sharing with Lambda handlers
3. Slightly smaller binary size helps meet 50MB image target
4. The team has more experience with the tower ecosystem

### Why Distroless over Alpine?

Alpine requires musl libc compilation which can cause subtle runtime differences:

1. Distroless uses glibc matching our development and Lambda environments
2. No shell means fewer CVEs and smaller attack surface
3. Well-documented by Google with clear security update policy

### Why cargo-zigbuild over cross-rs?

1. Faster builds (3 min vs 5+ min per target)
2. Simpler setup in CI (single binary install)
3. More predictable behavior with fewer Docker-in-Docker issues

## Consequences

### Positive

- Consistent development experience across Lambda and container deployments
- Small, secure container images (<50MB)
- Fast CI builds with cross-compilation
- Strong supply chain security with signed images

### Negative

- Debugging Distroless containers requires additional tooling (no shell)
- cargo-zigbuild adds a build dependency not used elsewhere
- Multiple deployment targets (Lambda + containers) increase maintenance burden

### Follow-up Tasks

1. Create Dockerfiles for each microservice using multi-stage builds
2. Set up docker-compose.yml for local development
3. Create Helm chart for Kubernetes deployment
4. Configure CI workflow for image builds

## References

- [axum documentation](https://docs.rs/axum)
- [Distroless containers](https://github.com/GoogleContainerTools/distroless)
- [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild)
- [Sigstore cosign](https://docs.sigstore.dev/signing/signing_with_containers/)
- [ADR 0006: Software Components](./0006-software-components.md)
- [ADR 0007: DevSecOps Practices](./0007-devsecops-practices.md)
