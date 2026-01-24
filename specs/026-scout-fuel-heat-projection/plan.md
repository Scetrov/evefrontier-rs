# Implementation Plan: Scout Fuel and Heat Projection

**Branch**: `026-scout-fuel-heat-projection` | **Date**: 2026-01-24 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/026-scout-fuel-heat-projection/spec.md`

## Summary

Enhance `scout range` to calculate fuel and heat projections when a ship is specified. Systems will
be ordered using a nearest-neighbor heuristic (Hamiltonian path approximation), with per-hop and
cumulative fuel/heat tracking. Reuses existing `calculate_jump_fuel_cost` and `calculate_jump_heat`
functions from `evefrontier-lib`.

## Technical Context

**Language/Version**: Rust 1.91.1 (per `.rust-toolchain`)  
**Primary Dependencies**: clap (CLI args), evefrontier-lib (fuel/heat calculations, ship catalog)  
**Storage**: SQLite dataset (read-only), ship_data.csv (ship catalog)  
**Testing**: cargo test with fixtures in `docs/fixtures/minimal/static_data.db`  
**Target Platform**: Linux CLI (x86_64, aarch64)  
**Project Type**: Rust workspace (crates/evefrontier-cli, crates/evefrontier-lib)  
**Performance Goals**: Nearest-neighbor heuristic <100ms for 100 systems (NFR-1)  
**Constraints**: Reuse existing fuel/heat code (NFR-2), match route output format (NFR-3)  
**Scale/Scope**: Typical 5-20 systems per scout query; max 100 per --limit

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-Driven Development | ✅ PASS | Will write tests first for nearest-neighbor algorithm and fuel/heat projection |
| II. Library-First Architecture | ✅ PASS | Fuel/heat calculations already in evefrontier-lib; CLI is thin wrapper |
| III. ADRs for Significant Decisions | ✅ PASS | No new architectural decisions; reusing ADR 0015 fuel/heat patterns |
| IV. Clean Code & Cognitive Load | ✅ PASS | Will extract nearest-neighbor into separate function; complexity <15 |
| V. Security-First Development | ✅ PASS | No external inputs beyond existing CLI validation |
| VI. Testing Tiers | ✅ PASS | Unit tests for algorithm, integration tests for CLI output |
| VII. Refactoring & Technical Debt | ✅ PASS | No new debt; extending existing patterns |

## Project Structure

### Documentation (this feature)

```text
specs/026-scout-fuel-heat-projection/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── cli-interface.md
└── tasks.md             # Phase 2 output (NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
crates/
├── evefrontier-lib/
│   └── src/
│       └── ship.rs           # Existing: calculate_jump_fuel_cost, calculate_jump_heat
└── evefrontier-cli/
    ├── src/
    │   ├── main.rs           # ScoutRangeArgs - add ship options
    │   ├── commands/
    │   │   └── scout.rs      # execute_scout_range - add route planning logic
    │   └── output_helpers.rs # Add fuel/heat formatting for scout
    └── tests/
        └── scout_range.rs    # Add fuel/heat projection tests
```

**Structure Decision**: Extending existing crate structure. No new crates needed; all changes fit
within established patterns from spec 025 (Scout CLI Subcommand).

## Complexity Tracking

> No Constitution violations requiring justification.

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Nearest-neighbor produces poor routes | Low | Medium | Acceptable for greedy heuristic; document as approximation |
| Fuel/heat calculations diverge from route | Low | High | Reuse identical library functions |
| Performance regression with 100 systems | Low | Medium | O(n²) is acceptable for n≤100 |

## Acceptance Criteria

1. `scout range --ship Reflex` shows fuel/heat projections in visiting order
2. Output format matches route command's enhanced style
3. REFUEL/OVERHEATED warnings display when thresholds exceeded
4. Nearest-neighbor algorithm produces reasonable visiting order
5. JSON output includes all fuel/heat fields
6. Tests cover fuel calculation accuracy and edge cases
