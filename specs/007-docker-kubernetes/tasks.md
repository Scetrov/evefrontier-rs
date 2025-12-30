# Tasks: Docker Microservices & Kubernetes Deployment

**Input**: Design documents from `/specs/007-docker-kubernetes/`
**Prerequisites**: plan.md (required), spec.md (required), research.md (technical decisions)

**Tests**: Integration tests via Docker Compose and API contract verification.

**Organization**: Tasks are organized by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[US#]**: Which user story this task belongs to

## Path Conventions

- Service crates: `crates/evefrontier-service-*/`
- Shared service infrastructure: `crates/evefrontier-service-shared/`
- Docker Compose: `docker-compose.yml` (root)
- Helm chart: `charts/evefrontier/`
- CI workflow: `.github/workflows/docker-release.yml`
- ADR: `docs/adrs/0014-containerization-strategy.md`

---

## Phase 1: Setup (ADR & Workspace Configuration)

**Purpose**: Document architectural decisions and prepare workspace for new crates

- [x] T001 Create ADR `docs/adrs/0014-containerization-strategy.md` documenting container framework (axum), base image (Distroless), multi-arch strategy (cargo-zigbuild), and registry (ghcr.io)
- [x] T002 Add service crates to `Cargo.toml` workspace members array
- [x] T003 [P] Create `crates/evefrontier-service-shared/Cargo.toml` with axum, tokio, tower-http, serde dependencies
- [x] T004 [P] Create `crates/evefrontier-service-shared/project.json` with Nx targets (build, test, lint, clippy)

---

## Phase 2: Foundational (Shared Service Infrastructure)

**Purpose**: Create reusable HTTP infrastructure for all microservices

**⚠️ CRITICAL**: All microservices depend on this shared crate

- [x] T005 Create `crates/evefrontier-service-shared/src/lib.rs` exporting health, request, response modules
- [x] T006 [P] Create `crates/evefrontier-service-shared/src/health.rs` with `/health/live` and `/health/ready` handlers
- [x] T007 [P] Create `crates/evefrontier-service-shared/src/request.rs` with RouteRequest, ScoutGatesRequest, ScoutRangeRequest types (mirror lambda-shared)
- [x] T008 [P] Create `crates/evefrontier-service-shared/src/response.rs` with JSON response types and error handling (mirror lambda-shared)
- [x] T009 Create `crates/evefrontier-service-shared/src/state.rs` with AppState holding Starmap and SpatialIndex
- [x] T010 Add unit tests for request parsing, response serialization, and AppState initialization in `crates/evefrontier-service-shared/src/lib.rs`

**Checkpoint**: Shared infrastructure validated and ready for microservice implementation

---

## Phase 3: User Story 1 - Local Development with Docker Compose (Priority: P1) 🎯 MVP

**Goal**: Developers can run `docker compose up` to start all services locally

**Independent Test**: Run `docker compose up` and curl all three endpoints successfully

### Route Microservice

- [x] T011 [US1] Create `crates/evefrontier-service-route/Cargo.toml` depending on evefrontier-lib, evefrontier-service-shared, axum, tokio
- [x] T012 [US1] Create `crates/evefrontier-service-route/project.json` with Nx targets
- [x] T013 [US1] Create `crates/evefrontier-service-route/src/main.rs` with axum router, /route POST handler, health endpoints
- [x] T014 [US1] Create `crates/evefrontier-service-route/Dockerfile` with multi-stage build (builder + distroless runtime); verify image <50MB
- [ ] T014a [US1] Add API contract test comparing service JSON response to Lambda response format (golden file at `docs/fixtures/route_response.golden.json`)
- [ ] T015 [P] [US1] Add integration test in `crates/evefrontier-service-route/tests/integration.rs` for route endpoint

### Scout-Gates Microservice

- [x] T016 [P] [US1] Create `crates/evefrontier-service-scout-gates/Cargo.toml` depending on evefrontier-lib, evefrontier-service-shared, axum, tokio
- [x] T017 [P] [US1] Create `crates/evefrontier-service-scout-gates/project.json` with Nx targets
- [x] T018 [US1] Create `crates/evefrontier-service-scout-gates/src/main.rs` with axum router, /scout/gates POST handler, health endpoints
- [x] T019 [US1] Create `crates/evefrontier-service-scout-gates/Dockerfile` with multi-stage build; verify image <50MB
- [ ] T019a [US1] Add API contract test comparing service JSON response to Lambda response format (golden file at `docs/fixtures/scout_gates_response.golden.json`)
- [ ] T020 [P] [US1] Add integration test in `crates/evefrontier-service-scout-gates/tests/integration.rs`

### Scout-Range Microservice

- [x] T021 [P] [US1] Create `crates/evefrontier-service-scout-range/Cargo.toml` depending on evefrontier-lib, evefrontier-service-shared, axum, tokio
- [x] T022 [P] [US1] Create `crates/evefrontier-service-scout-range/project.json` with Nx targets
- [x] T023 [US1] Create `crates/evefrontier-service-scout-range/src/main.rs` with axum router, /scout/range POST handler, health endpoints
- [x] T024 [US1] Create `crates/evefrontier-service-scout-range/Dockerfile` with multi-stage build; verify image <50MB
- [ ] T024a [US1] Add API contract test comparing service JSON response to Lambda response format (golden file at `docs/fixtures/scout_range_response.golden.json`)
- [ ] T025 [P] [US1] Add integration test in `crates/evefrontier-service-scout-range/tests/integration.rs`

### Docker Compose

- [x] T026 [US1] Create `docker-compose.yml` with route, scout-gates, scout-range services and Traefik reverse proxy
- [x] T027 [US1] Add `.dockerignore` to exclude target/, docs/, specs/ from build context
- [x] T028 [US1] Test `docker compose up` starts all services and responds to requests (tested with podman compose)
- [x] T029 [US1] Document local development in `docs/DEPLOYMENT.md` Docker Compose section

**Checkpoint**: User Story 1 complete - developers can run services locally with Docker Compose

---

## Phase 4: User Story 2 - Kubernetes Deployment with Helm (Priority: P2)

**Goal**: Platform operators can deploy to Kubernetes with `helm install`

**Independent Test**: Deploy to kind/minikube and verify all endpoints via ingress

### Helm Chart Structure

- [x] T030 [US2] Create `charts/evefrontier/Chart.yaml` with chart name, version, appVersion
- [x] T031 [US2] Create `charts/evefrontier/values.yaml` with default configuration (replicas, resources, ingress)
- [x] T032 [US2] Create `charts/evefrontier/templates/_helpers.tpl` with common template functions

### Deployments

- [x] T033 [P] [US2] Create `charts/evefrontier/templates/deployment-route.yaml` with pod spec, probes, resources
- [x] T034 [P] [US2] Create `charts/evefrontier/templates/deployment-scout-gates.yaml`
- [x] T035 [P] [US2] Create `charts/evefrontier/templates/deployment-scout-range.yaml`

### Services

- [x] T036 [P] [US2] Create `charts/evefrontier/templates/service-route.yaml` (ClusterIP, port 8080)
- [x] T037 [P] [US2] Create `charts/evefrontier/templates/service-scout-gates.yaml`
- [x] T038 [P] [US2] Create `charts/evefrontier/templates/service-scout-range.yaml`

### Ingress

- [x] T039 [US2] Create `charts/evefrontier/templates/ingress.yaml` with Traefik IngressRoute for path-based routing (conditional on `.Values.ingress.enabled`)
- [x] T039a [US2] Create `charts/evefrontier/templates/configmap.yaml` for optional runtime configuration
- [x] T039b [US2] Update `charts/evefrontier/values.yaml` to support NodePort Service type as fallback when ingress is disabled
- [x] T040 [US2] Create `charts/evefrontier/templates/middleware.yaml` with rate limiting and CORS middleware (optional)

### Chart Documentation & Testing

- [x] T041 [US2] Create `charts/evefrontier/README.md` with installation instructions, configuration options, examples
- [x] T042 [US2] Validate Helm chart with `helm lint charts/evefrontier`
- [ ] T043 [US2] Test deployment on kind cluster with Traefik installed
- [x] T044 [US2] Document Kubernetes deployment in `docs/DEPLOYMENT.md` Helm section

**Checkpoint**: User Story 2 complete - operators can deploy to Kubernetes with Helm

---

## Phase 5: User Story 3 - CI/CD Pipeline for Container Images (Priority: P3)

**Goal**: Automated build, scan, sign, and publish of container images on release

**Independent Test**: Push a tag and verify images appear in ghcr.io with signatures

### CI Workflow

- [x] T045 [US3] Create `.github/workflows/docker-release.yml` triggered on `v*` tags
- [x] T046 [US3] Add cargo-zigbuild setup step for cross-compilation (x86_64, aarch64)
- [x] T047 [US3] Add build step compiling all three services for both architectures
- [x] T048 [P] [US3] Add Dockerfile build step using docker buildx for multi-arch images
- [x] T049 [US3] Add Trivy scan step with severity CRITICAL,HIGH and exit-code 1
- [x] T050 [US3] Add cosign sign step using OIDC identity (keyless)
- [x] T051 [US3] Add SBOM generation step with syft or cargo-sbom
- [x] T052 [US3] Add push to ghcr.io step with version tags (v0.1.0, latest)
- [x] T053 [US3] Add workflow summary step reporting image digests and scan results

### CI Documentation

- [x] T054 [US3] Document CI/CD workflow in `docs/RELEASE.md` container images section
- [x] T055 [US3] Add container image verification instructions (cosign verify)

**Checkpoint**: User Story 3 complete - images automatically built and published on release

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Documentation updates and TODO completion

- [x] T056 [P] Update `README.md` with Docker/Kubernetes deployment overview
- [x] T057 [P] Update `docs/DEPLOYMENT.md` table of contents and cross-references
- [x] T058 Mark all Docker/Kubernetes items complete in `docs/TODO.md`
- [x] T059 Run `pnpm nx run-many -t lint -t clippy -t test` to validate all new crates
- [ ] T060 Commit changes with signed commit

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - creates ADR and workspace config
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all microservices
- **User Story 1 (Phase 3)**: Depends on Foundational - creates all microservices and Docker Compose
- **User Story 2 (Phase 4)**: Depends on User Story 1 - requires working container images
- **User Story 3 (Phase 5)**: Depends on User Story 1 - requires Dockerfiles to exist
- **Polish (Phase 6)**: Depends on all user stories complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P2)**: Requires container images from US1, but Helm chart can be developed in parallel
- **User Story 3 (P3)**: Requires Dockerfiles from US1, CI workflow can be developed in parallel with US2

### Within User Story 1

Parallel opportunities:
```
Task T016-T017: Scout-Gates crate setup (parallel with T011-T012 Route setup)
Task T021-T022: Scout-Range crate setup (parallel with above)
Task T015, T020, T025: Integration tests (parallel, different files)
```

Sequential requirements:
```
T011 → T012 → T013 → T014 (Route: Cargo.toml → project.json → main.rs → Dockerfile)
T026 → T027 → T028 (Docker Compose → .dockerignore → Test)
```

### Within User Story 2

Parallel opportunities:
```
Task T033-T035: All deployments can be created in parallel
Task T036-T038: All services can be created in parallel
```

Sequential requirements:
```
T030 → T031 → T032 (Chart.yaml → values.yaml → _helpers.tpl)
T039 → T040 → T041 (IngressRoute → Middleware → README)
T042 → T043 (Lint → Test on kind)
```

---

## Parallel Example: User Story 1 Microservices

```bash
# Phase 3 parallelization - after T010 (Foundational) completes:

# Launch in parallel (different crates):
Task T011-T015: Route microservice (5 tasks)
Task T016-T020: Scout-Gates microservice (5 tasks)
Task T021-T025: Scout-Range microservice (5 tasks)

# Then sequential:
Task T026-T029: Docker Compose and testing
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (ADR, workspace config)
2. Complete Phase 2: Foundational (shared crate)
3. Complete Phase 3: User Story 1 (microservices + Docker Compose)
4. **STOP and VALIDATE**: Test `docker compose up` and all endpoints
5. Ship MVP - developers can run locally

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test locally → MVP ready
3. Add User Story 2 → Test on kind → Kubernetes ready
4. Add User Story 3 → Test CI → Automated releases ready
5. Each story adds deployment capability without breaking previous

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks in same phase
- [US#] label maps task to specific user story for traceability
- **Constitution II compliance**: Microservice `main.rs` MUST be thin wrappers (~50 lines); all business logic lives in evefrontier-lib
- Request/response types are duplicated from lambda-shared for simplicity (TODO: extract to evefrontier-api-types crate if maintenance burden increases)
- All Dockerfiles use multi-stage builds with Distroless base (gcr.io/distroless/cc-debian12)
- **Image size target**: Each container image MUST be <50MB compressed (FR-003)
- **Memory requirements**: Minimum 128Mi, recommended 256Mi per service (for ~8 system fixture; scale for production dataset)
- Helm chart uses Traefik IngressRoute CRDs; NodePort fallback available when `ingress.enabled=false`
- **Build tool**: cargo-zigbuild for cross-compilation (not cross-rs)
