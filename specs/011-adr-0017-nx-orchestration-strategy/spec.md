# Feature Specification: ADR 0017 - NX Repository Orchestration Strategy

**Feature Branch**: `011-adr-0017-nx-orchestration-strategy`  
**Created**: 2025-12-31  
**Status**: Draft  
**Input**: Document ADR for implicit Nx orchestration patterns currently implemented in the
workspace

## User Scenarios & Testing

### User Story 1 - Developer understands Nx task orchestration strategy (Priority: P1)

A developer or maintainer needs to understand why Nx was chosen, how tasks are organized, and what
patterns govern their configuration. Currently, this knowledge is implicit in nx.json and
project.json files with no formal documentation.

**Why this priority**: This is essential for project maintainability, onboarding new contributors,
and making informed architectural decisions about build system changes.

**Independent Test**: An ADR that clearly explains Nx rationale, target configuration patterns, CI
integration, and custom patterns for Rust crates can be read and understood without referring to
external docs.

**Acceptance Scenarios**:

1. **Given** a developer joining the project, **When** they read ADR 0017, **Then** they understand
   why Nx was chosen over plain Cargo
2. **Given** a maintainer needing to add a new Rust crate, **When** they consult ADR 0017, **Then**
   they know exactly what project.json configuration is required
3. **Given** a contributor debugging CI failures, **When** they review ADR 0017's CI integration
   section, **Then** they understand how tasks flow through GitHub Actions
4. **Given** someone planning to refactor Nx configuration, **When** they read ADR 0017, **Then**
   they understand the design trade-offs that inform current choices

---

### User Story 2 - Documentation links Nx decisions to repository architecture (Priority: P2)

The workspace uses Nx for multiple domains: Rust build/test/lint, Python scripts, Node-based
tooling. An ADR should clarify how Nx unifies these and the decision trade-offs.

**Why this priority**: Helps future decision-makers understand whether Nx is the right tool for
future expansions (e.g., frontend, documentation generation).

**Independent Test**: ADR 0017 includes sections documenting rationale, alternatives considered, and
domain-specific patterns that explain the chosen architecture.

**Acceptance Scenarios**:

1. **Given** a proposal to add frontend tooling, **When** consulted about Nx, **Then** ADR 0017
   explains whether Nx should expand or a separate tool should be used
2. **Given** a performance issue with Nx caching, **When** reviewed against ADR 0017, **Then**
   engineers understand if it's a misconfiguration or architectural limitation

---

### User Story 3 - Document custom task patterns for Rust and non-standard targets (Priority: P2)

The workspace has implemented custom patterns for Rust crate orchestration (e.g., `parallel: false`,
`dependsOn` chains, shared inputs for `sharedGlobals`). These patterns need documentation so future
tasks follow the same conventions.

**Why this priority**: Ensures consistency and prevents misconfigurations that could break build
reproducibility or CI guarantees.

**Independent Test**: ADR 0017 includes a complete reference guide showing examples of each pattern
applied correctly.

**Acceptance Scenarios**:

1. **Given** adding a new task to a Rust crate's project.json, **When** consulted against ADR 0017,
   **Then** developer follows the correct input/output/cache conventions
2. **Given** the need to add a new global task (e.g., benchmark), **When** reviewer consults ADR
   0017, **Then** they can approve/reject based on documented patterns

---

## Requirements

### Functional Requirements

- **FR-001**: ADR MUST document rationale for selecting Nx over alternatives (plain Cargo, Bazel,
  other polyrepo tools)
- **FR-002**: ADR MUST describe the target configuration patterns applied to all projects (build,
  test, lint, clippy, audit, outdated, complexity)
- **FR-003**: ADR MUST explain input/output hashing strategy and caching decisions (what is cached,
  what is not, why)
- **FR-004**: ADR MUST document how tasks are orchestrated in CI/CD workflows (GitHub Actions
  integration, task sequencing, artifact handling)
- **FR-005**: ADR MUST provide concrete examples of project.json configurations for different crate
  types (library, CLI, Lambda, service)
- **FR-006**: ADR MUST document custom patterns for Rust-specific concerns (parallel compilation,
  dependency relationships, fixture inputs)
- **FR-007**: ADR MUST explain the rationale behind `parallel: false` for Rust tasks and when this
  might be reconsidered
- **FR-008**: ADR MUST describe how Nx task outputs integrate with release workflows (artifact
  generation, versioning)
- **FR-009**: ADR MUST include troubleshooting section for common issues (cache invalidation, task
  order dependencies, daemon problems)
- **FR-010**: ADR MUST reference implementation in nx.json and project.json files to ground
  discussion in concrete examples

### Non-Functional Requirements

- Document MUST be accessible to developers without deep Nx expertise; explain concepts clearly
- Document MUST be concise (target: <15 pages); refer to Nx docs for deep dives on specific features
- Document MUST follow ADR format defined in ADR 0001 (problem statement, decision, consequences,
  alternatives)
- Document structure MUST align with other ADRs (0006 for components, 0007 for DevSecOps, 0009 for
  spatial index)

### Key Entities

- **Nx Workspace**: Configuration file (`nx.json`) defining task defaults, named inputs, caching
  strategy, and plugin setup
- **Project Configuration**: Per-project file (`project.json`) extending workspace defaults with
  custom targets and executor configuration
- **Task Target**: Named executable unit (e.g., `build`, `test`, `clippy`) with defined inputs,
  outputs, dependencies, and caching behavior
- **Named Input**: Glob pattern (e.g., `production`, `sharedGlobals`) used to determine task cache
  keys and input hashing
- **Task Dependency**: Ordering relationship between targets (e.g., `test` depends on `build`) that
  ensures correct execution sequence
- **Executor**: The tool that actually runs a target (e.g., `@nx/cargo:build`, `@nx/cargo:test`,
  `nx/tasks-runners/default`)

### Edge Cases

- What happens when a Rust dependency changes that affects multiple crates? (Task invalidation
  strategy)
- How does Nx handle out-of-tree build artifacts (e.g., `/target/release/`)? (Outputs configuration)
- Should spatial index rebuilds be treated as cached or uncached? (Performance vs. correctness
  trade-off)
- What if a developer disables Nx daemon (NX_DAEMON=false)? (Fallback behavior and performance
  implications)
- How should Lambda deployment artifacts be handled in Nx? (Include spatial index, ship data, etc.)

## Context & Prior Work

This ADR formalizes the Nx orchestration strategy already implemented in the workspace:

- **Current State**: nx.json and project.json files configured per ADR 0006 (Software Components)
  and ADR 0007 (DevSecOps)
- **Workspace Layout**: 6 Rust crates (lib, cli, 3 lambda, 1 service shared) + 9 service crates +
  scripts
- **Task Targets Defined**: build, test, lint, clippy, audit, outdated, complexity
- **CI Integration**: Pre-commit hooks via rusty-hook, GitHub Actions workflows (ci.yml,
  release.yml, docker-release.yml)
- **Performance Goals**: Fast local dev loop with aggressive caching, reproducible CI builds with
  pinned toolchains

## References

- ADR 0001: Use Nygard ADR format
- ADR 0006: Software Components (describes Nx workspace structure)
- ADR 0007: DevSecOps Practices (describes CI/CD integration)
- ADR 0009: Spatial Index (describes caching of generated artifacts)
- [Nx Documentation](https://nx.dev/) (for reader deep dives)
- Repository Files: nx.json, crates/_/project.json, .github/workflows/_.yml

## Acceptance Criteria

- [ ] ADR 0017 draft created in `docs/adrs/0017-nx-orchestration-strategy.md`
- [ ] ADR follows Nygard format (problem, decision, consequences, alternatives)
- [ ] All target configuration patterns explained with examples
- [ ] CI integration section documents GitHub Actions task execution
- [ ] Troubleshooting section added with common issues and solutions
- [ ] ADR reviewed and approved per governance in ADR 0001
- [ ] `.github/copilot-instructions.md` or AGENTS.md updated if needed to reference ADR 0017
- [ ] CONTRIBUTING.md updated to reference ADR 0017 for task configuration guidance
