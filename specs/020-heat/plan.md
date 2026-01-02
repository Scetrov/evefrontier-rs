# Implementation Plan: Heat Mechanics Integration

**Branch**: `020-heat` | **Date**: 2026-01-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/020-heat/spec.md`

**Note**: This plan implements heat generation calculations and status line display for EVE Frontier route planning.

## Summary

Integrate heat mechanics into route planning by calculating per-hop heat generation using the community-validated formula from `HEAT_MECHANICS.md`. Display heat information in route status lines (per-hop generation and cumulative totals), add heat warnings when tolerance thresholds are exceeded, and update ADR 0015 with heat mechanics design decisions. Heat calculations will reuse existing ship data infrastructure and support both static and dynamic mass modes.

## Technical Context

**Language/Version**: Rust 1.91.1 (pinned via `.rust-toolchain`)  
**Primary Dependencies**: 
- `serde` 1.0 (JSON serialization for heat data in API responses)
- `thiserror` (error handling for heat validation failures)
- No new dependencies required (reuses existing ship data CSV parsing)

**Storage**: SQLite database (`static_data.db`) + CSV (`ship_data.csv`)  
**Testing**: `cargo test` with unit tests in `ship.rs` + integration tests in `tests/heat_calculations.rs`  
**Target Platform**: Linux/macOS/Windows CLI + AWS Lambda (cross-platform Rust)  
**Project Type**: Library-first (heat logic in `evefrontier-lib`), consumed by CLI and Lambda crates  
**Performance Goals**: Heat calculation overhead < 5% of total route planning time  
**Constraints**: 
- Must match community-validated heat formula from `HEAT_MECHANICS.md`
- No breaking changes to existing CLI/Lambda API contracts
- Heat data must serialize cleanly to JSON for Lambda responses

**Scale/Scope**: 
- Single new function `calculate_jump_heat()` in `ship.rs`
- Extension of existing `RouteStep` and `RouteSummary` structs in `output.rs`
- ~300-400 lines of new code across 3 files (`ship.rs`, `output.rs`, tests)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Test-Driven Development ✅ PASS

**Status**: Compliant  
**Justification**: Heat calculations will follow TDD:
1. Write tests in `tests/heat_calculations.rs` with known heat values from `HEAT_MECHANICS.md` examples
2. Implement `calculate_jump_heat()` to pass tests
3. Refactor for clarity (extract constants, simplify formula)

Minimum coverage: 80% for heat calculation paths (critical for accuracy).

### II. Library-First Architecture ✅ PASS

**Status**: Compliant  
**Justification**: All heat logic lives in `crates/evefrontier-lib/src/ship.rs` (library crate). CLI and Lambda crates call `attach_heat()` method on `RouteSummary` with no heat calculation logic in handlers. Heat generation is a pure function (`calculate_jump_heat(mass, distance, hull_mass)`) that uses the fixed calibration constant (`1e-7`) internally and has no I/O or console output.

### III. Architecture Decision Records ✅ PASS

**Status**: Compliant  
**Justification**: ADR 0015 (`fuel-cost-heat-impact-calculation.md`) already exists and documents heat formula design. This implementation updates ADR 0015 with:
- Heat formula implementation details (calibration constant, mass modes)
- Status line display format decisions
- Heat warning threshold rationale (75% warning, 100% error)

No new ADR required; this is completion of deferred work documented in ADR 0015 and TODO.md.

### IV. Clean Code & Cognitive Load ✅ PASS

**Status**: Compliant  
**Constraints**:
  - `calculate_jump_heat()` is a single-purpose function with McCabe complexity < 5
  - Heat accumulation loop in `attach_heat()` mirrors existing fuel accumulation (consistent pattern)
  - Constants named clearly: `HEAT_PROPORTIONALITY_FACTOR` (calibration constant is fixed at `1e-7` and not exposed to callers)
- No nesting depth > 2 levels (simple conditional for gate vs. jump)

### V. Security-First Development ✅ PASS

**Status**: Compliant  
**Justification**: Heat calculations use validated numeric inputs:
- Mass, distance, hull_mass validated in `ShipLoadout::new()` (existing)
- Calibration constant is a compile-time constant (no user input)
- Heat formula cannot overflow (all inputs finite, positive, bounded by physical constraints)
- No external data sources or new attack surface

### Summary

**Overall**: ✅ **PASS** — All gates satisfied. Proceed to Phase 0 research.

No complexity violations requiring justification. Heat feature is a natural extension of existing fuel calculation infrastructure with minimal architectural impact.

## Project Structure

### Documentation (this feature)

```text
specs/020-heat/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (community formula validation)
├── data-model.md        # Phase 1 output (HeatProjection, HeatSummary structs)
├── quickstart.md        # Phase 1 output (heat CLI examples)
└── contracts/           # Phase 1 output (Lambda JSON schema for heat fields)
```

### Source Code (repository root)

```text
crates/evefrontier-lib/src/
├── ship.rs              # Add: calculate_jump_heat(), HeatProjection, HeatConfig
├── output.rs            # Extend: RouteStep.heat, RouteSummary.heat, HeatSummary
├── routing.rs           # (No changes; heat attached post-route planning)
└── lib.rs               # Export new heat types

crates/evefrontier-lib/tests/
├── heat_calculations.rs # New: Unit tests for heat formula
└── routing.rs           # Extend: Integration tests for heat in routes

crates/evefrontier-cli/src/
└── commands/route.rs    # (No changes; heat auto-enabled with --ship flag)

crates/evefrontier-lambda-route/src/
└── main.rs              # (No changes; heat included in RouteResponse via RouteSummary)

docs/adrs/
└── 0015-fuel-cost-heat-impact-calculation.md  # Update: Heat implementation details

docs/
├── HEAT_MECHANICS.md    # (Reference only; validated formula source)
└── USAGE.md             # Update: Heat output examples
```

**Structure Decision**: Library-first implementation. Heat calculation logic in `ship.rs` (alongside fuel calculations), output formatting in `output.rs`. CLI and Lambda crates consume heat data via `RouteSummary` with zero additional code (automatic inclusion when ship data provided).

## Complexity Tracking

**No violations** — All constitution gates pass without exceptions. This section is included for completeness per template requirements but contains no entries.

---

## Phase 0: Outline & Research

### Objectives

1. Validate heat formula from `HEAT_MECHANICS.md` against community sources
2. Extract test cases with known heat values for TDD
3. Identify integration points in existing fuel calculation infrastructure
4. Research heat dissipation mechanics (deferred scope confirmation)

### Research Tasks

| Task ID | Description | Deliverable |
|---------|-------------|-------------|
| R0.1 | Validate heat formula constants (3, C=1.0) against EVE Frontier community sources | research.md § Formula Validation |
| R0.2 | Extract 5+ test cases with known heat values (ship, mass, distance, expected heat) | research.md § Test Cases |
| R0.3 | Document mass calculation modes (static vs. dynamic) impact on heat | research.md § Mass Modes |
| R0.4 | Confirm heat dissipation is out-of-scope (requires time-based routing) | research.md § Deferred Features |
| R0.5 | Review existing fuel calculation patterns for reuse in heat calculations | research.md § Integration Patterns |

### Research Questions

1. **Formula Calibration**: Is C=1.0 confirmed by community testing? Any alternative values?
2. **Heat Thresholds**: Is 75% warning / 100% error the right threshold? Game mechanics evidence?
3. **Mass Modes**: Does dynamic mass mode significantly affect heat compared to static mode?
4. **Edge Cases**: What happens with zero-distance jumps (gate transitions)? Heat = 0 confirmed?
5. **Display Format**: What precision should heat values use? (Fuel uses 2 decimal places)

### Output

- `specs/020-heat/research.md` — Consolidated findings with:
  - Decision: Heat formula confirmed with C=1.0, 3x proportionality factor
  - Rationale: Community-validated, matches in-game observations
  - Alternatives considered: C=0.5, C=2.0 (rejected: inconsistent with player data)
  - Test cases: 5 scenarios with ship/mass/distance/expected_heat

---

## Phase 1: Design & Contracts

**Prerequisites:** `research.md` complete, all formula constants validated

### Data Model Design

Create `specs/020-heat/data-model.md` with:

#### Entities

1. **HeatProjection** (per-hop heat data)
  - Fields: `hop_heat`, `cumulative_heat`, `warning` (no per-ship percentage; warnings use canonical thresholds)
  - Relationships: Embedded in `RouteStep`
  - Validation: `hop_heat >= 0.0`, `cumulative_heat >= hop_heat`

2. **HeatSummary** (route-level heat aggregate)
  - Fields: `total`, `warnings` (warnings contain entries when canonical thresholds are exceeded)
  - Relationships: Embedded in `RouteSummary`
  - Validation: `total >= 0.0`; `warnings` contains entries when `total >= HEAT_OVERHEATED` or `total >= HEAT_CRITICAL`

3. **HeatConfig** (calculation configuration)
   - Fields: `calibration_constant`, `dynamic_mass`
   - Relationships: Passed to `calculate_jump_heat()`
   - Validation: `calibration_constant > 0.0`

#### State Transitions

Heat accumulation state machine:
- **Initial**: `cumulative_heat = 0.0`
- **Per-hop**: `cumulative_heat += hop_heat` (jump) or `cumulative_heat += 0.0` (gate)
- **Warning**: `cumulative_heat >= HEAT_OVERHEATED` → add overheated warning
- **Error**: `cumulative_heat >= HEAT_CRITICAL` → add critical warning and mark as infeasible

### API Contracts

Generate `specs/020-heat/contracts/heat_response.json` (OpenAPI fragment):

```json
{
  "RouteStep": {
    "heat": {
      "type": "object",
      "nullable": true,
      "properties": {
        "hop_heat": { "type": "number", "format": "double", "minimum": 0 },
        "cumulative_heat": { "type": "number", "format": "double", "minimum": 0 },
        "wait_time_seconds": { "type": "number", "format": "double", "minimum": 0, "nullable": true },
        "cooled_cumulative_heat": { "type": "number", "format": "double", "minimum": 0, "nullable": true },
        "can_proceed": { "type": "boolean" },
        "warning": { "type": "string", "nullable": true }
      }
    }
  },
  "RouteSummary": {
    "heat": {
      "type": "object",
      "nullable": true,
      "properties": {
        "total": { "type": "number", "format": "double", "minimum": 0 },
        "warnings": { "type": "array", "items": { "type": "string" } }
      }
    }
  }
}
```

### Quickstart Guide

Create `specs/020-heat/quickstart.md` with CLI examples:

```bash
# Basic route with heat projection (Reflex ship, full fuel, no cargo)
evefrontier-cli route Nod Brana --ship Reflex

# Heat projection with dynamic mass mode
evefrontier-cli route Nod Brana --ship Reflex --dynamic-mass

# Heat projection with cargo (affects mass and heat)
evefrontier-cli route Nod Brana --ship Reflex --cargo-mass 500000

# JSON output with heat data
evefrontier-cli route Nod Brana --ship Reflex --format json
```

Expected output snippet:
```text
Route: Nod → Brana (3 hops, algorithm: a-star)
  0: Nod (30000001; min 2.73K, 2 planets, 2 moons)
  1: D:2NAS (30000003; min 2.73K, 4 planets, 9 moons) [Jump 18.95 ly] [Heat: +88.96 (89.0 / 1000.0)]
  2: G:3OA0 (30000004; min 2.73K, 3 planets, 7 moons) [Jump 38.26 ly] [Heat: +179.73 (268.7 / 1000.0)]
  3: Brana (30000002; min 2.73K, 3 planets, 7 moons) [Jump 23.09 ly] [Heat: +108.39 (377.1 / 1000.0)]

via 0 gates / 3 jump drive
Ship: Reflex | Fuel: 99.44 / 1,750 | Heat: 377.1 / 1000.0 (37.7%)
```

### Agent Context Update

Run `.specify/scripts/bash/update-agent-context.sh copilot` after creating contracts to:
- Add heat calculation technology (no new dependencies)
- Update context with heat formula constants
- Preserve existing manual additions

### Outputs

- `specs/020-heat/data-model.md` — HeatProjection, HeatSummary, HeatConfig entities
- `specs/020-heat/contracts/heat_response.json` — Lambda JSON schema
- `specs/020-heat/quickstart.md` — CLI usage examples
- `.github/copilot-instructions.md` — Updated with heat calculation context

### Constitution Re-Check

After Phase 1 design:
- ✅ TDD: Test cases defined in research.md, ready for implementation
- ✅ Library-first: All heat logic in `ship.rs`, no CLI/Lambda business logic
- ✅ ADR: ADR 0015 update plan documented in research.md
- ✅ Clean code: `calculate_jump_heat()` is single-purpose, complexity < 5
- ✅ Security: No new external inputs, numeric validation reuses existing patterns

**Result**: ✅ **PASS** — Design maintains constitution compliance. Proceed to Phase 2 planning.

---

## Phase 2: Implementation Planning (Stopping Point)

**Note**: This phase is planned but not executed by `/speckit.plan`. Execution occurs via `/speckit.tasks` command.

### Implementation Scope

1. **ship.rs** — Heat calculation functions
   - `calculate_jump_heat(mass, distance, hull_mass, config)` — Pure function
   - `HeatProjection` struct (per-hop data)
   - `HeatConfig` struct (calibration, mass mode)
   - Unit tests for formula validation

2. **output.rs** — Heat display integration
   - Extend `RouteStep` with `heat: Option<HeatProjection>`
   - Extend `RouteSummary` with `heat: Option<HeatSummary>`
   - `attach_heat()` method (mirrors `attach_fuel()` pattern)
   - Update `render_plain()`, `render_rich()`, `render_note()` for heat status lines

3. **tests/heat_calculations.rs** — Integration tests
   - Test cases from research.md
   - Static vs. dynamic mass mode comparison
   - Heat warning threshold validation
   - Gate transition zero-heat validation

4. **ADR 0015** — Documentation update
   - Add "Heat Implementation Details" section
   - Document calibration constant rationale
   - Update status line format examples

5. **USAGE.md** — User documentation
   - Add heat output examples
   - Document heat warning interpretation
   - Include dynamic mass mode heat comparison

### Task Breakdown Preview

(Detailed task list will be generated by `/speckit.tasks` command)

- T1: Write heat calculation tests (TDD red phase)
- T2: Implement `calculate_jump_heat()` function (TDD green phase)
- T3: Add `HeatProjection` and `HeatSummary` structs
- T4: Implement `attach_heat()` in `output.rs`
- T5: Update route rendering for heat status lines
- T6: Add integration tests for heat in routes
- T7: Update ADR 0015 with heat implementation details
- T8: Update USAGE.md with heat examples
- T9: Verify all tests pass, run complexity checks

### Acceptance Criteria

- [ ] All heat calculation tests pass (100% coverage for heat paths)
- [ ] CLI route output includes heat when `--ship` provided
- [ ] Heat warnings appear at 75% and 100% thresholds
- [ ] JSON output includes `heat` field in `RouteStep` and `RouteSummary`
- [ ] ADR 0015 updated with implementation details
- [ ] `cargo clippy` passes (complexity < 15 for all functions)
- [ ] `cargo test --workspace` passes (no regressions)

---

## Next Steps

1. **Review this plan** — Ensure all stakeholders agree with design decisions
2. **Execute Phase 0** — Run research tasks, generate `research.md`
3. **Execute Phase 1** — Generate data models, contracts, quickstart guide
4. **Run `/speckit.tasks`** — Generate detailed implementation task list
5. **Implement** — Follow TDD cycle (red-green-refactor) for each task

**Branch**: `020-heat` (already created)  
**Estimated Effort**: 2-3 days (research: 0.5 days, design: 0.5 days, implementation: 1-2 days)  
**Dependencies**: None (reuses existing fuel calculation infrastructure)

---

## References

- `docs/HEAT_MECHANICS.md` — Community-validated heat formula
- `docs/adrs/0015-fuel-cost-heat-impact-calculation.md` — Fuel/heat ADR
- `crates/evefrontier-lib/src/ship.rs` — Existing fuel calculation patterns
- `crates/evefrontier-lib/src/output.rs` — Route output formatting
- `.github/copilot-instructions.md` — Repository conventions
- `.specify/memory/constitution.md` — Development principles
