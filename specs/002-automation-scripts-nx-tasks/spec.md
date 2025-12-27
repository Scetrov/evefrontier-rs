# Feature Specification: Automation Scripts & Nx Task Integration

**Feature Branch**: `002-automation-scripts-nx-tasks`  
**Created**: 2025-12-27  
**Status**: Draft  
**Input**: From docs/TODO.md - "Introduce automation scripts under `scripts/` (e.g., dataset fixture
sync, release helpers) and register them as Nx tasks if applicable."

## User Scenarios & Testing

### User Story 1 - Unified Script Discovery via Nx (Priority: P1)

As a **developer**, I want to **discover and run automation scripts via Nx tasks**, so that **I can
use consistent tooling (`pnpm nx run <project>:<task>`) for all workspace operations**.

**Why this priority**: This is the core value proposition - unifying disparate scripts under Nx
orchestration provides consistency, caching, and task dependency management aligned with ADR 0006.

**Independent Test**: After implementation, `pnpm nx show project scripts` should list available
script tasks, and running them should execute correctly.

**Acceptance Scenarios**:

1. **Given** the scripts project is configured, **When** I run `pnpm nx show project scripts`,
   **Then** I see a list of available script tasks with descriptions
2. **Given** fixture verification scripts exist, **When** I run `pnpm nx run scripts:fixture-verify`,
   **Then** the script executes and reports fixture health status
3. **Given** a developer wants to sync fixtures, **When** they run `pnpm nx run scripts:fixture-sync`,
   **Then** the fixture is regenerated from the dataset and metadata updated

---

### User Story 2 - Fixture Management Automation (Priority: P1)

As a **test author**, I want **automated fixture synchronization and verification scripts**, so that
**test fixtures remain consistent with the e6c3 dataset schema and contents**.

**Why this priority**: Fixture management is critical for test reliability. The TODO specifically
calls out "dataset fixture sync" as an example use case.

**Independent Test**: Run fixture-verify against current fixtures and confirm pass; corrupt a fixture
and confirm failure is detected.

**Acceptance Scenarios**:

1. **Given** `minimal_static_data.db` exists with valid metadata, **When** I run fixture-verify,
   **Then** the script confirms hash and row counts match the `.meta.json` file
2. **Given** the fixture database differs from metadata, **When** I run fixture-verify, **Then** the
   script fails with a diff showing expected vs actual values
3. **Given** a fresh e6c3 dataset download, **When** I run fixture-sync with the dataset path,
   **Then** a new fixture is extracted and metadata JSON is updated
4. **Given** the fixture is already up to date, **When** I run fixture-sync, **Then** the script
   reports "Fixture already current" without modification

---

### User Story 3 - Database Inspection Utilities (Priority: P2)

As a **developer debugging schema issues**, I want **database inspection scripts available via Nx**,
so that **I can quickly inspect dataset contents without writing ad-hoc queries**.

**Why this priority**: Supports development workflow but not critical for CI/release.

**Acceptance Scenarios**:

1. **Given** a database path, **When** I run `pnpm nx run scripts:inspect-db -- <db-path>`,
   **Then** I see table names, row counts, and sample data
2. **Given** an invalid database path, **When** I run the inspect command, **Then** I see a clear
   error message

---

### User Story 4 - Release Preparation Helpers (Priority: P3)

As a **release engineer**, I want **release preparation scripts**, so that **I can automate version
bumping, changelog validation, and artifact preparation**.

**Why this priority**: Deferred until release workflow is defined (per other TODO items).

**Acceptance Scenarios**:

1. **Given** I'm preparing a release, **When** I run release-prepare, **Then** it validates
   CHANGELOG.md has an unreleased section, checks version consistency, and reports readiness
2. **Given** release blockers exist, **When** I run release-prepare, **Then** it fails with
   actionable items to resolve

---

## Requirements

### Functional Requirements

- **FR-001**: Create a `scripts/project.json` Nx project configuration for the scripts directory
- **FR-002**: Register Python scripts as Nx tasks using `nx:run-commands` executor
- **FR-003**: Register Node.js scripts as Nx tasks using `nx:run-commands` executor
- **FR-004**: Implement `fixture-verify` task that runs `fixture_status.py verify`
- **FR-005**: Implement `fixture-sync` task that runs `extract_fixture_from_dataset.py` with
  appropriate arguments
- **FR-006**: Implement `fixture-status` task that runs `fixture_status.py status`
- **FR-007**: Implement `inspect-db` task that runs `inspect_db.py` with passthrough arguments
- **FR-008**: All script tasks MUST support `--help` and provide usage information
- **FR-009**: Script tasks MUST exit with non-zero status on failure
- **FR-010**: Scripts MUST be runnable both directly (`python scripts/foo.py`) and via Nx
  (`pnpm nx run scripts:foo`)
- **FR-011**: Implement `venv-setup` task that creates and configures Python virtual environment
- **FR-012**: Implement `verify-all` meta-task that runs all verification tasks (fixture-verify, etc.)
- **FR-013**: `fixture-sync` MUST be runnable in CI environments (non-interactive)

### Non-Functional Requirements

- **NFR-001**: Scripts MUST use Python 3.10+ (matching development environment)
- **NFR-002**: Python scripts SHOULD use a virtual environment managed via `requirements.txt`
- **NFR-003**: Node.js scripts MUST use Node 20+ (matching .nvmrc)
- **NFR-004**: Task outputs SHOULD be cacheable where deterministic (e.g., verify with same input)
- **NFR-005**: Scripts MUST work in both CI and local environments without interactive input
- **NFR-006**: A `venv-setup` task MUST be provided for easy environment setup

### Key Entities

| Script | Language | Purpose | Nx Task Name |
|--------|----------|---------|--------------|
| `fixture_status.py` | Python | Verify fixture metadata | `fixture-verify`, `fixture-status`, `fixture-record` |
| `extract_fixture_from_dataset.py` | Python | Extract fixture from dataset | `fixture-sync` |
| `inspect_db.py` | Python | Database inspection | `inspect-db` |
| `create_minimal_db.py` | Python | Alternative fixture creation | `fixture-create` |
| `analyze_sample_routes.py` | Python | Route analysis | `analyze-routes` |
| `extract_route_fixture.py` | Python | Route fixture extraction | `route-fixture-extract` |
| `run-audit.js` | Node.js | Cargo audit wrapper | Already in use via Nx |
| `outdated-report.js` | Node.js | pnpm outdated reporter | Already exposed via package.json |
| `check-pnpm-outdated.js` | Node.js | pnpm outdated check | `check-outdated` |
| (new) `requirements.txt` | - | Python dependencies | Used by `venv-setup` |
| (meta) | - | Run all verification | `verify-all` |

### File Changes Summary

| File | Action | Description |
|------|--------|-------------|
| `scripts/project.json` | CREATE | New Nx project for scripts |
| `scripts/README.md` | CREATE | Documentation for scripts and Nx tasks |
| `scripts/requirements.txt` | CREATE | Python dependencies for venv |
| `nx.json` | MODIFY | Add scripts project inputs/caching rules |
| `CONTRIBUTING.md` | MODIFY | Document script usage via Nx |
| `docs/USAGE.md` | MODIFY | Add developer scripts section |
| `.github/workflows/ci.yml` | MODIFY | Add fixture-sync CI job (optional) |

---

## Technical Design

### Nx Project Structure

```json
// scripts/project.json
{
  "name": "scripts",
  "$schema": "../node_modules/nx/schemas/project-schema.json",
  "projectType": "application",
  "sourceRoot": "scripts",
  "tags": ["type:tooling", "lang:mixed", "scope:scripts"],
  "targets": {
    "fixture-verify": { ... },
    "fixture-status": { ... },
    "fixture-sync": { ... },
    "inspect-db": { ... },
    ...
  }
}
```

### Task Configuration Pattern

For Python scripts:
```json
{
  "executor": "nx:run-commands",
  "options": {
    "command": "python3 scripts/fixture_status.py verify",
    "cwd": "{workspaceRoot}"
  }
}
```

For tasks requiring arguments:
```json
{
  "executor": "nx:run-commands",
  "options": {
    "command": "python3 scripts/inspect_db.py {args.path}",
    "cwd": "{workspaceRoot}"
  }
}
```

### Caching Considerations

- `fixture-verify`: Cacheable based on fixture DB + metadata file
- `fixture-sync`: NOT cacheable (modifies files)
- `inspect-db`: NOT cacheable (ad-hoc queries)
- `analyze-routes`: Cacheable based on input CSV

---

## Edge Cases & Error Handling

1. **Python not installed**: Scripts should check for Python 3.10+ and fail with install instructions
2. **Missing fixture files**: Clear error pointing to fixture regeneration docs
3. **Database locked**: Retry logic or clear message about concurrent access
4. **Invalid arguments**: argparse-style help message with usage examples

---

## Dependencies

- Existing scripts in `scripts/` directory (no new external dependencies)
- Nx 20+ (already installed)
- Python 3.10+ (development dependency, documented in CONTRIBUTING.md)

---

## Out of Scope

- Release signing scripts (covered by separate TODO item)
- CI-specific scripts (already exist in `.github/scripts/`)
- Lambda deployment scripts (covered by Terraform TODO)

---

## Resolved Questions

1. **Should we add a virtual environment setup script for Python dependencies?**
   - **Answer**: YES - Use a venv for Python scripts
   - **Implementation**: Add `scripts/requirements.txt` and `venv-setup` task

2. **Should fixture-sync be runnable in CI, or only locally?**
   - **Answer**: BOTH - fixture-sync should work in CI and locally
   - **Implementation**: Ensure scripts work without interactive input, add CI job

3. **Should we add a `scripts:all` meta-task that runs all verification tasks?**
   - **Answer**: YES - Add meta-task for running all verification tasks
   - **Implementation**: Add `verify-all` task that depends on fixture-verify and other checks
