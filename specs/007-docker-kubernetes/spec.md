# Feature Specification: Docker Microservices & Kubernetes Deployment

**Feature Branch**: `007-docker-kubernetes`  
**Created**: 2025-12-30  
**Status**: Draft  
**Input**: docs/TODO.md "Docker Microservices & Kubernetes Deployment" section

## Problem Statement

The EVE Frontier services are currently deployable only as AWS Lambda functions. To support
self-hosting, local development, and Kubernetes deployment scenarios, we need containerized
microservice equivalents of each Lambda function, with proper orchestration via Traefik and
Kubernetes deployment manifests.

## Goals

1. Create Docker images for each microservice (route, scout-gates, scout-range)
2. Ensure microservices implement the same API contracts as their Lambda equivalents
3. Provide Kubernetes deployment via Helm charts with Traefik ingress
4. Enable local development with Docker Compose
5. Maintain supply chain security with image signing and vulnerability scanning

## Non-Goals

1. Replacing the existing Lambda infrastructure (this is an alternative deployment)
2. Auto-scaling configuration (Kubernetes handles this natively)
3. Database deployment (SQLite is embedded in the dataset)
4. Custom domain/DNS configuration (user responsibility)

## User Scenarios & Testing

### User Story 1 - Local Development with Docker Compose (Priority: P1)

A developer wants to run the EVE Frontier API locally for testing integrations or developing
client applications without deploying to AWS.

**Why this priority**: Enables developers to test without AWS costs, simplifies onboarding, and
provides a foundation for all other deployment scenarios.

**Independent Test**: Run `docker compose up` and verify all three endpoints respond correctly
with the fixture dataset.

**Acceptance Scenarios**:

1. **Given** Docker and Docker Compose are installed, **When** a user runs `docker compose up` in the
   repository root, **Then** all three services start and respond to HTTP requests within 30 seconds.
2. **Given** services are running, **When** a user sends a route request to `localhost:8080/route`,
   **Then** the response matches the Lambda API contract.
3. **Given** services are running, **When** a user sends a scout-gates request to
   `localhost:8080/scout/gates`, **Then** the response matches the Lambda API contract.
4. **Given** services are running, **When** a user sends a scout-range request to
   `localhost:8080/scout/range`, **Then** the response matches the Lambda API contract.

---

### User Story 2 - Kubernetes Deployment with Helm (Priority: P2)

A platform operator wants to deploy the EVE Frontier API to a Kubernetes cluster for production use,
with proper health checks, resource limits, and ingress configuration.

**Why this priority**: Production deployment capability is the primary goal, but requires
containerization (US1) to be complete first.

**Independent Test**: Deploy to a local Kubernetes cluster (kind/minikube) using Helm and verify
all endpoints respond via the ingress.

**Acceptance Scenarios**:

1. **Given** a Kubernetes cluster with Traefik installed, **When** a user runs
   `helm install evefrontier ./charts/evefrontier`, **Then** all three deployments start successfully.
2. **Given** the Helm release is deployed, **When** a user queries the ingress endpoint `/route`,
   **Then** the request is routed to the route service and returns a valid response.
3. **Given** the Helm release is deployed, **When** a pod is deleted, **Then** Kubernetes recreates
   it and the service recovers within 60 seconds.
4. **Given** the Helm release is deployed, **When** a user runs health check endpoints, **Then** all
   services report healthy status.

---

### User Story 3 - CI/CD Pipeline for Container Images (Priority: P3)

A maintainer wants container images to be automatically built, scanned, signed, and published to a
container registry on each release, following supply chain security best practices.

**Why this priority**: Automation enables sustainable releases but isn't required for initial
deployment capability.

**Independent Test**: Push a tag and verify images are published to the registry with signatures and
SBOMs attached.

**Acceptance Scenarios**:

1. **Given** a new release tag is pushed, **When** CI runs, **Then** multi-arch images (amd64, arm64)
   are built for all three services.
2. **Given** images are built, **When** CI completes, **Then** images are scanned for vulnerabilities
   with Trivy/Grype.
3. **Given** images pass scanning, **When** CI completes, **Then** images are signed with cosign and
   SBOMs are attached.
4. **Given** images are signed, **When** CI completes, **Then** images are pushed to the container
   registry with semantic version tags.

---

### Edge Cases

- What happens when the dataset file is missing from the container? → Service should fail fast with
  clear error message at startup.
- How does the system handle memory limits with large datasets? → Document minimum memory requirements.
- What if Traefik is not installed in the cluster? → Helm chart should work with any ingress
  controller or none (NodePort fallback).

## Requirements

### Functional Requirements

- **FR-001**: System MUST provide Dockerfiles for route, scout-gates, and scout-range microservices.
- **FR-002**: Dockerfiles MUST use multi-stage builds with Distroless base images for minimal
  runtime size.
- **FR-003**: Each container image MUST be less than 50MB compressed.
- **FR-004**: Microservices MUST implement the same API contracts as their Lambda equivalents.
- **FR-005**: System MUST provide a Docker Compose configuration for local development.
- **FR-006**: System MUST provide a Helm chart for Kubernetes deployment.
- **FR-007**: Helm chart MUST include Traefik IngressRoute resources for routing.
- **FR-008**: Helm chart MUST include health check probes (liveness and readiness).
- **FR-009**: Helm chart MUST allow configuration of resource limits via values.yaml.
- **FR-010**: CI MUST build multi-architecture images (amd64, arm64) on release.
- **FR-011**: CI MUST scan images for vulnerabilities before publishing.
- **FR-012**: CI MUST sign images with cosign using OIDC identity.
- **FR-013**: CI MUST generate and attach SBOMs to container images.
- **FR-014**: Each microservice MUST function with 128Mi memory minimum; 256Mi recommended for
  production workloads.

### Key Entities

- **Microservice**: Containerized HTTP service implementing one API endpoint (route, scout-gates,
  scout-range).
- **Helm Chart**: Kubernetes deployment package including deployments, services, ingress, and
  configuration.
- **Container Image**: Multi-arch Docker image published to registry with signatures.

## Success Criteria

### Measurable Outcomes

- **SC-001**: `docker compose up` starts all services within 30 seconds on a standard development
  machine.
- **SC-002**: Each container image is less than 50MB compressed.
- **SC-003**: Helm deployment completes on a fresh Kubernetes cluster within 2 minutes.
- **SC-004**: All three endpoints respond correctly to requests matching Lambda API contracts.
- **SC-005**: CI builds and publishes images in under 15 minutes.
- **SC-006**: Zero high/critical vulnerabilities in published container images.
