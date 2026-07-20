## ADDED Requirements

### Requirement: Immutable container base images
Every checked-in or generated container build used for local development, CI, or release SHALL reference each external base image with a human-readable tag and an immutable SHA-256 digest.

#### Scenario: Service image base resolution
- **WHEN** any route, scout-gates, or scout-range service Dockerfile is inspected
- **THEN** its Rust builder and runtime base references include immutable SHA-256 digests

#### Scenario: Release image base resolution
- **WHEN** the multi-architecture release workflow builds a service image
- **THEN** it consumes a checked-in Dockerfile whose runtime base is pinned by SHA-256 digest

### Requirement: Multi-architecture digest compatibility
Container base-image digests used by the release process MUST resolve as manifest-list digests compatible with every architecture published by the workflow.

#### Scenario: Release platforms are validated
- **WHEN** a base-image digest is introduced or refreshed
- **THEN** image builds resolve successfully for both `linux/amd64` and `linux/arm64`

### Requirement: Controlled base-image updates
The repository SHALL automate checks for newer Docker base tags or digests across all service and release Dockerfiles and SHALL document a manual digest refresh and verification procedure for any reference the automation cannot update.

#### Scenario: Upstream base image changes
- **WHEN** an upstream Rust or distroless base image receives a supported update
- **THEN** the maintainer receives a reviewable update that preserves immutable digest pinning

#### Scenario: Automated refresh is unsupported
- **WHEN** dependency automation cannot refresh a pinned image reference
- **THEN** release documentation provides commands to resolve the manifest-list digest, verify its platforms, and update every affected reference consistently

### Requirement: Container hardening regression validation
Base-image pinning or refreshes SHALL preserve successful service image builds, vulnerability scanning, non-root runtime configuration, provenance generation, signing, and SBOM generation.

#### Scenario: Pinned images are prepared for release
- **WHEN** a base-image change is submitted
- **THEN** repository-standard validation demonstrates that all service images build and existing release hardening controls remain enabled
