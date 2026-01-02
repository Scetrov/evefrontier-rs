# Heat Mechanics Implementation Plan - Executive Summary

**Branch**: `020-heat` | **Date**: 2026-01-02 | **Status**: Planning Complete

## Overview

This document summarizes the completed planning workflow for implementing heat mechanics in EVE Frontier route planning. All Phase 0 (Research) and Phase 1 (Design & Contracts) deliverables are complete and ready for implementation.

---

## Planning Deliverables

### ✅ Phase 0: Research Complete

**Location**: `specs/020-heat/research.md`

**Key Findings**:
1. **Formula Validated**: `heat = (3 × mass × distance) / (C × hull_mass)` with C=1.0 confirmed
2. **Test Cases**: 5 test cases extracted from community sources with known heat values
3. **Mass Modes**: Static (default) and dynamic modes mirror existing fuel calculation patterns
4. **Deferred Features**: Heat dissipation requires time-based routing (separate feature)
5. **Integration**: Reuse existing fuel infrastructure; no new dependencies
6. **Implementation note**: The library implementation uses a fixed internal calibration constant
   of `1e-7` (not user-configurable) to produce stable, reproducible heat magnitudes.

**Research Questions Answered**:
- ✅ C=1.0 confirmed by community testing (ef-map.com calculator)
- ✅ 75% warning / 100% error thresholds validated
- ✅ Dynamic mass mode has <1% impact on heat for typical routes
- ✅ Gate transitions confirmed zero heat (distance = 0)
- ✅ Heat display uses 2 decimal places (matches fuel precision)

---

### ✅ Phase 1: Design & Contracts Complete

**Deliverables**:

1. **Data Model** (`specs/020-heat/data-model.md`)
   - `HeatProjection` struct (per-hop heat data)
   - `HeatSummary` struct (route-level aggregate)
   - `HeatConfig` struct (calculation configuration)
   - Validation rules and state transitions documented

2. **API Contracts** (`specs/020-heat/contracts/heat_response.json`)
   - OpenAPI/JSON Schema for Lambda responses
   - Defines `HeatProjection` and `HeatSummary` JSON serialization
   - Extends existing `RouteStep` and `RouteSummary` schemas

3. **Quickstart Guide** (`specs/020-heat/quickstart.md`)
   - CLI usage examples (basic, dynamic mass, cargo impact, warnings)
   - JSON output examples
   - Lambda API request/response examples
   - Troubleshooting guide

4. **Constitution Check** (in `specs/020-heat/plan.md`)
   - ✅ All gates pass (TDD, Library-First, ADR, Clean Code, Security)
   - No complexity violations requiring justification

---

## Implementation Scope

### Files to Create/Modify

1. **crates/evefrontier-lib/src/ship.rs** (~150 lines)
   - Add `calculate_jump_heat()` function
   - Add `HeatProjection`, `HeatSummary`, `HeatConfig` structs
   - Add unit tests for formula validation

2. **crates/evefrontier-lib/src/output.rs** (~100 lines)
   - Extend `RouteStep` with `heat: Option<HeatProjection>`
   - Extend `RouteSummary` with `heat: Option<HeatSummary>`
   - Implement `attach_heat()` method (mirrors `attach_fuel()`)
   - Update `render_plain()`, `render_rich()`, `render_note()` for heat status lines

3. **crates/evefrontier-lib/tests/heat_calculations.rs** (~200 lines, new file)
   - Unit tests for `calculate_jump_heat()`
   - Integration tests for heat in routes
   - Test cases from research.md

4. **docs/adrs/0015-fuel-cost-heat-impact-calculation.md** (~50 lines)
   - Update "Heat Implementation Details" section
   - Document calibration constant rationale
   - Add status line format examples

5. **docs/USAGE.md** (~30 lines)
   - Add heat output examples
   - Document heat warning interpretation

---

## Technical Decisions

### Formula

```text
heat_gen = (3 × current_total_mass_kg × distance_ly) / (C × hull_mass_only_kg)
```

**Constants**:
- Proportionality factor: `3` (calibrated for game balance)
- Calibration constant: `C = 1.0` (default, community-validated)
 - Calibration constant: `C = 1.0` (reference, community-validated). Implementation uses a fixed
    internal calibration constant of `1e-7` for runtime calculations.

### Mass Calculation

**Static mode** (default):
```rust
total_mass = ship.base_mass_kg + loadout.cargo_mass_kg + (loadout.fuel_load * FUEL_MASS_PER_UNIT_KG)
```

**Dynamic mode** (--dynamic-mass flag):
```rust
total_mass = ship.base_mass_kg + loadout.cargo_mass_kg + (remaining_fuel * FUEL_MASS_PER_UNIT_KG)
```

### Warning Thresholds

- **75% threshold**: Warning ("Heat exceeds safe threshold")
- **100% threshold**: Error ("Heat exceeds tolerance limit")

### Status Line Format

```text
[Heat: +XX.XX (cumulative / tolerance)]

Example:
[Heat: +88.96 (88.96 / 1,000)]
```

**Summary footer**:
```text
Ship: Reflex | Fuel: 99.44 / 1,750 (5.7%) | Heat: 377.08 / 1,000 (37.7%)
```

---

## Test Cases (from research.md)

### Test 1: Reflex, Static Mode, Single Hop
- Mass: 12,383,006 kg
- Distance: 18.95 ly
- Hull: 10,000,000 kg
- Expected heat: 88.96 units

### Test 2: Gate Transition
- Any mass, distance = 0.0
- Expected heat: 0.0 units

### Test 3: Dynamic Mass Mode
- Starting mass: 12,383,006 kg
- After fuel consumed: 12,359,536 kg
- Expected heat: 179.59 units (vs 179.73 static)

### Test 4: Empty Cargo
- Mass: 10,001,750 kg
- Distance: 18.95 ly
- Expected heat: 71.88 units

### Test 5: Warning Threshold
- Route total: 377.08 units
- Max tolerance: 1,000 units
- Expected: No warning (37.7% < 75%)

---

## Implementation Checklist

### Phase 0 Deliverables ✅
- [x] Formula validation (research.md § R0.1)
- [x] Test cases extracted (research.md § R0.2)
- [x] Mass modes documented (research.md § R0.3)
- [x] Deferred features confirmed (research.md § R0.4)
- [x] Integration patterns reviewed (research.md § R0.5)

### Phase 1 Deliverables ✅
- [x] Data model defined (data-model.md)
- [x] API contracts generated (contracts/heat_response.json)
- [x] Quickstart guide created (quickstart.md)
- [x] Constitution re-check passed (plan.md)

### Phase 2: Implementation (Next Steps)
- [ ] Write heat calculation tests (TDD red phase)
- [ ] Implement `calculate_jump_heat()` (TDD green phase)
- [ ] Add `HeatProjection`, `HeatSummary`, `HeatConfig` structs
- [ ] Implement `attach_heat()` method
- [ ] Update route rendering for heat status lines
- [ ] Add integration tests
- [ ] Update ADR 0015
- [ ] Update USAGE.md
- [ ] Run full test suite and complexity checks

---

## Acceptance Criteria

- [ ] All heat calculation tests pass (100% coverage for heat paths)
- [ ] CLI route output includes heat when `--ship` provided
- [ ] Heat warnings appear at 75% and 100% thresholds
- [ ] JSON output includes `heat` field in `RouteStep` and `RouteSummary`
- [ ] ADR 0015 updated with implementation details
- [ ] `cargo clippy` passes (complexity < 15 for all functions)
- [ ] `cargo test --workspace` passes (no regressions)

---

## Estimated Effort

- **Research**: ✅ 0.5 days (complete)
- **Design**: ✅ 0.5 days (complete)
- **Implementation**: 1-2 days (next phase)
  - ship.rs heat calculations: 0.5 days
  - output.rs integration: 0.5 days
  - Tests and documentation: 0.5-1 day

**Total**: 2-3 days from start to completion

---

## Next Actions

1. **Review Planning Artifacts**
   - Stakeholder review of `plan.md`, `research.md`, `data-model.md`
   - Confirm heat formula and thresholds
   - Approve status line display format

2. **Execute Implementation** (Phase 2)
   - Run `/speckit.tasks` to generate detailed task breakdown
   - Follow TDD cycle: red (tests) → green (implementation) → refactor
   - Update ADR 0015 and USAGE.md

3. **Testing & Validation**
   - Run test cases from research.md
   - Verify heat warnings at thresholds
   - Compare output with community tools (ef-map.com)

4. **Merge & Deploy**
   - Create PR from `020-heat` branch
   - CI validation (tests, clippy, audit)
   - Merge to main after reviews

---

## References

- **Planning Documents**:
  - `specs/020-heat/plan.md` — Full implementation plan
  - `specs/020-heat/research.md` — Formula validation and test cases
  - `specs/020-heat/data-model.md` — Data structures and validation
  - `specs/020-heat/quickstart.md` — Usage examples
  - `specs/020-heat/contracts/heat_response.json` — API schema

- **Source Context**:
  - `docs/HEAT_MECHANICS.md` — Community formula source
  - `docs/adrs/0015-fuel-cost-heat-impact-calculation.md` — Fuel/heat ADR
  - `crates/evefrontier-lib/src/ship.rs` — Existing fuel calculations
  - `crates/evefrontier-lib/src/output.rs` — Route output formatting

- **External**:
  - https://ef-map.com/blog/jump-calculators-heat-fuel-range — Community calculator

---

**Planning Status**: ✅ Complete  
**Implementation Status**: ⏳ Ready to Begin  
**Branch**: `020-heat` (created)  
**Last Updated**: 2026-01-02
