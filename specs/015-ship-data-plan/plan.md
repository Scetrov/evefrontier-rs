# Implementation Plan: Ship Data & Fuel Calculations

**Branch**: `015-ship-data-plan` | **Date**: 2025-12-31 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/015-ship-data-plan/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement ADR 0015 fuel projection support: ingest `ship_data.csv`, expose ship catalog and fuel calculators in `evefrontier-lib`, and surface fuel projections across CLI, Lambda, and container outputs while keeping non-ship callers backward compatible.

## Technical Context

**Language/Version**: Rust 1.91.1  
**Primary Dependencies**: `rusqlite`, `serde`, `csv`, `thiserror`, `kiddo` (existing spatial index), `clap`, `aws_lambda_events`  
**Storage**: SQLite dataset (`static_data.db`) plus cached `ship_data.csv` in `evefrontier_datasets/`  
**Testing**: `cargo test --workspace`, CLI integration via `assert_cmd`, Lambda handler unit tests; fixture DB at `docs/fixtures/minimal_static_data.db`  
**Target Platform**: CLI (Linux/macOS/Windows), AWS Lambda (x86_64/aarch64), container images  
**Project Type**: Multi-crate Rust workspace (library-first with CLI/Lambda wrappers)  
**Performance Goals**: Fuel projection adds <5ms per CLI route (≤50 hops) and <10ms CPU overhead on Lambda/container; no material memory increase within 512 MB limit  
**Constraints**: Library-first, TDD, Lambda memory limits (~512 MB), backward compatibility for callers without ship data, cached downloads only over HTTPS  
**Scale/Scope**: Ship catalog expected in tens (<200 entries) with updates per dataset release; load catalog fully into memory per process

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- TDD required (tests precede implementation) — to be enforced in plan/tests.
- Library-first: business logic must live in `crates/evefrontier-lib` — enforced.
- ADR requirement: ADR 0015 exists but is **Proposed**; implementation must be blocked until ratified or explicitly approved.
- Branching: feature branch `015-ship-data-plan` active — pass.
- Security-first: validate external inputs (CSV, CLI params); avoid leaking sensitive info — to be addressed in design.

## Project Structure

### Documentation (this feature)

```text
specs/015-ship-data-plan/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
crates/
├── evefrontier-lib/           # Ship catalog, fuel calculators, schema detection
├── evefrontier-cli/           # Route CLI: ship flags, list-ships
├── evefrontier-lambda-route/  # Route handler with fuel projection response
├── evefrontier-lambda-shared/ # Shared Lambda bootstrap and models
├── evefrontier-service-*/     # Containerized services mirroring Lambda APIs
└── evefrontier-lambda-scout-* # (unchanged, no ship data)

docs/
├── USAGE.md                   # CLI examples (to update with fuel flags)
└── fixtures/                  # Test fixtures; add ship data fixture

tests/ (per-crate)             # Unit/integration tests using fixtures
```

**Structure Decision**: Use existing multi-crate workspace; add ship data/module to `evefrontier-lib`, surface flags in CLI, extend Lambda/service schemas without creating new crates.

## Default Values

- **Default Ship**: `Reflex` — applied when `--ship` not specified
- **Default Fuel Quality**: `10%` — applied when `--fuel-quality` not specified
- **Implementation**: Set as Clap argument defaults; ship catalog validates Reflex exists at startup

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| ADR 0015 status Proposed | Implementation depends on ADR ratification; proceed with planning only | Blocking planning would stall delivery; implementation will wait for ratification |
